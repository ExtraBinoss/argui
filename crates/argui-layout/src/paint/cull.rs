//! Conservative rejection of paint subtrees outside their visible clip.

use argui_core::{Affine2D, Point, Rect, Size, Transform2D};
use argui_ui::{Element, ElementKind};

use crate::{LayoutNode, engine::NodeMap};

use super::PaintContext;

/// Returns whether `map` and all descendants are guaranteed outside `parent`'s clip.
///
/// `elements` supplies authored portal and scroll metadata; `node` is the current
/// translated layout geometry; `element` is the retained root of this subtree.
/// Portals, nested scroll regions, transforms and effects conservatively disable
/// culling. A four-pixel margin covers ordinary glyph and border overdraw.
pub(super) fn outside_visible_clip(
    map: &NodeMap,
    elements: &[&Element],
    node: LayoutNode,
    element: &Element,
    parent: &PaintContext,
) -> bool {
    let clips_descendants = map.style.overflow.x.clips() && map.style.overflow.y.clips();
    if !(map.children.is_empty() || clips_descendants)
        || map.style.overflow.x.scrolls()
        || map.style.overflow.y.scrolls()
        || parent.transform != Affine2D::IDENTITY
        || element.transform != Transform2D::IDENTITY
        || element.portal.is_some()
        || element.layer.is_some()
        || element.desktop_backdrop.is_some()
        || !element.effects.is_empty()
        || !element.bindings.is_empty()
        || element.has_state_animation()
        || matches!(
            element.kind,
            ElementKind::Custom(_) | ElementKind::GpuCanvas(_)
        )
        || (clips_descendants && has_escaping_descendant(map, elements))
    {
        return false;
    }
    let margin = 4.0;
    let bounds = Rect::new(
        Point::new(node.bounds.origin.x - margin, node.bounds.origin.y - margin),
        Size::new(
            node.bounds.size.width + margin * 2.0,
            node.bounds.size.height + margin * 2.0,
        ),
    );
    bounds.intersection(parent.clip_bounds).is_none()
}

/// Returns whether descendants could paint outside a clipped root or need scroll updates.
fn has_escaping_descendant(map: &NodeMap, elements: &[&Element]) -> bool {
    map.children.iter().any(|child| {
        let element = elements[child.index];
        element.portal.is_some()
            || child.style.overflow.x.scrolls()
            || child.style.overflow.y.scrolls()
            || has_escaping_descendant(child, elements)
    })
}
