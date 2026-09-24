use super::{editor, field, key, type_text};
use argui_core::{CaretAffinity, ImeInput, Key, Modifiers, TextPosition};
use argui_ui::TextInputFilter;
use argui_ui::{HistoryConfig, SelectionCommand, TextSelection, TextSelectionRequest, UiEventKind};

#[test]
fn explicit_actions_cancel_preedit_even_without_an_undo_transaction() {
    let (mut tree, region) = editor("", TextInputFilter::Any);
    for id in [argui_ui::ActionId::UNDO, argui_ui::ActionId::REDO] {
        tree.ime_input(ImeInput::Preedit {
            text: "preedit".into(),
            cursor: None,
        });
        assert!(!tree.can_undo(region.node));
        let update = tree.invoke_action(argui_ui::ActionInvocation::new(id).at(region.node));
        assert!(update.layout_changed);
        assert!(!tree.text_input_composing(region.node));
        assert_eq!(tree.text_input_value(region.node), Some(""));
    }
}

#[test]
fn delayed_paste_and_ime_cannot_modify_a_disabled_editor() {
    let (mut tree, region) = editor("initial", TextInputFilter::Any);
    tree.paste_text(None, "!");
    tree.ime_input(ImeInput::Preedit {
        text: "composing".into(),
        cursor: None,
    });
    let mut disabled = field("initial", TextInputFilter::Any);
    disabled.interaction.as_mut().unwrap().enabled = false;
    tree.replace(disabled);
    assert!(!tree.can_undo(region.node));
    assert!(
        !tree
            .paste_text(Some(region.node), "late clipboard")
            .layout_changed
    );
    tree.ime_input(ImeInput::Commit("late commit".into()));
    assert!(!tree.text_input_composing(region.node));
    assert_eq!(tree.text_input_value(region.node), Some("initial!"));
    tree.selection_command(Some(region.node), SelectionCommand::Cut);
    type_text(&mut tree, "x");
    assert_eq!(tree.text_input_value(region.node), Some("initial!"));
}

#[test]
fn authored_history_configuration_is_retained_and_zero_disables_recording() {
    let (mut tree, region) = editor("", TextInputFilter::Any);
    let config = HistoryConfig {
        transactions: 1,
        ..Default::default()
    };
    tree.replace(field("", TextInputFilter::Any).text_history(config));
    tree.paste_text(None, "a");
    tree.paste_text(None, "b");
    tree.replace(field("ab", TextInputFilter::Any).text_history(config));
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("a"));
    assert!(!tree.can_undo(region.node));
    tree.replace(
        field("a", TextInputFilter::Any).text_history(HistoryConfig {
            transactions: 0,
            ..config
        }),
    );
    tree.paste_text(None, "x");
    assert!(!tree.can_undo(region.node));
}

#[test]
fn typing_groups_and_redo_branches_are_discarded_by_a_new_edit() {
    let (mut tree, region) = editor("", TextInputFilter::Any);
    for value in ["A", "👋🏽", "é"] {
        type_text(&mut tree, value);
    }
    assert!(tree.can_undo(region.node));
    let update = tree.undo_text_input(region.node);
    assert!(matches!(&update.events[0].kind, UiEventKind::TextChanged(value) if value.is_empty()));
    assert_eq!(update.events[0].edit_history(), Some((false, true)));
    tree.redo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("A👋🏽é"));
    tree.undo_text_input(region.node);
    type_text(&mut tree, "new");
    assert!(!tree.can_redo(region.node));
    assert!(!tree.redo_text_input(region.node).layout_changed);
}

#[test]
fn paste_and_programmatic_replacement_are_separate_transactions() {
    let (mut tree, region) = editor("start", TextInputFilter::Any);
    type_text(&mut tree, "!");
    tree.paste_text(None, " paste");
    tree.replace_text_input(region.node, "entire");
    for expected in ["start! paste", "start!", "start"] {
        tree.undo_text_input(region.node);
        assert_eq!(tree.text_input_value(region.node), Some(expected));
    }
    for expected in ["start!", "start! paste", "entire"] {
        tree.redo_text_input(region.node);
        assert_eq!(tree.text_input_value(region.node), Some(expected));
    }
}

