use argui_ui::{Element, TextSelectionStyle, UiTree, UserSelect};

#[test]
fn indexed_elements_follow_reordering_removal_and_inherited_selection_changes() {
    let first = Element::text("first").keyed("first");
    let second = Element::text("second").keyed("second");
    let root = Element::column([first.clone(), second.clone()]).user_select(UserSelect::None);
    let mut tree = UiTree::new(root);
    let first_id = tree.node_ids()[1];
    let second_id = tree.node_ids()[2];
    assert_eq!(tree.resolved_user_select(first_id), UserSelect::None);
    let style = TextSelectionStyle {
        background: argui_core::Color::WHITE,
        ..TextSelectionStyle::default()
    };
    tree.update(Element::column([second, first]).selection_style(style));
    assert_eq!(tree.node_ids()[1], second_id);
    assert_eq!(tree.element_at(1).unwrap().key.as_deref(), Some("second"));
    assert_eq!(tree.key(first_id), Some("first"));
    assert_eq!(tree.resolved_user_select(first_id), UserSelect::Text);
    assert_eq!(tree.resolved_selection_style(first_id), style);
    tree.update(Element::column([Element::text("changed").keyed("first")]));
    assert_eq!(tree.key(first_id), Some("first"));
    assert_eq!(tree.key(second_id), None);
    assert_eq!(tree.resolved_user_select(second_id), UserSelect::None);
    assert_eq!(
        tree.resolved_selection_style(second_id),
        TextSelectionStyle::default()
    );
    assert!(tree.element_at(2).is_none());
}

#[test]
fn index_refreshes_for_paint_only_changes_and_explicit_selection_overrides() {
    let root = |color| {
        Element::column([Element::text("child")
            .keyed("child")
            .user_select(UserSelect::Text)
            .background(color)])
        .user_select(UserSelect::None)
    };
    let mut tree = UiTree::new(root(argui_core::Color::WHITE));
    let node = tree.node_ids()[1];
    tree.update(root(argui_core::Color::BLACK));
    assert_eq!(tree.resolved_user_select(node), UserSelect::Text);
    assert_eq!(
        tree.element_at(1).unwrap().paint,
        root(argui_core::Color::BLACK).children[0].paint
    );
}
