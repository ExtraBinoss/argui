use argui_paint::{DisplayList, ImageAsset, VectorAsset};
use argui_text::{PreparedText, TextEngine};
use std::{collections::HashMap, mem::size_of, sync::Arc};
use wgpu::{SurfaceTarget, TextureFormat, TextureViewDescriptor};

use crate::{
    DamagePlan, DamageProfile, EffectGraphStats, RenderProfile, RendererConfig, RendererError,
    batch::{DrawBatch, DrawKind, build_batches},
    damage::{DamageGpu, DamageSnapshot, scene_damage},
    effect::EffectGpu,
    effect_graph::EffectGraph,
    gpu_canvas::CanvasGpu,
    gpu_profile::GpuProfiler,
    image::ImageGpu,
    offscreen::{TexturePool, TexturePoolStats},
    profile::FrameProfiler,
    quad::QuadGpu,
    text::TextGpu,
    vector::VectorGpu,
};

mod acquisition;
mod alpha;
mod api;
mod blur;
mod composite;
mod effect_damage;
mod effect_registry;
mod effects;
mod filter_shadow;
mod initialization;
mod offscreen_api;
mod retained;

mod configure;
mod device;
use configure::{drawable_size, srgb_target, surface_alpha_mode};
use device::next_device_generation;
use offscreen_api::offscreen_texture;
use retained::full_damage_profile;

mod frame_content;
use frame_content::FrameContent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderStatus {
    Presented,
    /// Frame acquisition timed out; request another redraw without new input.
    Retry,
    Reconfigure,
    RecreateSurface,
    Skipped,
}

#[derive(Clone)]
pub struct RendererDevice(Arc<RendererDeviceInner>);

struct RendererDeviceInner {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    initialization_fallback: Option<String>,
    generation: u64,
}

pub struct SurfaceRenderer {
    device_handle: RendererDevice,
    instance: wgpu::Instance,
    surface: Option<wgpu::Surface<'static>>,
    offscreen_target: Option<wgpu::Texture>,
    alpha_target: Option<wgpu::Texture>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    target_format: TextureFormat,
    renderer_config: RendererConfig,
    batches: Vec<DrawBatch>,
    text_ranges: Vec<std::ops::Range<u32>>,
    text_bounds: Vec<Option<argui_core::Rect>>,
    quad: QuadGpu,
    text: TextGpu,
    image: ImageGpu,
    vector: VectorGpu,
    gpu_canvas: CanvasGpu,
    effect: EffectGpu,
    offscreen: TexturePool,
    gpu_profiler: GpuProfiler,
    last_profile: RenderProfile,
    profiling_active: bool,
    layer_cache: HashMap<argui_paint::RenderObjectId, effects::CachedLayer>,
    content_revision: u64,
    damage: DamageGpu,
    scene_snapshot: Option<DamageSnapshot>,
    effect_root: Option<crate::target::TextureTarget>,
    deferred_reconfigure: bool,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Reclaims completed GPU submissions, waiting for the current frame when configured.
    fn poll_submitted_gpu_work(&self) {
        let poll_type = if self.renderer_config.wait_for_submitted_gpu_work {
            wgpu::PollType::wait_indefinitely()
        } else {
            wgpu::PollType::Poll
        };
        let _ = self.device.poll(poll_type);
    }

