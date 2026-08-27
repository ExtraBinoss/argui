use argui_paint::{DisplayList, ImageAsset, ShaderEffectId, VectorAsset};
use argui_text::{PreparedText, TextEngine};
use wgpu::{
    CurrentSurfaceTexture, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, SurfaceTarget, TextureFormat, TextureViewDescriptor,
};

use crate::{
    EffectGraphStats, RenderProfile, RendererConfig, RendererError,
    batch::{DrawBatch, DrawKind, build_batches},
    effect::EffectGpu,
    effect_graph::EffectGraph,
    image::ImageGpu,
    offscreen::{TexturePool, TexturePoolStats},
    profile::FrameProfiler,
    quad::QuadGpu,
    target::PixelRegion,
    text::TextGpu,
    vector::VectorGpu,
};

mod effects;

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

pub struct SurfaceRenderer {
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
    last_profile: RenderProfile,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn new(
        target: impl Into<SurfaceTarget<'static>>,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(target)
            .map_err(|error| RendererError::SurfaceCreation(error.to_string()))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: renderer_config.power_preference,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|error| RendererError::AdapterRequest(error.to_string()))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("argui-device"),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await
            .map_err(|error| RendererError::DeviceRequest(error.to_string()))?;
        let mut surface_config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .ok_or(RendererError::UnsupportedSurface)?;
        surface_config.present_mode = renderer_config.present_mode;
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
        let effect = EffectGpu::new(&device, target_format);
        let offscreen = TexturePool::new(target_format, 128 * 1024 * 1024);

        Ok(Self {
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
            last_profile: RenderProfile::default(),
        })
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn resize(&mut self, width: u32, height: u32) -> bool {
        let Some((width, height)) = drawable_size(width, height) else {
            return false;
        };
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        true
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn recreate_surface(
        &mut self,
        target: impl Into<SurfaceTarget<'static>>,
    ) -> Result<(), RendererError> {
        self.surface = self
            .instance
            .create_surface(target)
            .map_err(|error| RendererError::SurfaceCreation(error.to_string()))?;
        self.surface.configure(&self.device, &self.surface_config);
        Ok(())
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render(&mut self) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::None)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render_text(
        &mut self,
        engine: &mut TextEngine,
        text: &PreparedText,
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::Text { engine, text })
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render_ui(
        &mut self,
        engine: &mut TextEngine,
        text: &PreparedText,
        display_list: &DisplayList,
        scale_factor: f32,
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::Ui {
            engine,
            text,
            display_list,
            scale_factor,
        })
    }

    #[must_use]
    pub fn texture_pool_stats(&self) -> TexturePoolStats {
        self.offscreen.stats()
    }

    #[must_use]
    pub const fn last_profile(&self) -> RenderProfile {
        self.last_profile
    }

    pub fn register_effect_shader(
        &mut self,
        id: ShaderEffectId,
        wgsl: &str,
    ) -> Result<(), RendererError> {
        self.effect.register(&self.device, id, wgsl)
    }

    pub fn register_image(&mut self, asset: &ImageAsset) -> Result<(), RendererError> {
        self.image.register(&self.device, &self.queue, asset)
    }

    pub fn register_vector(&mut self, asset: &VectorAsset) {
        self.vector.register(&self.device, asset);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn render_frame(&mut self, content: FrameContent<'_>) -> Result<RenderStatus, RendererError> {
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
        let profiler = FrameProfiler::start(self.renderer_config.profiling);
        let mut graph_stats = EffectGraphStats::default();
        let mut effect_graph = None;
        match content {
            FrameContent::None => self.batches.clear(),
            FrameContent::Text { engine, text } => {
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
                self.quad
                    .prepare(&self.device, &self.queue, display_list, scale_factor)?;
                self.image
                    .prepare(&self.device, &self.queue, display_list, scale_factor)?;
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
                build_batches(display_list, draw.ranges(), &mut self.batches);
                let graph = EffectGraph::build(display_list, draw.ranges(), viewport, scale_factor)
                    .map_err(|error| RendererError::InvalidDisplayList(error.to_string()))?;
                self.validate_custom_effects(&graph)?;
                graph_stats = graph.stats();
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
            self.render_effect_graph(&mut encoder, &view, &graph, viewport);
            self.queue.submit([encoder.finish()]);
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
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("argui-clear-pass"),
                color_attachments: &[attachment],
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
                    DrawKind::Vector(vector) => {
                        self.vector
                            .draw(&mut pass, vector, batch.instances.clone(), vector_offset)
                    }
                }
            }
        }
        self.queue.submit([encoder.finish()]);
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
        if let Some(profile) = profiler.finish(
            viewport,
            self.batches.len(),
            effects,
            self.offscreen.stats(),
        ) {
            self.last_profile = profile;
        }
    }
}

const fn drawable_size(width: u32, height: u32) -> Option<(u32, u32)> {
    if width == 0 || height == 0 {
        None
    } else {
        Some((width, height))
    }
}

fn srgb_target(format: TextureFormat) -> TextureFormat {
    format.add_srgb_suffix()
}

#[cfg(test)]
mod tests {
    use super::{drawable_size, srgb_target};

    #[test]
    fn zero_sized_surfaces_are_not_configured() {
        assert_eq!(drawable_size(800, 600), Some((800, 600)));
        assert_eq!(drawable_size(0, 600), None);
        assert_eq!(drawable_size(800, 0), None);
    }

    #[test]
    fn presentation_uses_an_srgb_view_when_the_surface_has_one() {
        assert_eq!(
            srgb_target(wgpu::TextureFormat::Bgra8Unorm),
            wgpu::TextureFormat::Bgra8UnormSrgb
        );
        assert_eq!(
            srgb_target(wgpu::TextureFormat::Rgba8UnormSrgb),
            wgpu::TextureFormat::Rgba8UnormSrgb
        );
        assert_eq!(
            srgb_target(wgpu::TextureFormat::Rgba16Float),
            wgpu::TextureFormat::Rgba16Float
        );
    }
}
