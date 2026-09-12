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

#[test]
fn compact_parents_and_selection_owners_follow_nested_scopes_and_stale_ids() {
    use argui_core::Color;
    use argui_ui::{FocusTarget, WritingDirection};
    let outer = TextSelectionStyle {
        background: Color::BLACK,
        handle: Color::WHITE,
    };
    let inner = TextSelectionStyle {
        background: Color::WHITE,
        handle: Color::BLACK,
    };
    let branch = |name: &str, style| {
        Element::column([Element::text(name).keyed(format!("{name}-leaf"))])
            .keyed(name)
            .selection_style(style)
    };
    let mut tree = UiTree::new(
        Element::column([branch("a", inner), branch("b", outer)])
            .selection_style(outer)
            .user_select(UserSelect::None)
            .direction_scope(WritingDirection::Rtl),
    );
    let root = tree.node_ids()[0];
    let a = tree.node_ids()[1];
    let leaf = tree.node_ids()[2];
    let b = tree.node_ids()[3];
    assert_eq!(tree.parent_of(root), None);
    assert_eq!(tree.parent_of(leaf), Some(a));
    assert_eq!(tree.resolved_selection_style(leaf), inner);
    assert_eq!(tree.resolved_user_select(leaf), UserSelect::None);
    let mut reordered = tree.root().clone();
    reordered.children.swap(0, 1);
    reordered.children[1].selection_style = None;
    reordered.children[1].user_select = UserSelect::All;
    tree.update(reordered);
    assert_eq!(tree.parent_of(leaf), Some(a));
    assert_eq!(tree.parent_of(b), Some(root));
    assert_eq!(tree.resolved_selection_style(leaf), outer);
    assert_eq!(tree.resolved_user_select(leaf), UserSelect::All);
    // Drop an interior subtree, then mount a fresh one at the same dense slots.
    tree.update(Element::column([branch("b", inner), branch("c", outer)]));
    assert_eq!(tree.parent_of(a), None);
    assert_eq!(tree.parent_of(leaf), None);
    assert!(tree.element_for(leaf).is_none());
    let c = tree
        .resolve_node(&FocusTarget::Key("c-leaf".into()))
        .unwrap();
    assert_ne!(c, leaf);
    assert_eq!(tree.resolved_selection_style(c), outer);
    assert_eq!(tree.resolved_user_select(c), UserSelect::Text);
}
