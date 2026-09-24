use super::{editor, editor_with_privacy, field, key};
use argui_core::{CaretAffinity, ImeInput, Key, Modifiers, TextPosition};
use argui_ui::TextInputFilter;
use argui_ui::{
    SelectionCommand, TextEdit, TextPrivacy, TextSelection, TextSelectionRequest, UiEvent,
    UiEventKind,
};

#[test]
fn passwords_mask_graphemes_and_map_mouse_positions_to_real_offsets() {
    let (mut tree, region) =
        editor_with_privacy("A👩‍🚀e\u{301}", TextInputFilter::Any, TextPrivacy::Password);
    assert_eq!(tree.text_input_display(region.node).as_deref(), Some("•••"));
    assert_eq!(tree.text_input_cursor(region.node), Some(9));
    tree.move_text_position(
        region.node,
        TextPosition::new(3, CaretAffinity::Before),
        false,
    );
    tree.paste_text(None, "x");
    assert_eq!(tree.text_input_value(region.node), Some("Ax👩‍🚀e\u{301}"));
    tree.select_text(TextSelectionRequest::new(region.node, TextSelection::All));
    let (anchor, cursor) = tree.text_input_selection_positions(region.node).unwrap();
    assert_eq!(anchor.index, 0);
    assert_eq!(cursor.index, 12);
}

#[test]
fn password_copy_cut_and_history_are_disabled_including_revealed_mode() {
    for privacy in [TextPrivacy::Password, TextPrivacy::RevealedPassword] {
        let (mut tree, region) =
            editor_with_privacy("secret", TextInputFilter::Any, TextPrivacy::Password);
        tree.replace(field("secret", TextInputFilter::Any).text_privacy(privacy));
        tree.selection_command(Some(region.node), SelectionCommand::SelectAll);
        let caps = tree.selection_capabilities(region.node);
        assert!(!caps.copy && !caps.cut && caps.paste && caps.select_all);
        for command in [SelectionCommand::Copy, SelectionCommand::Cut] {
            let result = tree.selection_command(Some(region.node), command);
            assert!(result.clipboard.is_none());
            assert!(!result.layout_changed);
        }
        tree.paste_text(None, "new");
        assert!(!tree.can_undo(region.node));
        assert!(!tree.undo_text_input(region.node).layout_changed);
        assert_eq!(tree.text_input_value(region.node), Some("new"));
    }
}

#[test]
fn password_semantics_debug_and_telemetry_do_not_export_plaintext() {
    let (tree, region) =
        editor_with_privacy("topsecret", TextInputFilter::Any, TextPrivacy::Password);
    let snapshot = tree.semantic_tree(&[], 1.0);
    assert!(snapshot.nodes[0].semantics.state.protected);
    assert!(snapshot.nodes[0].semantics.value.is_none());
    assert!(!format!("{snapshot:?} {tree:?}").contains("topsecret"));
    for kind in [
        UiEventKind::TextChanged("topsecret".into()),
        UiEventKind::TextEdited(TextEdit::new(0..0, "topsecret")),
        UiEventKind::Submitted("topsecret".into()),
        UiEventKind::KeyInput(key(
            Key::Character("topsecret".into()),
            Some("topsecret"),
            Modifiers::default(),
        )),
    ] {
        let event = UiEvent::new(region.node, None, kind);
        assert!(!format!("{:?}", tree.inspect_event(&event)).contains("topsecret"));
    }
}

#[test]
fn password_preedit_is_masked_and_never_undoable() {
    let (mut tree, region) = editor_with_privacy("a", TextInputFilter::Any, TextPrivacy::Password);
    tree.ime_input(ImeInput::Preedit {
        text: "日本".into(),
        cursor: None,
    });
    assert_eq!(tree.text_input_display(region.node).as_deref(), Some("•••"));
    assert_eq!(tree.text_input_cursor(region.node), Some(9));
    tree.ime_input(ImeInput::Commit("日本".into()));
    assert!(!tree.can_undo(region.node));
    tree.replace(field("a日本", TextInputFilter::Any).text_privacy(TextPrivacy::RevealedPassword));
    assert_eq!(
        tree.text_input_display(region.node).as_deref(),
        Some("a日本")
    );
    assert!(
        tree.semantic_tree(&[], 1.0).nodes[0]
            .semantics
            .value
            .is_none()
    );
}

#[test]
fn entering_password_policy_erases_existing_public_history() {
    let (mut tree, region) = editor("", TextInputFilter::Any);
    tree.paste_text(None, "secret");
    assert!(tree.can_undo(region.node));
    tree.replace(field("secret", TextInputFilter::Any).text_privacy(TextPrivacy::Password));
    assert!(!tree.can_undo(region.node));
    tree.replace(field("secret", TextInputFilter::Any));
    assert!(!tree.can_undo(region.node));
}
