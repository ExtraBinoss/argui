//! Measures mounted virtual items and requests native item windows.

use std::collections::HashMap;

use argui_core::Point;
use argui_ui::{NodeId, UiEventKind, UiTree, VirtualMeasurement};

use crate::engine::{LayoutOutput, NodeMap};

/// Records measured item extents and emits changed native windows.
///
/// * `root` — retained layout branch to scan.
/// * `output` — computed geometry and event deliveries to update.
/// * `ui` — scroll offsets and native event listener state.
///
/// Returns whether a measurement changed and requires another layout pass.
pub(crate) fn measure(root: &NodeMap, output: &mut LayoutOutput, ui: &mut UiTree) -> bool {
    let mut reports = HashMap::<NodeId, Vec<VirtualMeasurement>>::new();
    let mut containers = Vec::<(NodeId, f32, bool)>::new();
    let changed = visit(root, None, output, ui, &mut reports, &mut containers);
    for (node, viewport_extent, horizontal) in containers {
        if let Some(items) = reports.remove(&node) {
            let offset = ui.scroll_offset(node);
            output.virtual_events.extend(ui.event_deliveries(
                node,
                UiEventKind::VirtualMeasured {
                    items,
                    corrected_offset: if horizontal { offset.x } else { offset.y },
                    viewport_extent,
                },
            ));
        }
        output
            .virtual_events
            .extend(ui.request_virtual_window(node, viewport_extent));
    }
    changed
}

/// Visits one retained layout branch and measures its mounted virtual items.
///
/// * `node` — current retained layout branch.
/// * `scroll_container` — nearest scroll viewport enclosing this branch.
/// * `output` — computed geometry for the branch.
/// * `ui` — mutable scroll offsets.
/// * `reports` — measurements grouped by scroll viewport.
/// * `containers` — measured native viewport extents.
///
/// Returns whether any item changed its retained extent.
fn visit(
    node: &NodeMap,
    scroll_container: Option<NodeId>,
    output: &LayoutOutput,
    ui: &mut UiTree,
    reports: &mut HashMap<NodeId, Vec<VirtualMeasurement>>,
    containers: &mut Vec<(NodeId, f32, bool)>,
) -> bool {
    let layout = output.nodes.get(node.index);
    if let (Some(viewport), Some(layout)) = (node.element.virtual_viewport(), layout) {
        let extent = if viewport.list.is_horizontal() {
            layout.layout_bounds.size.width
        } else {
            layout.layout_bounds.size.height
        };
        containers.push((node.node, extent, viewport.list.is_horizontal()));
    }
    let mut changed = false;
    if let (Some(item), Some(container), Some(layout)) =
        (node.element.virtual_item(), scroll_container, layout)
    {
        let offset = ui.scroll_offset(container);
        let extent = if item.is_horizontal() {
            layout.layout_bounds.size.width
        } else {
            layout.layout_bounds.size.height
        };
        let measurement = item.measure_layout(
            extent,
            if item.is_horizontal() {
                offset.x
            } else {
                offset.y
            },
        );
        if measurement.changed {
            changed = true;
            reports
                .entry(container)
                .or_default()
                .push(VirtualMeasurement {
                    index: item.index(),
                    extent,
                });
            ui.set_scroll_offset(
                container,
                if item.is_horizontal() {
                    Point::new(measurement.corrected_offset, offset.y)
                } else {
                    Point::new(offset.x, measurement.corrected_offset)
                },
            );
        }
    }
    let child_scroll_container = crate::scroll::config(node, &node.element)
        .is_some()
        .then_some(node.node)
        .or(scroll_container);
    for child in &node.children {
        changed |= visit(
            child,
            child_scroll_container,
            output,
            ui,
            reports,
            containers,
        );
    }
    changed
}
