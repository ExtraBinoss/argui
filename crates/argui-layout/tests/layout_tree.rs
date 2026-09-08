use argui_core::Size;
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, UiTree, length};
#[path = "layout_tree/algorithms/custom.rs"]
mod custom;

#[test]
fn replacing_roots_and_subtrees_does_not_retain_detached_layout_nodes() {
    let mut tree = UiTree::new(Element::container([]));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    for iteration in 0..1_000 {
        let (root, expected) = if iteration % 2 == 0 {
            (
                Element::row([
                    Element::container([]).width(length(10.0)),
                    Element::column([Element::container([]).height(length(20.0))]),
                ])
                .keyed("container"),
                4,
            )
        } else {
            (Element::text("leaf").keyed("leaf"), 1)
        };
        tree.update(root);
        let output = engine
            .compute(&mut tree, &mut text, Size::new(300.0, 200.0))
            .unwrap();
        assert_eq!(output.nodes.len(), expected);
        assert_eq!(engine.retained_node_count(), expected);
    }
}
