use wgpu::{LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};

use crate::{effect_graph::EffectLayer, target::PixelRegion};

/// Returns whether `cached` and `current` have identical rendered foregrounds.
/// Composition-only style changes can reuse the cached foreground.
pub(super) fn same_layer_content(
    cached: &crate::effect_graph::EffectLayer,
    current: &crate::effect_graph::EffectLayer,
) -> bool {
    cached.region == current.region
        && cached.content_revision == current.content_revision
        && cached.children == current.children
        && cached.style.bounds == current.style.bounds
        && cached.style.filters == current.style.filters
        && cached.style.mask == current.style.mask
}

/// Clears `view` to `color` by recording a render pass in `encoder`.
pub(super) fn clear_view(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    color: wgpu::Color,
) {
    let attachment = Some(RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(color),
            store: StoreOp::Store,
        },
    });
    drop(encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("argui-effect-clear"),
        color_attachments: &[attachment],
        ..Default::default()
    }));
}

/// Returns a blur downsample divisor selected from `radius` and quality `bias`.
pub(super) fn blur_downsample(radius: f32, bias: u32) -> u32 {
    let base = if radius >= 12.0 {
        4
    } else if radius >= 6.0 {
        2
    } else {
        1
    };
    base * bias.max(1)
}

/// Returns the profiler label for built-in effect `mode`.
pub(super) const fn built_in_label(mode: u32) -> &'static str {
    match mode {
        1 => "effect.blur-horizontal",
        2 => "effect.blur-vertical",
        8 => "effect.color-matrix",
        9 => "effect.refraction",
        10 => "composite.drop-shadow",
        11 => "composite.inset-shadow",
        12 => "composite.backdrop",
        _ => "composite.copy",
    }
}

/// Returns whether `layer` cannot affect `target` inside the optional damage `clip`.
/// Non-offscreen layers are kept because their children can overdraw their bounds.
pub(super) fn skipped_layer(
    layer: &EffectLayer,
    target: PixelRegion,
    clip: Option<PixelRegion>,
) -> bool {
    if layer.style.opacity <= 0.0 {
        return true;
    }
    if !layer.style.requires_offscreen() {
        return false;
    }
    layer.region.is_none()
        || PixelRegion::from_rect(layer.style.transformed_bounds(), target)
            .and_then(|region| clip.map_or(Some(region), |clip| region.intersection(clip)))
            .is_none()
}
