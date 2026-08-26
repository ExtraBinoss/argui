mod pipeline;

use argui_core::Rect;
use argui_paint::{BlendMode, LayerMask};

pub(crate) use pipeline::{EffectDraw, EffectGpu, EffectUniform};

pub(crate) fn uniform(viewport: [f32; 2], bounds: Rect) -> EffectUniform {
    EffectUniform {
        viewport,
        bounds: [
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            bounds.size.height,
        ],
        data: [1.0, 0.0, 0.0, 0.0],
        ..EffectUniform::default()
    }
}

pub(crate) const fn blend_mode(mode: BlendMode) -> u32 {
    match mode {
        BlendMode::Normal => 0,
        BlendMode::Multiply => 1,
        BlendMode::Screen => 2,
        BlendMode::Overlay => 3,
        BlendMode::Darken => 4,
        BlendMode::Lighten => 5,
        BlendMode::Difference => 6,
        BlendMode::Exclusion => 7,
        BlendMode::PlusLighter => 8,
    }
}

pub(crate) const fn layer_radii(mask: LayerMask) -> [f32; 4] {
    match mask {
        LayerMask::Rounded(radii) => radii.as_array(),
        LayerMask::None | LayerMask::Bounds => [0.0; 4],
    }
}
