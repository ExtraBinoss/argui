use super::*;

#[test]
fn word_shortcuts_extend_selections_and_delete_punctuation_separately() {
    let command = Modifiers {
        control: true,
        ..Default::default()
    };
    let selecting = Modifiers {
        shift: true,
        ..command
    };
    let (mut ui, region) = tree("one,  two! three");
    let node = region.node;
    ui.move_text_cursor(node, 0, false);
    focus(&mut ui, &region);
    for end in [3, 4, 9, 10, 16, 16] {
        ui.edit_text_input(&key(Key::ArrowRight, None, selecting));
        assert_eq!(ui.text_input_selection(node), Some((0, end)));
    }
    for end in [11, 9, 6, 3, 0, 0] {
        ui.edit_text_input(&key(Key::ArrowLeft, None, selecting));
        assert_eq!(ui.text_input_cursor(node), Some(end));
        assert_eq!(ui.text_input_selection(node), (end > 0).then_some((0, end)));
    }
    ui.edit_text_input(&key(Key::Delete, None, command));
    assert_eq!(ui.text_input_value(node), Some(",  two! three"));
    ui.edit_text_input(&key(Key::Delete, None, command));
    assert_eq!(ui.text_input_value(node), Some("  two! three"));
    ui.edit_text_input(&key(Key::Character("z".into()), None, command));
    assert_eq!(ui.text_input_value(node), Some(",  two! three"));
    ui.edit_text_input(&key(Key::End, None, command));
    ui.edit_text_input(&key(Key::Backspace, None, command));
    assert_eq!(ui.text_input_value(node), Some(",  two! "));
    ui.edit_text_input(&key(Key::Backspace, None, command));
    assert_eq!(ui.text_input_value(node), Some(",  two"));
}

#[test]
fn unmodified_arrows_collapse_selection_without_skipping_a_character() {
    let (mut ui, region) = tree("one two three");
    focus(&mut ui, &region);
    for (anchor, cursor, key_value, expected) in [
        (4, 7, Key::ArrowLeft, 4),
        (4, 7, Key::ArrowRight, 7),
        (7, 4, Key::ArrowLeft, 4),
        (7, 4, Key::ArrowRight, 7),
    ] {
        ui.move_text_cursor(region.node, anchor, false);
        ui.move_text_cursor(region.node, cursor, true);
        ui.edit_text_input(&key(key_value, None, Modifiers::default()));
        assert_eq!(ui.text_input_cursor(region.node), Some(expected));
        assert_eq!(ui.text_input_selection(region.node), None);
    }
}

#[test]
fn word_deletion_handles_only_spaces_edges_newlines_and_unicode() {
    let command = Modifiers {
        control: true,
        ..Default::default()
    };
    for (value, cursor, key_value, expected) in [
        ("   ", 3, Key::Backspace, ""),
        ("   ", 0, Key::Delete, ""),
        ("é 👩‍🚀", 0, Key::Delete, " 👩‍🚀"),
        ("é 👩‍🚀", 14, Key::Backspace, "é "),
        ("one\ntwo", 4, Key::Backspace, "onetwo"),
        ("one\ntwo", 3, Key::Delete, "onetwo"),
    ] {
        let (mut ui, region) = with_region(UiTree::new(multiline_field(value, "")));
        focus(&mut ui, &region);
        ui.move_text_cursor(region.node, cursor, false);
        ui.edit_text_input(&key(key_value, None, command));
        assert_eq!(
            ui.text_input_value(region.node),
            Some(expected),
            "{value:?}"
        );
    }
}

#[test]
fn modifier_and_selection_paths_keep_editing_deterministic() {
    let (mut tree, region) = tree("one two");
    focus(&mut tree, &region);
    let node = region.node;
    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    let command = Modifiers {
        control: true,
        ..Modifiers::default()
    };

    tree.edit_text_input(&key(Key::Home, None, Modifiers::default()));
    tree.edit_text_input(&key(Key::ArrowRight, None, alt));
    tree.edit_text_input(&key(Key::ArrowLeft, None, alt));
    tree.edit_text_input(&key(Key::ArrowRight, None, shift));
    tree.edit_text_input(&key(Key::ArrowRight, None, shift));
    assert_eq!(tree.text_input_selection(node), Some((0, 2)));
    tree.edit_text_input(&key(Key::Backspace, None, Modifiers::default()));
    assert_eq!(tree.text_input_value(node), Some("e two"));

    assert!(
        !tree
            .edit_text_input(&key(Key::Home, None, command))
            .layout_changed
    );
    assert!(
        !tree
            .edit_text_input(&key(
                Key::Character(String::new()),
                Some(""),
                Modifiers::default()
            ))
            .layout_changed
    );
}
