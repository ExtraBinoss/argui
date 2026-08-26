use wgpu::{
    CurrentSurfaceTexture, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, SurfaceTarget,
};

use crate::{RendererConfig, RendererError};

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

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            surface_config,
            renderer_config,
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
    pub fn render(&self) -> Result<RenderStatus, RendererError> {
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
            let _pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("argui-clear-pass"),
                color_attachments: &[attachment],
                ..Default::default()
            });
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
