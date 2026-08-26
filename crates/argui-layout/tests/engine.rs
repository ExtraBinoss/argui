use argui_layout::LayoutEngine;
use taffy::Style;

#[test]
fn engine_accepts_taffy_nodes() {
    let mut engine = LayoutEngine::new();
    let node = engine.tree_mut().new_leaf(Style::default());

    assert!(node.is_ok());
}
