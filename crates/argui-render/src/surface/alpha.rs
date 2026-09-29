//! Final transfer from linear composition to premultiplied sRGB native pixels.

use super::SurfaceRenderer;
use crate::{
    effect::{EffectDraw, uniform},
    gpu_profile::GpuFrameCapture,
    target::{PixelRegion, TextureTarget},
};

impl SurfaceRenderer {
    /// Returns whether the native output requires encoded premultiplied sRGB.
    fn requires_alpha_transfer(&self) -> bool {
        self.is_transparent() && self.target_format.is_srgb()
    }

    /// Chooses an intermediate sRGB render target for transparent presentation.
    /// `texture` is the final swapchain or copyable offscreen output. Opaque
    /// surfaces retain their direct path and transparent intermediates are reused.
    pub(super) fn scene_output_view(&mut self, texture: &wgpu::Texture) -> wgpu::TextureView {
        if !self.requires_alpha_transfer() {
            return texture.create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.target_format),
                ..Default::default()
            });
        }
        let size = texture.size();
        if self
            .alpha_target
            .as_ref()
            .is_none_or(|target| target.size() != size)
        {
            self.alpha_target = Some(self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("argui-linear-alpha-root"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.target_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }));
        }
        self.alpha_target
            .as_ref()
            .expect("alpha target exists")
            .create_view(&Default::default())
    }

    /// Presents `alpha_target` to `surface` with alpha applied after sRGB encoding.
    /// Internal blending remains linear. The native compositor expects encoded
    /// premultiplied RGB; encoding already-premultiplied linear RGB makes rounded
    /// corners and fading borders too bright. `viewport` and `profiler` describe
    /// the final pass, which runs only for transparent output.
    pub(super) fn present_alpha(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        viewport: [f32; 2],
        profiler: Option<&GpuFrameCapture>,
    ) {
        let Some(texture) = self
            .alpha_target
            .as_ref()
            .filter(|_| self.requires_alpha_transfer())
        else {
            return;
        };
        let source = texture.create_view(&Default::default());
        let region = PixelRegion::viewport(viewport[0] as u32, viewport[1] as u32);
        let target = TextureTarget::new(0, region, region.size);
        let mut params = uniform(viewport, region, target, target, region.as_rect());
        params.mode = 98;
        self.effect.draw(
            &self.device,
            &self.queue,
            encoder,
            EffectDraw {
                target: surface,
                target_region: region,
                target_extent: region.size,
                output_region: region,
                source: &source,
                backdrop: &source,
                uniform: params,
                shader: None,
                parameters: &[],
                profiler,
                profile_label: "surface.alpha.present",
                profile_object: None,
            },
        );
    }
}
