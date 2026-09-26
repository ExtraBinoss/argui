use argui_paint::{DisplayList, ImageAsset, VectorAsset};
use argui_text::{PreparedText, TextEngine};
use std::{collections::HashMap, mem::size_of, sync::Arc};
use wgpu::{CurrentSurfaceTexture, SurfaceTarget, TextureFormat, TextureViewDescriptor};

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

mod api;
mod blur;
mod composite;
mod effect_damage;
mod effect_registry;
mod effects;
mod filter_shadow;
mod offscreen_api;
mod retained;

mod configure;
mod device;
use configure::{drawable_size, srgb_target, surface_alpha_mode};
use device::next_device_generation;
use offscreen_api::offscreen_texture;
use retained::full_damage_profile;

enum FrameContent<'a> {
    None,
    Text {
        engine: &'a mut TextEngine,
        text: &'a PreparedText,
    },
    Ui {
        engine: &'a mut TextEngine,
        text: &'a PreparedText,
        display_list: &'a DisplayList,
        scale_factor: f32,
    },
    Composite {
        display_list: &'a DisplayList,
        scale_factor: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderStatus {
    Presented,
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

    /// Reuses `device_handle` and `instance` for a visible `surface` of the
    /// requested pixel size, applying `renderer_config` to its render resources.
    /// Returns a renderer or a surface/resource configuration error.
    fn from_existing_device(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        device_handle: RendererDevice,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError> {
        Self::finish_new(
            instance,
            Some(surface),
            device_handle.0.adapter.clone(),
            device_handle.0.device.clone(),
            device_handle.0.queue.clone(),
            device_handle,
            width,
            height,
            renderer_config,
        )
    }

    /// Creates shared render resources from `instance`, `adapter`, `device`,
    /// `queue`, and `device_handle`. `surface` selects visible or offscreen
    /// output; `width`, `height`, and `renderer_config` set target properties.
    /// Returns a renderer or a device, surface, or resource configuration error.
    #[allow(clippy::too_many_arguments)]
    fn finish_new(
        instance: wgpu::Instance,
        surface: Option<wgpu::Surface<'static>>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        device_handle: RendererDevice,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError> {
        renderer_config
            .gpu_canvases
            .validate_device(device.features(), &device.limits())?;
        let mut surface_config = if let Some(surface) = &surface {
            let mut config = surface
                .get_default_config(&adapter, width.max(1), height.max(1))
                .ok_or(RendererError::UnsupportedSurface)?;
            config.present_mode = renderer_config.present_mode;
            config.desired_maximum_frame_latency = renderer_config.maximum_frame_latency;
            config.alpha_mode = surface_alpha_mode(
                renderer_config.surface_alpha,
                &surface.get_capabilities(&adapter).alpha_modes,
            )?;
            config
        } else {
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: TextureFormat::Rgba8Unorm,
                color_space: wgpu::SurfaceColorSpace::Auto,
                width: width.max(1),
                height: height.max(1),
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: Vec::new(),
            }
        };
        let target_format = srgb_target(surface_config.format);
        if target_format != surface_config.format {
            surface_config.view_formats.push(target_format);
        }
        if let Some(surface) = &surface {
            surface.configure(&device, &surface_config);
        }
        let offscreen_target = surface
            .is_none()
            .then(|| offscreen_texture(&device, surface_config.width, surface_config.height));
        let quad = QuadGpu::new(
            &device,
            target_format,
            renderer_config.gradient_stop_capacity,
        );
        let text = TextGpu::new(&device, target_format);
        let image = ImageGpu::new(&device, target_format, renderer_config.image_cache_bytes);
        let vector = VectorGpu::new(&device, target_format);
        let gpu_canvas = CanvasGpu::new(
            &device,
            &queue,
            target_format,
            device_handle.0.generation,
            renderer_config.gpu_canvases.clone(),
            renderer_config.gpu_canvas_cache_bytes,
        );
        let maximum_parameter_words = renderer_config
            .effects
            .definitions()
            .iter()
            .map(crate::EffectDefinition::parameter_words)
            .max()
            .unwrap_or(1);
        let maximum_storage_bytes = device.limits().max_storage_buffer_binding_size as usize;
        let parameter_bytes = maximum_parameter_words * size_of::<u32>();
        if parameter_bytes > maximum_storage_bytes {
            return Err(RendererError::EffectParametersTooLarge {
                provided: parameter_bytes,
                maximum: maximum_storage_bytes,
            });
        }
        let effect = EffectGpu::new(&device, target_format, maximum_parameter_words);
        let damage = DamageGpu::new(&device, target_format);
        let offscreen = TexturePool::new(target_format, 32 * 1024 * 1024);
        let gpu_profiler = GpuProfiler::new(&adapter, &device, &queue, renderer_config.profiling);

        let profiling_active = renderer_config.profiling;
        Ok(Self {
            device_handle,
            instance,
            surface,
            offscreen_target,
            device,
            queue,
            surface_config,
            target_format,
            renderer_config,
            batches: Vec::new(),
            text_ranges: Vec::new(),
            text_bounds: Vec::new(),
            quad,
            text,
            image,
            vector,
            gpu_canvas,
            effect,
            offscreen,
            gpu_profiler,
            last_profile: RenderProfile::default(),
            profiling_active,
            layer_cache: HashMap::new(),
            content_revision: 0,
            damage,
            scene_snapshot: None,
            effect_root: None,
        })
    }

    /// Renders `content` to the configured target, calls `notify` after work is
    /// queued, and returns presentation status or a renderer error.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn render_frame(
        &mut self,
        content: FrameContent<'_>,
        notify: impl FnOnce(),
    ) -> Result<RenderStatus, RendererError> {
        let (frame, texture, status) = if let Some(surface) = &self.surface {
            match surface.get_current_texture() {
                CurrentSurfaceTexture::Success(frame) => {
                    let texture = frame.texture.clone();
                    (Some(frame), texture, RenderStatus::Presented)
                }
                CurrentSurfaceTexture::Suboptimal(frame) => {
                    let texture = frame.texture.clone();
                    (Some(frame), texture, RenderStatus::Reconfigure)
                }
                CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                    return Ok(RenderStatus::Skipped);
                }
                CurrentSurfaceTexture::Outdated => return Ok(RenderStatus::Reconfigure),
                CurrentSurfaceTexture::Lost => return Ok(RenderStatus::RecreateSurface),
                CurrentSurfaceTexture::Validation => return Err(RendererError::Validation),
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
        let profiler = FrameProfiler::start(self.profiling_active);
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
        let view = texture.create_view(&TextureViewDescriptor {
            format: Some(self.target_format),
            ..Default::default()
        });
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
            notify();
            if let Some(capture) = gpu_capture {
                capture.finish(&mut encoder);
            }
            canvas_commands.push(encoder.finish());
            self.queue.submit(canvas_commands);
            self.poll_submitted_gpu_work();
            self.finish_profile(profiler, viewport, graph_stats, damage_profile, false);
            if let Some(frame) = frame {
                self.queue.present(frame);
            }
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
        notify();
        if let Some(capture) = gpu_capture {
            capture.finish(&mut encoder);
        }
        canvas_commands.push(encoder.finish());
        self.queue.submit(canvas_commands);
        self.poll_submitted_gpu_work();
        self.finish_profile(
            profiler,
            viewport,
            graph_stats,
            damage_profile,
            direct_surface,
        );
        if let Some(frame) = frame {
            self.queue.present(frame);
        }
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
