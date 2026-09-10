use argui_core::{Point, Size, Transform2D};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, UiTree, length};

#[test]
fn nested_transformed_clips_limit_accessible_bounds_after_cached_repaint() {
    use argui_core::Rect;
    use argui_ui::{Axes, Overflow};
    let clipped = |child| {
        Element::container([child])
            .width(length(50.0))
            .height(length(50.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            })
    };
    let leaf = Element::container([])
        .keyed("leaf")
        .width(length(100.0))
        .height(length(100.0))
        .shrink(0.0);
    let mut ui = UiTree::new(
        clipped(clipped(leaf).transform(Transform2D::IDENTITY.translate(20.0, 20.0)))
            .transform(Transform2D::IDENTITY.translate(10.0, 10.0)),
    );
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(200.0, 200.0))
        .unwrap();
    for _ in 0..2 {
        let leaf = output
            .semantic_bounds
            .iter()
            .find(|(node, _)| ui.key(*node) == Some("leaf"))
            .unwrap()
            .1;
        assert_eq!(
            leaf,
            Rect::new(Point::new(30.0, 30.0), Size::new(30.0, 30.0))
        );
        engine.repaint(&ui, &mut output);
    }
}

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