#[test]
fn transactions_restore_selection_direction_and_affinity() {
    let (mut tree, region) = editor("A👩‍🚀Z", TextInputFilter::Any);
    let anchor = TextPosition::new("A👩‍🚀".len(), CaretAffinity::After);
    let cursor = TextPosition::new(1, CaretAffinity::Before);
    tree.select_text(TextSelectionRequest::new(
        region.node,
        TextSelection::Range { anchor, cursor },
    ));
    tree.paste_text(None, "replacement");
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("A👩‍🚀Z"));
    assert_eq!(
        tree.text_input_selection_positions(region.node),
        Some((anchor, cursor))
    );
}

#[test]
fn navigation_and_change_of_delete_direction_break_groups() {
    let (mut tree, region) = editor("abcd", TextInputFilter::Any);
    tree.edit_text_input(&key(Key::Backspace, None, Modifiers::default()));
    tree.edit_text_input(&key(Key::Backspace, None, Modifiers::default()));
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("abcd"));
    tree.move_text_position(
        region.node,
        TextPosition::new(1, CaretAffinity::Before),
        false,
    );
    tree.edit_text_input(&key(Key::Delete, None, Modifiers::default()));
    tree.edit_text_input(&key(Key::Backspace, None, Modifiers::default()));
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("acd"));
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("abcd"));
}

#[test]
fn grouping_interval_is_configurable_without_sleeping_in_tests() {
    let (mut tree, region) = editor("", TextInputFilter::Any);
    tree.configure_text_history(
        region.node,
        HistoryConfig {
            grouping_interval: std::time::Duration::ZERO,
            ..Default::default()
        },
    );
    type_text(&mut tree, "a");
    type_text(&mut tree, "b");
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("a"));
}

#[test]
fn count_and_byte_limits_evict_old_history_and_oversized_edits_clear_it() {
    let (mut tree, region) = editor("", TextInputFilter::Any);
    tree.configure_text_history(
        region.node,
        HistoryConfig {
            transactions: 2,
            bytes: 3,
            ..Default::default()
        },
    );
    for value in ["a", "b", "c"] {
        tree.paste_text(None, value);
    }
    tree.undo_text_input(region.node);
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("a"));
    assert!(!tree.can_undo(region.node));
    tree.paste_text(None, "oversized");
    assert!(!tree.can_undo(region.node));
    assert!(!tree.can_redo(region.node));
    tree.configure_text_history(
        region.node,
        HistoryConfig {
            transactions: 0,
            ..Default::default()
        },
    );
    tree.paste_text(None, "x");
    assert!(!tree.can_undo(region.node));
}

#[test]
fn unchanged_authored_value_keeps_history_during_layout_rebuild_external_change_resets() {
    let (mut tree, region) = editor("base", TextInputFilter::Any);
    type_text(&mut tree, "!");
    tree.replace(field("base", TextInputFilter::Any).width(argui_ui::length(450.0)));
    assert_eq!(tree.text_input_value(region.node), Some("base!"));
    assert!(tree.can_undo(region.node));
    tree.replace(field("base!", TextInputFilter::Any));
    assert!(tree.can_undo(region.node));
    tree.replace(field("remote", TextInputFilter::Any));
    assert_eq!(tree.text_input_value(region.node), Some("remote"));
    assert!(!tree.can_undo(region.node));
}

#[test]
fn delayed_controlled_values_do_not_rewind_newer_native_edits() {
    let (mut tree, region) = editor("", TextInputFilter::Any);
    type_text(&mut tree, "a");
    type_text(&mut tree, "b");
    assert_eq!(tree.text_input_value(region.node), Some("ab"));
    assert_eq!(tree.text_input_cursor(region.node), Some(2));

    tree.replace(field("a", TextInputFilter::Any));
    assert_eq!(tree.text_input_value(region.node), Some("ab"));
    assert_eq!(tree.text_input_cursor(region.node), Some(2));
    tree.replace(field("ab", TextInputFilter::Any));
    assert_eq!(tree.text_input_value(region.node), Some("ab"));
    assert_eq!(tree.text_input_cursor(region.node), Some(2));

    tree.replace(field("remote", TextInputFilter::Any));
    assert_eq!(tree.text_input_value(region.node), Some("remote"));
    assert!(!tree.can_undo(region.node));
}

