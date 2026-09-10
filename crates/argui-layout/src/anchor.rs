use argui_core::Point;
use argui_ui::{NodeId, ScrollAnchoring, ScrollAxes, UiTree};

use crate::engine::{LayoutOutput, NodeMap};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ScrollAnchor {
    container: NodeId,
    target: NodeId,
    relative: Point,
}

pub(crate) fn capture(root: &NodeMap, output: &LayoutOutput) -> Vec<ScrollAnchor> {
    output
        .scroll_regions
        .iter()
        .filter(|region| region.config.anchoring == ScrollAnchoring::Auto)
        .filter_map(|region| {
            let map = find(root, region.node)?;
            let target = first_visible_keyed(map, output, region.clip)?;
            Some(ScrollAnchor {
                container: region.node,
                target: target.node,
                relative: Point::new(
                    target.bounds.origin.x - region.bounds.origin.x,
                    target.bounds.origin.y - region.bounds.origin.y,
                ),
            })
        })
        .collect()
}

fn first_visible_keyed<'a>(
    node: &NodeMap,
    output: &'a LayoutOutput,
    clip: argui_core::Rect,
) -> Option<&'a crate::LayoutNode> {
    node.children
        .iter()
        .filter(|child| child.element.portal.is_none())
        .flat_map(|child| {
            let own = (!child.element.semantic_hidden && child.element.key.is_some())
                .then(|| output.nodes.get(child.index))
                .flatten()
                .filter(|layout| {
                    layout.bounds.size.width > 0.0
                        && layout.bounds.size.height > 0.0
                        && layout.bounds.intersection(clip).is_some()
                });
            own.into_iter()
                .chain(first_visible_keyed(child, output, clip))
        })
        .min_by(|left, right| {
            left.bounds
                .origin
                .y
                .total_cmp(&right.bounds.origin.y)
                .then(left.bounds.origin.x.total_cmp(&right.bounds.origin.x))
        })
}

pub(crate) fn apply(anchors: &[ScrollAnchor], ui: &mut UiTree, output: &LayoutOutput) -> bool {
    anchors.iter().fold(false, |changed, anchor| {
        let Some(region) = output
            .scroll_regions
            .iter()
            .find(|region| region.node == anchor.container)
        else {
            return changed;
        };
        let Some(target) = output.nodes.iter().find(|node| node.node == anchor.target) else {
            return changed;
        };
        let relative = Point::new(
            target.bounds.origin.x - region.bounds.origin.x,
            target.bounds.origin.y - region.bounds.origin.y,
        );
        let mut offset = ui.scroll_offset(region.node);
        match region.config.axes {
            ScrollAxes::Horizontal => offset.x += relative.x - anchor.relative.x,
            ScrollAxes::Vertical => offset.y += relative.y - anchor.relative.y,
            ScrollAxes::Both => {
                offset.x += relative.x - anchor.relative.x;
                offset.y += relative.y - anchor.relative.y;
            }
        }
        offset.x = offset.x.clamp(0.0, region.max_offset.x);
        offset.y = offset.y.clamp(0.0, region.max_offset.y);
        ui.set_scroll_offset(region.node, offset) || changed
    })
}

fn find(node: &NodeMap, target: NodeId) -> Option<&NodeMap> {
    if node.node == target {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, target))
}