    /// Renders `content` to the configured target, calls `notify` after work is
    /// queued, and returns presentation status or a renderer error.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn render_frame(
        &mut self,
        content: FrameContent<'_>,
        notify: impl FnOnce(),
    ) -> Result<RenderStatus, RendererError> {
        // Acquisition can block on an occluded or remapped swapchain; include it
        // in CPU timing instead of hiding that wait from the native profiler.
        let profiler = FrameProfiler::start(self.profiling_active);
        // Preserve a valid suboptimal presentation until its replacement is ready.
        // A native resize may already have applied the replacement configuration.
        if std::mem::take(&mut self.deferred_reconfigure) {
            self.reconfigure_surface();
        }
        let (frame, texture, status) = if let Some(surface) = &self.surface {
            match acquisition::resolve(surface.get_current_texture())? {
                acquisition::Acquisition::Ready { frame, suboptimal } => {
                    let texture = frame.texture.clone();
                    self.deferred_reconfigure = suboptimal;
                    (Some(frame), texture, RenderStatus::Presented)
                }
                acquisition::Acquisition::Deferred(status) => return Ok(status),
            }
        } else {
            (
                None,
                self.offscreen_target
                    .as_ref()
                    .expect("offscreen target exists")
                    .clone(),
                RenderStatus::Presented,
            )
        };

        let viewport = [
            self.surface_config.width as f32,
            self.surface_config.height as f32,
        ];
        let gpu_capture = self.gpu_profiler.begin_frame(self.profiling_active);
        let mut graph_stats = EffectGraphStats::default();
        let mut effect_graph = None;
        let mut canvas_commands = Vec::new();
        let mut scene_update = None;
        self.text.clear_frame_stats();
        match content {
            FrameContent::None => {
                self.vector.clear_frame_stats();
                self.gpu_canvas.clear_frame_stats();
                self.batches.clear();
                self.text_bounds.clear();
            }
            FrameContent::Text { engine, text } => {
                self.vector.clear_frame_stats();
                self.gpu_canvas.clear_frame_stats();
                let draw = self.text.prepare(&self.device, &self.queue, engine, text)?;
                let range = draw.all();
                self.batches.clear();
                if !range.is_empty() {
                    self.batches.push(DrawBatch {
                        kind: DrawKind::Text,
                        instances: range,
                    });
                }
                self.text_bounds.clear();
            }
            FrameContent::Ui {
                engine,
                text,
                display_list,
                scale_factor,
            } => {
                let quad_changed =
                    self.quad
                        .prepare(&self.device, &self.queue, display_list, scale_factor)?;
                let image_changed =
                    self.image
                        .prepare(&self.device, &self.queue, display_list, scale_factor)?;
                let vector_changed =
                    self.vector
                        .prepare(&self.device, &self.queue, display_list, scale_factor)?;
                let canvas =
                    self.gpu_canvas
                        .prepare(&self.device, &self.queue, display_list, scale_factor);
                canvas_commands = canvas.command_buffers;
                let draw = self.text.prepare_ui(
                    &self.device,
                    &self.queue,
                    engine,
                    text,
                    display_list,
                    scale_factor,
                )?;
                let content_changed = quad_changed
                    || image_changed
                    || vector_changed
                    || canvas.changed
                    || draw.changed();
                if content_changed {
                    self.content_revision = self.content_revision.wrapping_add(1);
                }
                self.text_ranges = draw.ranges().to_vec();
                self.text_bounds = draw.bounds().to_vec();
                build_batches(display_list, &self.text_ranges, &mut self.batches);
                let mut graph = EffectGraph::build(
                    display_list,
                    &self.text_ranges,
                    viewport,
                    scale_factor,
                    self.content_revision,
                )
                .map_err(|error| RendererError::InvalidDisplayList(error.to_string()))?;
                let additional_effect_passes = self.validate_custom_effects(&graph)?;
                graph_stats = graph.stats();
                graph_stats.filter_passes += additional_effect_passes;
                if self.renderer_config.damage_tracking.enabled {
                    let snapshot = DamageSnapshot::capture(
                        display_list,
                        text,
                        draw.bounds(),
                        [viewport[0] as u32, viewport[1] as u32],
                        scale_factor,
                    );
                    let mut damage_plan = scene_damage(
                        self.scene_snapshot.as_ref(),
                        &snapshot,
                        self.renderer_config.damage_tracking,
                        &self.renderer_config.effects,
                    );
                    if content_changed && damage_plan == DamagePlan::Unchanged {
                        damage_plan = DamagePlan::Full;
                    }
                    graph.retain_layer_revisions(
                        self.scene_snapshot.as_ref(),
                        &snapshot,
                        content_changed
                            && self.scene_snapshot.as_ref().is_some_and(|previous| {
                                previous
                                    .unchanged_range(&snapshot, 0..display_list.commands().len())
                            }),
                        &|profile| {
                            self.layer_cache
                                .get(&profile)
                                .map(|cached| cached.layer.content_revision)
                        },
                    );
                    scene_update = Some((snapshot, damage_plan));
                }
                effect_graph = Some(graph);
            }
            FrameContent::Composite {
                display_list,
                scale_factor,
            } => {
                self.vector.clear_frame_stats();
                build_batches(display_list, &self.text_ranges, &mut self.batches);
                let graph = EffectGraph::build(
                    display_list,
                    &self.text_ranges,
                    viewport,
                    scale_factor,
                    self.content_revision,
                )
                .map_err(|error| RendererError::InvalidDisplayList(error.to_string()))?;
                let additional_effect_passes = self.validate_custom_effects(&graph)?;
                graph_stats = graph.stats();
                graph_stats.filter_passes += additional_effect_passes;
                if self.renderer_config.damage_tracking.enabled
                    && let Some(previous) = self.scene_snapshot.as_ref()
                {
                    let snapshot = previous.capture_composite(
                        display_list,
                        &self.text_bounds,
                        [viewport[0] as u32, viewport[1] as u32],
                        scale_factor,
                    );
                    let damage_plan = scene_damage(
                        Some(previous),
                        &snapshot,
                        self.renderer_config.damage_tracking,
                        &self.renderer_config.effects,
                    );
                    scene_update = Some((snapshot, damage_plan));
                }
                effect_graph = Some(graph);
            }
        }
        let surface_view = texture.create_view(&TextureViewDescriptor {
            format: Some(self.target_format),
            ..Default::default()
        });
        let view = self.scene_output_view(&texture);
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.quad.begin_frame();
        self.text.begin_frame();
        self.image.begin_frame();
        self.vector.begin_frame();
        self.gpu_canvas.begin_frame();
        if let Some(graph) = effect_graph
            && graph.needs_offscreen_root()
        {
            self.damage.invalidate();
            let full = DamagePlan::Full;
            let plan = scene_update
                .as_ref()
                .map_or(&full, |(_, damage_plan)| damage_plan);
            let (cached_layers, damaged_pixels, damage_profile) = self.render_effect_graph(
                &mut encoder,
                &view,
                &graph,
                viewport,
                plan,
                gpu_capture.as_ref(),
            );
            if let Some((snapshot, _)) = scene_update {
                self.scene_snapshot = Some(snapshot);
            } else {
                self.scene_snapshot = None;
            }
            graph_stats.cached_layers = cached_layers;
            graph_stats.damaged_pixels = damaged_pixels;
            self.present_alpha(&mut encoder, &surface_view, viewport, gpu_capture.as_ref());
            notify();
            if let Some(capture) = gpu_capture {
                capture.finish(&mut encoder);
            }
            canvas_commands.push(encoder.finish());
            self.queue.submit(canvas_commands);
            self.gpu_canvas.submitted(&self.queue);
            if let Some(frame) = frame {
                self.queue.present(frame);
            }
            self.poll_submitted_gpu_work();
            self.finish_profile(profiler, viewport, graph_stats, damage_profile, false);
            return Ok(status);
        }
        self.effect_root = None;
        self.layer_cache.clear();
        self.offscreen.clear();
        self.effect.begin_frame();
        let (damage_profile, direct_surface) = match scene_update {
            Some((snapshot, DamagePlan::Partial(regions))) => {
                let profile = self.draw_retained_scene(
                    &mut encoder,
                    &view,
                    viewport,
                    &regions,
                    gpu_capture.as_ref(),
                );
                self.scene_snapshot = Some(snapshot);
                (profile, false)
            }
            Some((snapshot, DamagePlan::Unchanged)) if self.damage.valid() => {
                let profile =
                    self.reuse_retained_scene(&mut encoder, &view, viewport, gpu_capture.as_ref());
                self.scene_snapshot = Some(snapshot);
                (profile, false)
            }
            Some((snapshot, DamagePlan::Full | DamagePlan::Unchanged)) => {
                self.damage.invalidate();
                self.draw_full_scene(
                    &mut encoder,
                    &view,
                    viewport,
                    gpu_capture.as_ref(),
                    "surface.main",
                );
                self.scene_snapshot = Some(snapshot);
                (full_damage_profile(viewport), true)
            }
            None => {
                self.damage.invalidate();
                self.scene_snapshot = None;
                self.draw_full_scene(
                    &mut encoder,
                    &view,
                    viewport,
                    gpu_capture.as_ref(),
                    "surface.main",
                );
                (full_damage_profile(viewport), true)
            }
        };
        self.present_alpha(&mut encoder, &surface_view, viewport, gpu_capture.as_ref());
        notify();
        if let Some(capture) = gpu_capture {
            capture.finish(&mut encoder);
        }
        canvas_commands.push(encoder.finish());
        self.queue.submit(canvas_commands);
        self.gpu_canvas.submitted(&self.queue);
        if let Some(frame) = frame {
            self.queue.present(frame);
        }
        self.poll_submitted_gpu_work();
        self.finish_profile(
            profiler,
            viewport,
            graph_stats,
            damage_profile,
            direct_surface && !self.is_transparent(),
        );
        Ok(status)
    }

    fn finish_profile(
        &mut self,
        profiler: FrameProfiler,
        viewport: [f32; 2],
        effects: EffectGraphStats,
        damage: DamageProfile,
        direct_surface: bool,
    ) {
        let texture_pool = self.effect_root.map_or_else(
            || self.offscreen.stats(),
            |root| self.offscreen.stats_excluding(root.texture),
        );
        if let Some(profile) = profiler.finish(RenderProfile {
            viewport_pixels: viewport[0] as u64 * viewport[1] as u64,
            draw_batches: self.batches.len(),
            effects,
            damage,
            texture_pool,
            vector_atlas: self.vector.stats(),
            text_atlas: self.text.stats(),
            gpu_canvases: self.gpu_canvas.stats(),
            direct_surface,
            adapter: self.gpu_profiler.adapter().clone(),
            gpu: self.gpu_profiler.take_latest(),
            ..RenderProfile::default()
        }) {
            self.last_profile = profile;
        }
    }
}
