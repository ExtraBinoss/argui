use argui_core::{Key, KeyInput, KeyState, Modifiers};
use argui_paint::{PaintStyle, QuadStyle};
use argui_ui::{Element, Role, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Button, Dialog, DialogAction, DialogBehavior, shadcn};

fn event(key: Option<&str>, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(tree.node_ids()[0], key.map(str::to_owned), kind)
}

#[test]
fn dialog_actions_and_modal_tree_are_explicit() {
    let behavior = DialogBehavior::new("confirm", "Confirm", true);
    assert_eq!(
        behavior.action(&event(Some("confirm::trigger"), UiEventKind::Clicked)),
        Some(DialogAction::Open)
    );
    assert_eq!(
        behavior.action(&event(Some("confirm::close"), UiEventKind::Clicked)),
        Some(DialogAction::Close)
    );
    assert_eq!(
        behavior.action(&event(Some("confirm::backdrop"), UiEventKind::Clicked)),
        Some(DialogAction::Close)
    );
    assert_eq!(
        behavior.action(&event(
            Some("confirm::panel"),
            UiEventKind::KeyInput(KeyInput {
                key: Key::Escape,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            })
        )),
        Some(DialogAction::Close)
    );
    assert_eq!(
        behavior.action(&event(Some("confirm::panel"), UiEventKind::Clicked)),
        None
    );
    assert_eq!(
        behavior.action(&event(
            Some("confirm::panel"),
            UiEventKind::KeyInput(KeyInput {
                key: Key::Escape,
                state: KeyState::Released,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            })
        )),
        None
    );
    assert_eq!(
        behavior.action(&event(
            Some("confirm::panel"),
            UiEventKind::KeyInput(KeyInput {
                key: Key::Enter,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            })
        )),
        None
    );
    let themes = shadcn(argui_core::Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let dialog = Dialog::new(
        "confirm",
        "Confirm",
        true,
        Button::new("ignored", "Open", theme.button.clone()).build(),
        Element::text("content"),
    )
    .backdrop(argui_core::Color::srgba(0.1, 0.2, 0.3, 0.4))
    .backdrop_blur(0.0)
    .panel_paint(PaintStyle::new(QuadStyle::solid(argui_core::Color::BLACK)))
    .panel_width(320.0)
    .viewport_margin(18.0)
    .build(theme);
    assert_eq!(dialog.children.len(), 2);
    assert!(dialog.children[1].focus_scope.is_some());
    assert_eq!(
        dialog.children[1].portal.as_ref().unwrap().layer,
        argui_ui::WindowLayer::Modal
    );
    assert_eq!(
        dialog.children[1].children[1]
            .semantics
            .as_ref()
            .unwrap()
            .role,
        Role::Dialog
    );
}

#[test]
fn dialog_default_surface_uses_the_theme_blurs() {
    let themes = shadcn(argui_core::Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let dialog = Dialog::new(
        "default-dialog",
        "Default dialog",
        true,
        Element::text("Open"),
        Element::text("Content"),
    )
    .build(theme);
    let overlay = &dialog.children[1];

    assert!(
        !overlay.children[0]
            .layer
            .as_ref()
            .unwrap()
            .backdrop_filters
            .is_empty()
    );
    assert!(overlay.children[1].layer.is_some());
    assert!(overlay.children[1].paint.quad.background.is_some());
}
