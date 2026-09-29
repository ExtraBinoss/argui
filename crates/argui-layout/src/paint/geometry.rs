use super::PaintContext;
use crate::{LayoutNode, LayoutOutput};
use argui_core::{Affine2D, Point, TransformOrigin};
use argui_ui::{Element, FocusTarget, HitRegion, PointerEvents, PortalTarget, UiTree};

/// Keeps anchored popup entrances attached to their trigger after collision placement.
pub(crate) fn local_transform(
    ui: &UiTree,
    output: &LayoutOutput,
    node: LayoutNode,
    element: &Element,
) -> Affine2D {
    let mut transform = ui.resolved_transform(node.node, element);
    if transform == argui_core::Transform2D::IDENTITY {
        return Affine2D::IDENTITY;
    }
    let mut origin = element.transform_origin;
    if let Some(portal) = &element.portal
        && portal.transform_from_anchor
        && let PortalTarget::Anchor(anchor) = &portal.target
        && let Some(anchor_id) = ui.resolve_node(&FocusTarget::Key(anchor.key.clone()))
        && let Some(anchor_node) = output.nodes.iter().find(|node| node.node == anchor_id)
        && node.bounds.size.width > 0.0
        && node.bounds.size.height > 0.0
    {
        let center = Point::new(
            anchor_node.bounds.origin.x + anchor_node.bounds.size.width / 2.0,
            anchor_node.bounds.origin.y + anchor_node.bounds.size.height / 2.0,
        );
        origin = TransformOrigin::new(
            (center.x - node.bounds.origin.x) / node.bounds.size.width,
            (center.y - node.bounds.origin.y) / node.bounds.size.height,
        );
        if node.bounds.origin.y + node.bounds.size.height / 2.0 < center.y {
            transform.translation.y = -transform.translation.y;
        }
    }
    transform.affine(node.bounds, origin)
}

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
