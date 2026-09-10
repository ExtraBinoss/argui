use super::PaintContext;
use crate::{LayoutNode, LayoutOutput};

/// Accessible bounds are surface-space rectangles, not untransformed layout
/// boxes. Clips are conservatively represented by their transformed bounds.
pub(super) fn record(node: LayoutNode, context: &PaintContext, output: &mut LayoutOutput) {
    let bounds = context
        .transform
        .transform_rect(node.bounds)
        .intersection(context.clip_bounds)
        .unwrap_or_default();
    output.semantic_bounds.push((node.node, bounds));
}
