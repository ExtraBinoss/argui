use super::PaintContext;
use crate::{LayoutNode, LayoutOutput};
use argui_ui::{Element, HitRegion, PointerEvents};

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

pub(super) fn push_hit_region(
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let own_allowed = context.hit_allowed
        && !matches!(
            element.hit_test.pointer_events,
            PointerEvents::None | PointerEvents::ContentsOnly
        );
    let listens_to_pointer = element
        .event_listeners
        .iter()
        .any(|listener| listener.event.requires_hit_test());
    if own_allowed && (element.interaction.is_some() || listens_to_pointer) {
        let interaction = element.interaction.clone().unwrap_or_default();
        output.hit_regions.push(HitRegion {
            node: node.node,
            bounds: node.bounds,
            transform: context.transform,
            clips: context.clips.clone(),
            shape: element.hit_test.shape,
            slop: element.hit_test.slop,
            enabled: interaction.enabled,
            focus_policy: if interaction.enabled {
                interaction.focus_policy
            } else {
                argui_ui::FocusPolicy::None
            },
            cursor: interaction.cursor,
            gestures: interaction.gestures,
            window_drag: interaction.window_drag,
        });
    }
}
