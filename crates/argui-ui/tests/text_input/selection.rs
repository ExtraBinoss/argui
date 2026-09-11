use super::*;
use argui_ui::SelectionGranularity::{Character, Line, Word};

fn position(index: usize) -> TextPosition {
    TextPosition::new(index, CaretAffinity::Before)
}

#[test]
fn double_click_selects_words_spaces_punctuation_and_whole_graphemes() {
    for (value, index, expected) in [
        ("one  two!", 1, Some((0, 3))),
        ("one  two!", 4, Some((3, 5))),
        ("one  two!", 8, Some((8, 9))),
        ("one two", 7, Some((4, 7))),
        ("e\u{301}lan 👩‍🚀", 1, Some((0, 6))),
        ("e\u{301}lan 👩‍🚀", 9, Some((7, 18))),
        ("", 0, None),
    ] {
        let (mut ui, region) = tree(value);
        ui.begin_text_selection(region.node, position(index), false, Word);
        assert_eq!(ui.text_input_selection(region.node), expected, "{value:?}");
    }
}

#[test]
fn word_drag_expands_in_both_directions_and_stops_on_release() {
    let (mut ui, region) = tree("one two three");
    let node = region.node;
    ui.begin_text_selection(node, position(5), false, Word);
    for (index, expected) in [(10, (4, 13)), (1, (0, 7)), (5, (4, 7))] {
        ui.drag_text_position(node, position(index));
        assert_eq!(ui.text_input_selection(node), Some(expected));
    }
    assert!(ui.release_text_cursor());
    assert!(!ui.release_text_cursor());
    assert!(!ui.drag_text_position(node, position(0)).layout_changed);
    assert_eq!(ui.text_input_selection(node), Some((4, 7)));
}

#[test]
fn triple_click_and_drag_select_complete_lines_including_the_newline() {
    let (mut ui, region) = with_region(UiTree::new(
        TextArea::new(
            "field",
            "one\ntwo\nthree",
            "",
            InputStyle::new(PaintStyle::default(), TextStyle::default()),
        )
        .build(),
    ));
    let node = region.node;
    ui.begin_text_selection(node, position(5), false, Line);
    assert_eq!(ui.text_input_selection(node), Some((4, 8)));
    ui.drag_text_position(node, position(12));
    assert_eq!(ui.text_input_selection(node), Some((4, 13)));
    ui.drag_text_position(node, position(1));
    assert_eq!(ui.text_input_selection(node), Some((0, 8)));
    ui.paste_text(Some(node), "new\n");
    assert_eq!(ui.text_input_value(node), Some("new\nthree"));
}

#[test]
fn shift_click_extends_the_original_anchor_in_either_direction() {
    let (mut ui, region) = tree("one two three");
    let node = region.node;
    ui.begin_text_selection(node, position(4), false, Character);
    ui.begin_text_selection(node, position(10), true, Character);
    assert_eq!(ui.text_input_selection(node), Some((4, 10)));
    ui.begin_text_selection(node, position(1), true, Character);
    assert_eq!(ui.text_input_selection(node), Some((1, 4)));
    ui.begin_text_selection(node, position(10), true, Word);
    assert_eq!(ui.text_input_selection(node), Some((4, 13)));
    ui.begin_text_selection(node, position(1), true, Word);
    assert_eq!(ui.text_input_selection(node), Some((0, 4)));
}

#[test]
fn readonly_is_selectable_password_words_are_private_and_composition_is_untouched() {
    let (mut ui, region) = read_only_tree("one two");
    ui.begin_text_selection(region.node, position(5), false, Word);
    assert_eq!(ui.text_input_selection(region.node), Some((4, 7)));
    assert!(!ui.paste_text(Some(region.node), "changed").layout_changed);

    let (mut ui, region) = filtered_tree("one 👩‍🚀", InputKind::Password);
    ui.begin_text_selection(region.node, position(3), false, Word);
    assert_eq!(ui.text_input_selection(region.node), Some((0, 15)));
    focus(&mut ui, &region);
    ui.ime_input(ImeInput::Preedit {
        text: "é".into(),
        cursor: None,
    });
    assert!(
        !ui.begin_text_selection(region.node, position(0), false, Word)
            .layout_changed
    );
    assert!(ui.text_input_composing(region.node));

    ui.update(Element::text("removed"));
    assert!(
        !ui.begin_text_selection(region.node, position(0), false, Word)
            .layout_changed
    );
    assert!(!ui.release_text_cursor());
}
