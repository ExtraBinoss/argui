use argui_core::{Point, Size};
use argui_devtools::DevtoolsHost;
use argui_layout::LayoutEngine;
use argui_runtime::{LayoutBounds, LayoutSnapshot, ViewUpdate};
use argui_showcase::{StateShowcase, text_engine};
use argui_ui::{Element, UiEvent, UiEventKind, UiTree};

fn event(key: &str) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(
        tree.node_id_at(0).unwrap(),
        Some(key.to_owned()),
        UiEventKind::Clicked,
    )
}

#[test]
fn opening_tools_survives_the_transient_zero_height_application_viewport() {
    let mut host = DevtoolsHost::new(StateShowcase::default());
    assert_eq!(
        host.update(&event("__devtools-toggle")),
        ViewUpdate::Rebuild
    );
    let mut tree = UiTree::new(host.view());
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(1_100.0, 230.0))
        .unwrap();
    let snapshot = LayoutSnapshot {
        viewport: output.viewport,
        nodes: output
            .nodes
            .iter()
            .map(|layout| LayoutBounds {
                node: layout.node,
                key: tree.key(layout.node).map(str::to_owned),
                bounds: layout.bounds,
            })
            .collect(),
    };

    assert_eq!(host.layout_changed(&snapshot), ViewUpdate::None);
}

#[test]
fn toggle_button_keeps_its_authored_radius_while_pressed() {
    let mut host = DevtoolsHost::new(StateShowcase::default());
    let mut tree = UiTree::new(host.view());
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(1_100.0, 700.0))
        .unwrap();
    let (index, node) = tree
        .node_ids()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, node)| tree.key(*node) == Some("__devtools-toggle"))
        .unwrap();
    let initial = tree
        .resolved_quad(node, tree.element_at(index).unwrap())
        .radii;
    let hit = output
        .hit_regions
        .iter()
        .find(|region| region.node == node)
        .unwrap();
    let local = Point::new(
        hit.bounds.origin.x + hit.bounds.size.width * 0.5,
        hit.bounds.origin.y + hit.bounds.size.height * 0.5,
    );
    let point = hit.transform.transform_point(local);

    tree.pointer_moved(point, &output.hit_regions);
    tree.primary_pressed(&output.hit_regions);
    tree.set_reduced_motion(true);

    assert_eq!(
        tree.resolved_quad(node, tree.element_at(index).unwrap())
            .radii,
        initial
    );
}
