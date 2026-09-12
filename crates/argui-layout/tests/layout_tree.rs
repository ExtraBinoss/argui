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

#[test]
fn dense_columns_keep_geometry_and_caches_aligned_after_compaction_and_reordering() {
    let leaf = |id: usize| {
        Element::container([])
            .keyed(format!("leaf-{id}"))
            .width(length(5.0 + (id % 11) as f32))
            .height(length(7.0 + (id % 17) as f32))
    };
    let mut ui = UiTree::new(Element::row((0..300).map(leaf)));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    let viewport = Size::new(800.0, 600.0);
    engine.compute(&mut ui, &mut text, viewport).unwrap();
    for cycle in 0..12 {
        let count = if cycle % 3 == 0 { 3 } else { 180 };
        let root = Element::column((0..count).rev().map(|i| {
            Element::row([leaf(i), leaf(i + 1000)]).keyed(format!("group-{}", i + cycle))
        }));
        ui.update(root.clone());
        let retained = engine.compute(&mut ui, &mut text, viewport).unwrap();
        let fresh = LayoutEngine::new()
            .compute(&mut UiTree::new(root), &mut text, viewport)
            .unwrap();
        assert_eq!(engine.retained_node_count(), count * 3 + 1);
        assert_eq!(retained.nodes.len(), fresh.nodes.len());
        for (actual, expected) in retained.nodes.iter().zip(&fresh.nodes) {
            assert_eq!(actual.index, expected.index);
            assert_eq!(actual.bounds, expected.bounds);
            assert_eq!(actual.layout_bounds, expected.layout_bounds);
            assert_eq!(actual.clip, expected.clip);
        }
        // A cached computation must read the same slots as the cold computation.
        let cached = engine.compute(&mut ui, &mut text, viewport).unwrap();
        assert_eq!(cached.nodes, retained.nodes);
    }
}
