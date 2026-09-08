use argui_core::{Point, Size, Transform2D};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, UiTree, length};

#[test]
fn semantic_geometry_is_transformed_and_survives_cached_repaint() {
    let root = |offset| {
        Element::container([Element::container([])
            .keyed("child")
            .width(length(30.0))
            .height(length(20.0))])
        .transform(Transform2D::IDENTITY.translate(offset, 10.0))
    };
    let mut ui = UiTree::new(root(25.0));
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(200.0, 100.0))
        .unwrap();
    let child = ui.node_ids()[1];
    let bounds = |output: &argui_layout::LayoutOutput| {
        output
            .semantic_bounds
            .iter()
            .find(|(node, _)| *node == child)
            .unwrap()
            .1
    };
    assert_eq!(bounds(&output).origin, Point::new(25.0, 10.0));
    let before = output.semantic_bounds.clone();
    engine.repaint(&ui, &mut output);
    assert_eq!(output.semantic_bounds, before);
    assert!(output.paint_stats.reused_subtrees > 0);
    ui.update(root(45.0));
    engine.repaint(&ui, &mut output);
    assert_eq!(bounds(&output).origin, Point::new(45.0, 10.0));
    assert_eq!(output.semantic_bounds.len(), 2);
    ui.update(root(400.0));
    engine.repaint(&ui, &mut output);
    assert_eq!(bounds(&output).size, Size::default());
}
