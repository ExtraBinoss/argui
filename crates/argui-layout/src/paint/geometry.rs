use super::PaintContext;
use crate::{LayoutNode, LayoutOutput};

/// Accessible bounds are surface-space rectangles, not untransformed layout
/// boxes. Clips are conservatively represented by their transformed bounds.
pub(super) fn record(node: LayoutNode, context: &PaintContext, output: &mut LayoutOutput) {
    let mut bounds = context.transform.transform_rect(node.bounds);
    for clip in context.clips.regions() {
        let clip = clip.transform.transform_rect(clip.bounds);
        bounds = bounds.intersection(clip).unwrap_or_default();
    }
    output.semantic_bounds.push((node.node, bounds));
}
