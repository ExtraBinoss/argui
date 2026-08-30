use argui_core::{Key, KeyInput, KeyState, Modifiers};
use argui_ui::{Element, Role, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Button, Dialog, DialogAction, shadcn};

fn event(key: Option<&str>, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent {
        target: tree.node_ids()[0],
        key: key.map(str::to_owned),
        kind,
    }
}

#[test]
fn dialog_actions_and_modal_tree_are_explicit() {
    assert_eq!(
        Dialog::action(
            "confirm",
            &event(Some("confirm::trigger"), UiEventKind::Clicked)
        ),
        Some(DialogAction::Open)
    );
    assert_eq!(
        Dialog::action(
            "confirm",
            &event(Some("confirm::close"), UiEventKind::Clicked)
        ),
        Some(DialogAction::Close)
    );
    assert_eq!(
        Dialog::action(
            "confirm",
            &event(Some("confirm::backdrop"), UiEventKind::Clicked)
        ),
        Some(DialogAction::Close)
    );
    assert_eq!(
        Dialog::action(
            "confirm",
            &event(
                Some("confirm::panel"),
                UiEventKind::KeyInput(KeyInput {
                    key: Key::Escape,
                    state: KeyState::Pressed,
                    modifiers: Modifiers::default(),
                    repeat: false,
                    text: None,
                })
            )
        ),
        Some(DialogAction::Close)
    );
    let themes = shadcn(argui_core::Color::rgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let dialog = Dialog::new(
        "confirm",
        "Confirm",
        true,
        Button::new("ignored", "Open", theme.button.clone()).build(),
        Element::text("content"),
    )
    .build(theme);
    assert_eq!(dialog.children.len(), 2);
    assert!(dialog.children[1].focus_scope.is_some());
    assert_eq!(dialog.children[1].z_index, 2_000);
    assert_eq!(
        dialog.children[1].children[1]
            .semantics
            .as_ref()
            .unwrap()
            .role,
        Role::Dialog
    );
}
