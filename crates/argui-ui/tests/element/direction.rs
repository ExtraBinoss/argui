use argui_ui::{Element, UiTree, WritingDirection};

#[test]
fn inherited_direction_keeps_explicit_local_layout_without_a_scope() {
    let mut tree = UiTree::new(Element::row([]).writing_direction(WritingDirection::Rtl));
    let node = tree.node_ids()[0];
    assert_eq!(
        tree.resolved_layout_style(node, tree.element_at(0).unwrap())
            .writing_direction,
        WritingDirection::Rtl
    );
    tree.update(Element::row([]).direction_scope(WritingDirection::Ltr));
    assert_eq!(
        tree.resolved_layout_style(tree.node_ids()[0], tree.element_at(0).unwrap())
            .writing_direction,
        WritingDirection::Ltr
    );
}
