use argui_core::Rect;
use argui_paint::{RenderObjectId, Shadow};

use crate::{effect::uniform, gpu_profile::GpuFrameCapture, target::TextureTarget};

use super::{SurfaceRenderer, effects::EffectSources};

impl SurfaceRenderer {
    /// Applies a CSS-style shadow to the current filtered image and composites the image above it.
    ///
    /// `encoder` records the passes; `current` is the ordered filter input; `shadow` supplies
    /// offset, blur and color; `viewport` and `bounds` locate the image; `profiler` and `object`
    /// identify the operation in GPU diagnostics. Returns the resulting texture.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_filter_shadow(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        current: TextureTarget,
        shadow: Shadow,
        viewport: [f32; 2],
        bounds: Rect,
        profiler: Option<&GpuFrameCapture>,
        object: Option<RenderObjectId>,
    ) -> TextureTarget {
        let blurred = self.apply_blur(
            encoder,
            current,
            shadow.blur,
            viewport,
            bounds,
            profiler,
            object,
        );
        let empty = self.acquire_target(current.region, current.region.size);
        self.clear_target(encoder, empty, wgpu::Color::TRANSPARENT);
        let shadowed = self.acquire_target(current.region, current.region.size);
        self.clear_target(encoder, shadowed, wgpu::Color::TRANSPARENT);
        let mut params = uniform(viewport, shadowed.region, blurred, empty, bounds);
        params.mode = 10;
        params.color = shadow.color.to_linear_rgba();
        params.data = [1.0, shadow.offset[0], shadow.offset[1], 0.0];
        params.radii = [-1.0; 4];
        self.draw_effect(
            encoder,
            shadowed,
            shadowed.region,
            EffectSources {
                source: blurred,
                backdrop: empty,
            },
            params,
            None,
            &[],
            profiler,
            "filter.drop-shadow",
            object,
        );

        let combined = self.acquire_target(current.region, current.region.size);
        self.clear_target(encoder, combined, wgpu::Color::TRANSPARENT);
        let mut params = uniform(viewport, combined.region, current, shadowed, bounds);
        params.radii = [-1.0; 4];
        self.draw_effect(
            encoder,
            combined,
            combined.region,
            EffectSources {
                source: current,
                backdrop: shadowed,
            },
            params,
            None,
            &[],
            profiler,
            "filter.drop-shadow-composite",
            object,
        );
        combined
    }
}