#[test]
fn burst_typing_preserves_the_latest_caret_through_delayed_echoes() {
    let (mut tree, region) = editor("", TextInputFilter::Any);
    let mut value = String::new();
    let mut echoes = Vec::new();
    for index in 0..48 {
        let character = char::from(b'a' + (index % 26) as u8);
        type_text(&mut tree, &character.to_string());
        value.push(character);
        echoes.push(value.clone());
    }
    assert_eq!(tree.text_input_value(region.node), Some(value.as_str()));
    assert_eq!(tree.text_input_cursor(region.node), Some(value.len()));

    for echo in echoes {
        tree.replace(field(&echo, TextInputFilter::Any));
        assert_eq!(tree.text_input_value(region.node), Some(value.as_str()));
        assert_eq!(tree.text_input_cursor(region.node), Some(value.len()));
    }
    type_text(&mut tree, "é");
    value.push('é');
    assert_eq!(tree.text_input_value(region.node), Some(value.as_str()));
    assert_eq!(tree.text_input_cursor(region.node), Some(value.len()));
}

#[test]
fn preedit_has_no_history_and_commit_is_one_atomic_edit() {
    let (mut tree, region) = editor("", TextInputFilter::Any);
    tree.ime_input(ImeInput::Preedit {
        text: "に".into(),
        cursor: None,
    });
    assert!(!tree.can_undo(region.node));
    assert!(tree.text_input_composing(region.node));
    type_text(&mut tree, "x");
    assert_eq!(tree.text_input_value(region.node), Some(""));
    tree.ime_input(ImeInput::Commit("日本".into()));
    type_text(&mut tree, "!");
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("日本"));
    tree.ime_input(ImeInput::Preedit {
        text: "語".into(),
        cursor: None,
    });
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_display(region.node).as_deref(), Some(""));
    assert!(!tree.text_input_composing(region.node));
}

#[test]
fn rejected_filtered_edits_preserve_redo_and_clear_preedit_paint() {
    let (mut tree, region) = editor("", TextInputFilter::Decimal);
    tree.paste_text(None, "-");
    tree.paste_text(None, "12.5");
    tree.undo_text_input(region.node);
    assert!(!tree.paste_text(None, "wrong").layout_changed);
    assert!(!tree.replace_text_input(region.node, "bad").layout_changed);
    tree.ime_input(ImeInput::Preedit {
        text: "字".into(),
        cursor: None,
    });
    assert!(tree.ime_input(ImeInput::Commit("字".into())).layout_changed);
    assert!(tree.can_redo(region.node));
    tree.redo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("-12.5"));
}

#[test]
fn cut_undo_redo_and_keyboard_shortcuts_share_state() {
    let (mut tree, region) = editor("selected", TextInputFilter::Any);
    tree.selection_command(Some(region.node), SelectionCommand::SelectAll);
    let cut = tree.selection_command(Some(region.node), SelectionCommand::Cut);
    assert!(cut.clipboard.is_some());
    tree.edit_text_input(&key(
        Key::Character("z".into()),
        None,
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
    ));
    assert_eq!(tree.text_input_value(region.node), Some("selected"));
    tree.edit_text_input(&key(
        Key::Character("z".into()),
        None,
        Modifiers {
            control: true,
            shift: true,
            ..Modifiers::default()
        },
    ));
    assert_eq!(tree.text_input_value(region.node), Some(""));
    tree.selection_command(Some(region.node), SelectionCommand::Undo);
    tree.selection_command(Some(region.node), SelectionCommand::Redo);
    assert_eq!(tree.text_input_value(region.node), Some(""));
}

#[test]
fn read_only_and_disabled_history_commands_do_not_mutate() {
    let (mut tree, region) = editor("base", TextInputFilter::Any);
    tree.paste_text(None, "!");
    let mut input = field("base", TextInputFilter::Any);
    if let argui_ui::ElementKind::TextEditor { read_only, .. } = &mut input.kind {
        *read_only = true;
    }
    tree.replace(input);
    assert!(!tree.can_undo(region.node));
    assert!(!tree.undo_text_input(region.node).layout_changed);
    let mut input = field("base", TextInputFilter::Any);
    input.interaction.as_mut().unwrap().enabled = false;
    tree.replace(input);
    assert!(!tree.undo_text_input(region.node).layout_changed);
    assert_eq!(tree.text_input_value(region.node), Some("base!"));
}
