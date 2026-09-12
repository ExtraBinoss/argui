use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState};
use argui_ui::{ClickEvent, InitialFocus, Role, UiEvent, UiEventKind, UiTree};
use argui_widgets::{AlertDialog, AlertDialogAction, Button, shadcn};

#[test]
fn confirmation_focuses_cancel_and_backdrop_cannot_approve_or_cancel() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Dark);
    let mut dialog = AlertDialog::new(
        "delete",
        "Delete file?",
        "This removes the selected file.",
        true,
        Button::new("open", "Delete", theme.button()).build(),
    );
    let tree = UiTree::new(dialog.clone().build(theme));
    assert!(tree.semantic_diagnostics().is_empty());
    assert!(
        tree.semantic_tree(&[], 1.0)
            .nodes
            .iter()
            .any(|node| node.semantics.role == Role::AlertDialog)
    );
    assert_eq!(
        dialog.clone().build(theme).children[1]
            .focus_scope
            .as_ref()
            .unwrap()
            .initial,
        Some(InitialFocus::Target("delete::close".into()))
    );
    let click = |key: &str| {
        UiEvent::new(
            tree.node_ids()[0],
            Some(key.into()),
            UiEventKind::Click(ClickEvent::accessibility()),
        )
    };
    assert_eq!(
        dialog.action(&click("delete::confirm")),
        Some(AlertDialogAction::Confirm)
    );
    assert_eq!(
        dialog.action(&click("delete::close")),
        Some(AlertDialogAction::Cancel)
    );
    assert_eq!(dialog.action(&click("delete::backdrop")), None);
    dialog.confirm_enabled = false;
    assert_eq!(dialog.action(&click("delete::confirm")), None);
    let escape = UiEvent::new(
        tree.node_ids()[0],
        Some("delete::panel".into()),
        UiEventKind::KeyInput(KeyInput {
            key: Key::Escape,
            state: KeyState::Pressed,
            repeat: false,
            modifiers: Default::default(),
            text: None,
        }),
    );
    assert_eq!(dialog.action(&escape), Some(AlertDialogAction::Cancel));
    dialog.open = false;
    assert_eq!(dialog.action(&escape), None);
    assert_eq!(
        dialog.action(&click("delete::trigger")),
        Some(AlertDialogAction::Open)
    );
    assert_eq!(dialog.build(theme).children.len(), 1);
}
