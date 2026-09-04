use argui_ui::{NodeId, UiTree};

use crate::engine::{LayoutOutput, NodeMap};

pub(crate) fn measure(root: &NodeMap, output: &LayoutOutput, ui: &mut UiTree) -> bool {
    visit(root, None, output, ui)
}

fn visit(
    node: &NodeMap,
    scroll_container: Option<NodeId>,
    output: &LayoutOutput,
    ui: &mut UiTree,
) -> bool {
    let mut changed = false;
    if let (Some(item), Some(container), Some(layout)) = (
        node.element.virtual_item(),
        scroll_container,
        output.nodes.get(node.index),
    ) {
        let offset = ui.scroll_offset(container);
        let measurement = item.measure_layout(layout.layout_bounds.size.height, offset.y);
        if measurement.changed {
            changed = true;
            ui.set_scroll_offset(
                container,
                argui_core::Point::new(offset.x, measurement.corrected_offset),
            );
        }
    }
    let child_scroll_container = crate::scroll::config(node, &node.element)
        .is_some()
        .then_some(node.node)
        .or(scroll_container);
    node.children.iter().fold(changed, |changed, child| {
        visit(child, child_scroll_container, output, ui) || changed
    })
}
