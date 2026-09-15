use argui_paint::{DisplayList, ImageAsset, VectorAsset};
use argui_text::{PreparedText, TextEngine};
use std::{collections::HashMap, mem::size_of, sync::Arc};
use wgpu::{
    CurrentSurfaceTexture, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, SurfaceTarget, TextureFormat, TextureViewDescriptor,
};

use crate::{
    EffectGraphStats, RenderProfile, RendererConfig, RendererError,
    batch::{DrawBatch, DrawKind, build_batches},
    effect::EffectGpu,
    effect_graph::EffectGraph,
    gpu_profile::GpuProfiler,
    image::ImageGpu,
    offscreen::{TexturePool, TexturePoolStats},
    profile::FrameProfiler,
    quad::QuadGpu,
    target::PixelRegion,
    text::TextGpu,
    vector::VectorGpu,
};

mod api;
mod composite;
mod effects;

mod configure;
use configure::{drawable_size, srgb_target, surface_alpha_mode};

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
}

pub struct SurfaceRenderer {
    device_handle: RendererDevice,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    target_format: TextureFormat,
    renderer_config: RendererConfig,
    batches: Vec<DrawBatch>,
    quad: QuadGpu,
    text: TextGpu,
    image: ImageGpu,
    vector: VectorGpu,
    effect: EffectGpu,
    offscreen: TexturePool,
    gpu_profiler: GpuProfiler,
    last_profile: RenderProfile,
    profiling_active: bool,
    layer_cache: HashMap<argui_paint::RenderObjectId, effects::CachedLayer>,
    content_revision: u64,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
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
            surface,
            device_handle.0.adapter.clone(),
            device_handle.0.device.clone(),
            device_handle.0.queue.clone(),
            device_handle,
            width,
            height,
            renderer_config,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_new(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        device_handle: RendererDevice,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError> {
        let mut surface_config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .ok_or(RendererError::UnsupportedSurface)?;
        surface_config.present_mode = renderer_config.present_mode;
        surface_config.desired_maximum_frame_latency = renderer_config.maximum_frame_latency;
        surface_config.alpha_mode = surface_alpha_mode(
            renderer_config.surface_alpha,
            &surface.get_capabilities(&adapter).alpha_modes,
        )?;
        let target_format = srgb_target(surface_config.format);
        if target_format != surface_config.format {
            surface_config.view_formats.push(target_format);
        }
        surface.configure(&device, &surface_config);
        let quad = QuadGpu::new(
            &device,
            target_format,
            renderer_config.gradient_stop_capacity,
        );
        let text = TextGpu::new(&device, target_format);
        let image = ImageGpu::new(&device, target_format, renderer_config.image_cache_bytes);
        let vector = VectorGpu::new(&device, target_format);
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
        let offscreen = TexturePool::new(target_format, 128 * 1024 * 1024);
        let gpu_profiler = GpuProfiler::new(&adapter, &device, &queue, renderer_config.profiling);

        let profiling_active = renderer_config.profiling;
        Ok(Self {
            device_handle,
            instance,
            surface,
            device,
            queue,
            surface_config,
            target_format,
            renderer_config,
            batches: Vec::new(),
            quad,
            text,
            image,
            vector,
            effect,
            offscreen,
            gpu_profiler,
            last_profile: RenderProfile::default(),
            profiling_active,
            layer_cache: HashMap::new(),
            content_revision: 0,
        })
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn render_frame(
        &mut self,
        content: FrameContent<'_>,
        notify: impl FnOnce(),
    ) -> Result<RenderStatus, RendererError> {
        let (frame, status) = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => (frame, RenderStatus::Presented),
            CurrentSurfaceTexture::Suboptimal(frame) => (frame, RenderStatus::Reconfigure),
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                return Ok(RenderStatus::Skipped);
            }
            CurrentSurfaceTexture::Outdated => return Ok(RenderStatus::Reconfigure),
            CurrentSurfaceTexture::Lost => return Ok(RenderStatus::RecreateSurface),
            CurrentSurfaceTexture::Validation => return Err(RendererError::Validation),
        };

        let viewport = [
            self.surface_config.width as f32,
            self.surface_config.height as f32,
        ];
        let profiler = FrameProfiler::start(self.profiling_active);
        let gpu_capture = self.gpu_profiler.begin_frame(self.profiling_active);
        let mut graph_stats = EffectGraphStats::default();
        let mut effect_graph = None;
        match content {
            FrameContent::None => {
                self.vector.clear_frame_stats();
                self.batches.clear();
            }
            FrameContent::Text { engine, text } => {
                self.vector.clear_frame_stats();
                let draw = self.text.prepare(&self.device, &self.queue, engine, text)?;
                let range = draw.all();
                self.batches.clear();
                if !range.is_empty() {
                    self.batches.push(DrawBatch {
                        kind: DrawKind::Text,
                        instances: range,
                    });
                }
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
                let draw = self.text.prepare_ui(
                    &self.device,
                    &self.queue,
                    engine,
                    text,
                    display_list,
                    scale_factor,
                )?;
                if quad_changed || image_changed || vector_changed || draw.changed() {
                    self.content_revision = self.content_revision.wrapping_add(1);
                }
                build_batches(display_list, draw.ranges(), &mut self.batches);
                let graph = EffectGraph::build(
                    display_list,
                    draw.ranges(),
                    viewport,
                    scale_factor,
                    self.content_revision,
                )
                .map_err(|error| RendererError::InvalidDisplayList(error.to_string()))?;
                let additional_effect_passes = self.validate_custom_effects(&graph)?;
                graph_stats = graph.stats();
                graph_stats.filter_passes += additional_effect_passes;
                effect_graph = Some(graph);
            }
        }
        let view = frame.texture.create_view(&TextureViewDescriptor {
            format: Some(self.target_format),
            ..Default::default()
        });
        let attachment = Some(RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(self.renderer_config.wgpu_clear_color()),
                store: StoreOp::Store,
            },
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.quad.begin_frame();
        self.text.begin_frame();
        self.image.begin_frame();
        self.vector.begin_frame();
        if let Some(graph) = effect_graph
            && graph.needs_offscreen_root()
        {
            let (cached_layers, damaged_pixels) = self.render_effect_graph(
                &mut encoder,
                &view,
                &graph,
                viewport,
                gpu_capture.as_ref(),
            );
            graph_stats.cached_layers = cached_layers;
            graph_stats.damaged_pixels = damaged_pixels;
            notify();
            if let Some(capture) = gpu_capture {
                capture.finish(&mut encoder);
            }
            self.queue.submit([encoder.finish()]);
            let _ = self.device.poll(wgpu::PollType::Poll);
            self.finish_profile(profiler, viewport, graph_stats);
            self.queue.present(frame);
            return Ok(status);
        }
        let target = PixelRegion::viewport(viewport[0] as u32, viewport[1] as u32);
        let quad_offset = self.quad.target_offset(&self.queue, target.as_f32());
        let text_offset = self.text.target_offset(&self.queue, target.as_f32());
        let image_offset = self.image.target_offset(&self.queue, target.as_f32());
        let vector_offset = self.vector.target_offset(&self.queue, target.as_f32());
        {
            let timestamp_writes = gpu_capture.as_ref().and_then(|capture| {
                capture.timestamp_writes(
                    "surface.main",
                    None,
                    u64::from(target.size[0]) * u64::from(target.size[1]),
                )
            });
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("argui-clear-pass"),
                color_attachments: &[attachment],
                timestamp_writes,
                ..Default::default()
            });
            for batch in &self.batches {
                match batch.kind {
                    DrawKind::Quad => {
                        self.quad
                            .draw(&mut pass, batch.instances.clone(), quad_offset);
                    }
                    DrawKind::Text => {
                        self.text
                            .draw(&mut pass, batch.instances.clone(), text_offset);
                    }
                    DrawKind::Image(image, sampling) => self.image.draw(
                        &mut pass,
                        image,
                        sampling,
                        batch.instances.clone(),
                        image_offset,
                    ),
                    DrawKind::Vector => {
                        self.vector
                            .draw(&mut pass, batch.instances.clone(), vector_offset)
                    }
                }
            }
        }
        notify();
        if let Some(capture) = gpu_capture {
            capture.finish(&mut encoder);
        }
        self.queue.submit([encoder.finish()]);
        let _ = self.device.poll(wgpu::PollType::Poll);
        self.finish_profile(profiler, viewport, graph_stats);
        self.queue.present(frame);
        Ok(status)
    }

    fn finish_profile(
        &mut self,
        profiler: FrameProfiler,
        viewport: [f32; 2],
        effects: EffectGraphStats,
    ) {
        if let Some(profile) = profiler.finish(RenderProfile {
            viewport_pixels: viewport[0] as u64 * viewport[1] as u64,
            draw_batches: self.batches.len(),
            effects,
            texture_pool: self.offscreen.stats(),
            vector_atlas: self.vector.stats(),
            direct_surface: effects.offscreen_layers == 0,
            adapter: self.gpu_profiler.adapter().clone(),
            gpu: self.gpu_profiler.take_latest(),
            ..RenderProfile::default()
        }) {
            self.last_profile = profile;
        }
    }
}
