use argui_core::Size;
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, UiTree, WritingDirection, length};
use argui_widgets::Direction;

#[test]
fn scopes_reverse_nested_rows_and_can_be_overridden_or_changed() {
    let build = |direction| {
        Direction::new(
            direction,
            Element::column([
                Element::row([
                    Element::container([])
                        .keyed("a")
                        .width(length(30.0))
                        .height(length(20.0)),
                    Element::container([])
                        .keyed("b")
                        .width(length(30.0))
                        .height(length(20.0)),
                ])
                .keyed("inherited"),
                Direction::new(WritingDirection::Ltr, Element::row([]).keyed("override")).build(),
            ]),
        )
        .build()
    };
    let mut tree = UiTree::new(build(WritingDirection::Ltr));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    for direction in [
        WritingDirection::Ltr,
        WritingDirection::Rtl,
        WritingDirection::Ltr,
    ] {
        tree.update(build(direction));
        let output = engine
            .compute(&mut tree, &mut text, Size::new(300.0, 200.0))
            .unwrap();
        let node = |key| {
            output
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some(key))
                .unwrap()
        };
        assert_eq!(
            node("a").bounds.origin.x > node("b").bounds.origin.x,
            direction == WritingDirection::Rtl
        );
        let nested = node("override");
        assert_eq!(
            tree.resolved_layout_style(nested.node, tree.element_at(nested.index).unwrap())
                .writing_direction,
            WritingDirection::Ltr
        );
    }
}
