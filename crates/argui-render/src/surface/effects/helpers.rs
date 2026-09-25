use wgpu::{LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};

use argui_paint::LayerStyle;

use crate::{
    batch::DrawKind,
    effect_graph::{EffectLayer, EffectNode},
    target::PixelRegion,
};

/// Returns the bit mask of primitive pipelines used by `nodes`.
///
/// Layer nodes do not draw in the current pass. The return value uses bits
/// 0–4 for quad, text, image, vector, and canvas draws respectively.
pub(super) fn draw_kind_mask(nodes: &[EffectNode]) -> u8 {
    nodes.iter().fold(0, |mask, node| {
        mask | match node {
            EffectNode::Layer(_) => 0,
            EffectNode::Draw(batch) => match batch.kind {
                DrawKind::Quad => 1,
                DrawKind::Text => 2,
                DrawKind::Image(..) => 4,
                DrawKind::Vector => 8,
                DrawKind::GpuCanvas(..) => 16,
            },
        }
    })
}

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

/// Returns the profiler label for built-in effect `mode`.
pub(super) const fn built_in_label(mode: u32) -> &'static str {
    match mode {
        1 => "effect.blur-horizontal",
        2 => "effect.blur-vertical",
        3 => "effect.blur-dual-downsample",
        4 => "effect.blur-dual-upsample",
        8 => "effect.color-matrix",
        9 => "effect.refraction",
        10 => "composite.drop-shadow",
        11 => "composite.inset-shadow",
        12 => "composite.backdrop",
        _ => "composite.copy",
    }
}

/// Resolves a layer's output to its exact ancestor clip inside `target`.
///
/// `style` supplies transformed layer bounds and the optional scroll clip;
/// `target` bounds the render attachment. Returns `None` when no pixels remain.
pub(super) fn clipped_output_region(
    style: &LayerStyle,
    target: PixelRegion,
) -> Option<PixelRegion> {
    PixelRegion::from_rect(style.transformed_bounds(), target).and_then(|output| {
        style.clip.map_or(Some(output), |clip| {
            PixelRegion::from_clip_rect(clip, target).and_then(|clip| output.intersection(clip))
        })
    })
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
        || clipped_output_region(&layer.style, target)
            .and_then(|region| clip.map_or(Some(region), |clip| region.intersection(clip)))
            .is_none()
}
