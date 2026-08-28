use argui_core::Size;
use argui_devtools::DevtoolsHost;
use argui_layout::LayoutEngine;
use argui_runtime::{LayoutBounds, LayoutSnapshot, ViewUpdate};
use argui_showcase::{StateShowcase, text_engine};
use argui_ui::{Element, UiEvent, UiEventKind, UiTree};

fn event(key: &str) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent {
        target: tree.node_id_at(0).unwrap(),
        key: Some(key.to_owned()),
        kind: UiEventKind::Clicked,
    }
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
