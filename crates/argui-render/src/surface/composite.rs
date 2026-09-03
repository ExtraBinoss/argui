use argui_paint::{LayerMask, LayerStyle};

use crate::{
    effect::{blend_mode, layer_radii, uniform},
    gpu_profile::GpuFrameCapture,
    target::{PixelRegion, TextureTarget},
};

use super::{SurfaceRenderer, effects::EffectSources};

#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::too_many_arguments)]
impl SurfaceRenderer {
    pub(super) fn composite_layer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: TextureTarget,
        foreground: TextureTarget,
        style: &LayerStyle,
        viewport: [f32; 2],
        region: PixelRegion,
        profiler: Option<&GpuFrameCapture>,
    ) {
        let snapshot = self.snapshot(encoder, target, region);
        let filtered = self.apply_filters(
            encoder,
            snapshot,
            &style.backdrop_filters,
            viewport,
            style.bounds,
            style.mask,
            profiler,
            style.profile,
        );
        let mut backdrop = if style.backdrop_filters.is_empty() {
            snapshot
        } else {
            let merged = self.acquire_target(region, region.size);
            self.clear_target(encoder, merged, wgpu::Color::TRANSPARENT);
            let mut params = uniform(viewport, region, filtered, snapshot, style.bounds);
            params.mode = 12;
            params.data[0] = style.opacity.clamp(0.0, 1.0);
            params.radii = layer_radii(style.mask);
            self.draw_effect(
                encoder,
                merged,
                region,
                EffectSources {
                    source: filtered,
                    backdrop: snapshot,
                },
                params,
                None,
                &[],
                profiler,
                "composite.backdrop",
                style.profile,
            );
            merged
        };
        for shadow in &style.shadows {
            let blurred = self.apply_blur(
                encoder,
                foreground,
                shadow.blur,
                viewport,
                style.expanded_bounds(),
                profiler,
                style.profile,
            );
            let shadowed = self.acquire_target(region, region.size);
            self.clear_target(encoder, shadowed, wgpu::Color::TRANSPARENT);
            let mut params = uniform(viewport, region, blurred, backdrop, style.bounds);
            params.mode = if shadow.inset { 11 } else { 10 };
            params.color = shadow.color.to_linear_rgba();
            params.color[3] *= style.opacity.clamp(0.0, 1.0);
            params.data = [1.0, shadow.offset[0], shadow.offset[1], shadow.spread];
            params.radii = layer_radii(style.mask);
            self.draw_effect(
                encoder,
                shadowed,
                region,
                EffectSources {
                    source: blurred,
                    backdrop,
                },
                params,
                None,
                &[],
                profiler,
                "composite.shadow",
                style.profile,
            );
            backdrop = shadowed;
        }
        let expansion = style.foreground_expansion();
        let mut params = uniform(
            viewport,
            region,
            foreground,
            backdrop,
            style.foreground_bounds(),
        );
        params.blend = blend_mode(style.blend_mode);
        params.data[0] = style.opacity;
        params.radii = expanded_radii(style.mask, expansion);
        self.draw_effect(
            encoder,
            target,
            region,
            EffectSources {
                source: foreground,
                backdrop,
            },
            params,
            None,
            &[],
            profiler,
            "composite.layer",
            style.profile,
        );
    }

    fn snapshot(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: TextureTarget,
        region: PixelRegion,
    ) -> TextureTarget {
        let target = self.acquire_target(region, region.size);
        let mut source_copy = self.offscreen.texture(source.texture).as_image_copy();
        source_copy.origin = wgpu::Origin3d {
            x: region.origin[0] - source.region.origin[0],
            y: region.origin[1] - source.region.origin[1],
            z: 0,
        };
        encoder.copy_texture_to_texture(
            source_copy,
            self.offscreen.texture(target.texture).as_image_copy(),
            wgpu::Extent3d {
                width: region.size[0],
                height: region.size[1],
                depth_or_array_layers: 1,
            },
        );
        target
    }
}

fn expanded_radii(mask: LayerMask, expansion: f32) -> [f32; 4] {
    match mask {
        LayerMask::Rounded(radii) => radii.as_array().map(|radius| radius + expansion),
        LayerMask::None | LayerMask::Bounds => [0.0; 4],
    }
}

#[cfg(test)]
mod tests {
    use argui_paint::{CornerRadii, LayerMask};

    use super::expanded_radii;

    #[test]
    fn expansion_only_changes_rounded_masks() {
        assert_eq!(expanded_radii(LayerMask::None, 5.0), [0.0; 4]);
        assert_eq!(expanded_radii(LayerMask::Bounds, 5.0), [0.0; 4]);
        assert_eq!(
            expanded_radii(LayerMask::Rounded(CornerRadii::all(4.0)), 5.0),
            [9.0; 4]
        );
    }
}
