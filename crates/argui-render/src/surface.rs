use argui_paint::DisplayList;
use argui_text::{PreparedText, TextEngine};
use wgpu::{
    CurrentSurfaceTexture, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, SurfaceTarget,
};

use crate::{
    RendererConfig, RendererError,
    batch::{DrawBatch, DrawKind, build_batches},
    quad::QuadGpu,
    text::TextGpu,
};

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
    renderer_config: RendererConfig,
    batches: Vec<DrawBatch>,
    quad: QuadGpu,
    text: TextGpu,
}

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
        surface.configure(&device, &surface_config);
        let quad = QuadGpu::new(&device, surface_config.format);
        let text = TextGpu::new(&device, surface_config.format);

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            surface_config,
            renderer_config,
            batches: Vec::new(),
            quad,
            text,
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
        match content {
            FrameContent::None => self.batches.clear(),
            FrameContent::Text { engine, text } => {
                let draw = self
                    .text
                    .prepare(&self.device, &self.queue, engine, text, viewport)?;
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
                self.quad.prepare(
                    &self.device,
                    &self.queue,
                    display_list,
                    viewport,
                    scale_factor,
                );
                let draw = self
                    .text
                    .prepare(&self.device, &self.queue, engine, text, viewport)?;
                build_batches(display_list, draw.ranges(), &mut self.batches);
            }
        }
        let view = frame.texture.create_view(&Default::default());
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
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("argui-clear-pass"),
                color_attachments: &[attachment],
                ..Default::default()
            });
            for batch in &self.batches {
                match batch.kind {
                    DrawKind::Quad => self.quad.draw(&mut pass, batch.instances.clone()),
                    DrawKind::Text => self.text.draw(&mut pass, batch.instances.clone()),
                }
            }
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        Ok(status)
    }
}

const fn drawable_size(width: u32, height: u32) -> Option<(u32, u32)> {
    if width == 0 || height == 0 {
        None
    } else {
        Some((width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::drawable_size;

    #[test]
    fn zero_sized_surfaces_are_not_configured() {
        assert_eq!(drawable_size(800, 600), Some((800, 600)));
        assert_eq!(drawable_size(0, 600), None);
        assert_eq!(drawable_size(800, 0), None);
    }
}
