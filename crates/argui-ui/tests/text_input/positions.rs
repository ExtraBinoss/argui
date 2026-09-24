use super::*;

#[test]
fn visual_positions_preserve_affinity_across_pointer_and_keyboard_updates() {
    let (mut tree, region) = tree("abc العربية xyz");
    focus(&mut tree, &region);
    let node = region.node;
    let after = TextPosition::new(4, CaretAffinity::After);
    let before = TextPosition::new(4, CaretAffinity::Before);

    tree.place_text_position(node, after, false);
    assert_eq!(tree.text_input_position(node), Some(after));
    tree.drag_text_position(node, before);
    assert_eq!(tree.text_input_position(node), Some(before));
    tree.move_text_position(node, after, true);
    assert_eq!(tree.text_input_position(node), Some(after));

    tree.update(argui_ui::Element::container([]));
    assert!(
        !tree
            .place_text_position(node, before, false)
            .text_input_changed
    );
    assert!(!tree.drag_text_position(node, before).text_input_changed);
    assert!(
        !tree
            .move_text_position(node, before, false)
            .text_input_changed
    );
}

#[test]
fn targeted_selection_resolves_stable_keys_and_grapheme_boundaries() {
    let (mut tree, region) = tree("a👋🏽z");
    let update = tree.select_text(TextSelectionRequest::new("field", TextSelection::All));
    assert!(update.text_input_changed);
    assert_eq!(
        tree.text_input_selection(region.node),
        Some((0, "a👋🏽z".len()))
    );

    tree.select_text(TextSelectionRequest::new(
        "field",
        TextSelection::Range {
            anchor: TextPosition::new(2, CaretAffinity::After),
            cursor: TextPosition::new("a👋🏽".len(), CaretAffinity::Before),
        },
    ));
    assert_eq!(
        tree.text_input_selection(region.node),
        Some((1, "a👋🏽".len()))
    );
    assert_eq!(
        tree.select_text(TextSelectionRequest::new("missing", TextSelection::All)),
        argui_ui::InteractionUpdate::default()
    );
}

