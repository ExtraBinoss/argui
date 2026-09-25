//! Blur render passes over the effect pipeline's pooled, filterable textures.
//!
//! Fragment passes reuse the existing WebGPU/WebGL targets and bilinear sampler;
//! compute would require storage textures and another backend path.

use argui_core::Rect;
use argui_paint::RenderObjectId;

use crate::{config::BlurAlgorithm, gpu_profile::GpuFrameCapture, target::TextureTarget};

use super::{SurfaceRenderer, effects::EffectPass};

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Blurs `current` with the configured Gaussian or dual-filter render passes.
    ///
    /// `encoder` records the GPU work; `radius` is the physical blur radius;
    /// `viewport` and `bounds` locate the layer; `profiler` and `object` attribute
    /// passes in a frame capture. Returns the filtered texture, or `current`
    /// when the radius is effectively zero.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_blur(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        current: TextureTarget,
        radius: f32,
        viewport: [f32; 2],
        bounds: Rect,
        profiler: Option<&GpuFrameCapture>,
        object: Option<RenderObjectId>,
    ) -> TextureTarget {
        if radius <= 0.01 {
            return current;
        }
        match self
            .renderer_config
            .blur_algorithm
            .resolve(radius, current.extent)
        {
            BlurAlgorithm::Gaussian => {
                self.gaussian_blur(encoder, current, radius, viewport, bounds, profiler, object)
            }
            BlurAlgorithm::DualKawase => {
                self.dual_filter_blur(encoder, current, radius, viewport, bounds, profiler, object)
            }
            BlurAlgorithm::Auto => unreachable!("automatic blur must resolve to a filter"),
        }
    }

    /// Runs the separable Gaussian passes, preserving the configured quality downsample.
    ///
    /// `encoder` records GPU commands, `current` provides input pixels, and
    /// `radius` sets the physical blur width. `viewport` and `bounds` locate
    /// those pixels; `profiler` and `object` label GPU timing. Returns a
    /// filtered texture at the input extent.
    #[allow(clippy::too_many_arguments)]
    fn gaussian_blur(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        mut current: TextureTarget,
        radius: f32,
        viewport: [f32; 2],
        bounds: Rect,
        profiler: Option<&GpuFrameCapture>,
        object: Option<RenderObjectId>,
    ) -> TextureTarget {
        let base = if radius >= 12.0 {
            4
        } else if radius >= 6.0 {
            2
        } else {
            1
        };
        let downsample = base
            * self
                .renderer_config
                .effect_quality
                .settings()
                .blur_downsample_bias
                .max(1);
        if downsample > 1 {
            let mut pass = EffectPass::new(99, [0.0; 4], bounds);
            pass.extent = Some([
                (current.region.size[0] / downsample).max(1),
                (current.region.size[1] / downsample).max(1),
            ]);
            pass.object = object;
            current = self.effect_pass(encoder, current, viewport, pass, profiler);
        }
        let sigma = radius / downsample as f32 * 0.35;
        for mode in [1, 2] {
            let mut pass = EffectPass::new(mode, [sigma, 0.0, 0.0, 0.0], bounds);
            pass.extent = Some(current.extent);
            pass.object = object;
            current = self.effect_pass(encoder, current, viewport, pass, profiler);
        }
        if downsample > 1 {
            let mut pass = EffectPass::new(99, [0.0; 4], bounds);
            pass.object = object;
            current = self.effect_pass(encoder, current, viewport, pass, profiler);
        }
        current
    }

    /// Runs the five-tap downsample and eight-tap upsample pyramid.
    ///
    /// `encoder` records GPU commands, `current` provides input pixels, and
    /// `radius` sets the physical blur width. `viewport` and `bounds` locate
    /// those pixels; `profiler` and `object` label GPU timing. Returns a
    /// filtered texture at the input extent.
    #[allow(clippy::too_many_arguments)]
    fn dual_filter_blur(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        mut current: TextureTarget,
        radius: f32,
        viewport: [f32; 2],
        bounds: Rect,
        profiler: Option<&GpuFrameCapture>,
        object: Option<RenderObjectId>,
    ) -> TextureTarget {
        let full_extent = current.extent;
        let desired_levels = ((radius / 3.0).max(1.0).log2().ceil() as usize).clamp(1, 8);
        let offset =
            (radius / (3.0 * (1 << desired_levels.saturating_sub(1)) as f32)).clamp(0.75, 2.0);
        let mut extents = Vec::with_capacity(desired_levels);
        for _ in 0..desired_levels {
            let next = [
                (current.extent[0] / 2).max(1),
                (current.extent[1] / 2).max(1),
            ];
            if next == current.extent {
                break;
            }
            extents.push(current.extent);
            let mut pass = EffectPass::new(3, [offset, 0.0, 0.0, 0.0], bounds);
            pass.extent = Some(next);
            pass.object = object;
            current = self.effect_pass(encoder, current, viewport, pass, profiler);
        }
        for extent in extents.into_iter().rev() {
            let mut pass = EffectPass::new(4, [offset, 0.0, 0.0, 0.0], bounds);
            pass.extent = Some(extent);
            pass.object = object;
            current = self.effect_pass(encoder, current, viewport, pass, profiler);
        }
        debug_assert_eq!(current.extent, full_extent);
        current
    }
}
