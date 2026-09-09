use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Modifiers, Point};
use argui_ui::{
    ActionId, ActionInvocation, ActionState, Element, PortalTarget, UiEvent, UiEventKind, UiTree,
};
use argui_widgets::{ContextMenu, Menu, MenuItem, shadcn};

#[test]
fn pointer_and_keyboard_opening_use_the_same_menu_with_different_anchors() {
    let mut menu = ContextMenu {
        menu: Menu::new(
            "context",
            "Context",
            true,
            vec![MenuItem::new(
                "action",
                ActionInvocation::new(ActionId("action")),
                ActionState::new("Action"),
            )],
        ),
        position: Some(Point::new(80.0, 90.0)),
    };
    let id = UiTree::new(Element::container([])).node_ids()[0];
    let keyboard = UiEvent::new(
        id,
        Some("context".into()),
        UiEventKind::KeyInput(KeyInput {
            key: Key::Function(10),
            state: KeyState::Pressed,
            modifiers: Modifiers {
                shift: true,
                ..Default::default()
            },
            repeat: false,
            text: None,
        }),
    );
    assert_eq!(menu.open_action(&keyboard), Some(None));
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Light);
    let built = menu.build(Element::text("Target"), theme);
    assert!(matches!(
        built.children[1].portal.as_ref().unwrap().target,
        PortalTarget::Rect { .. }
    ));
    menu.position = None;
    let built = menu.build(Element::text("Target"), theme);
    assert!(matches!(
        built.children[1].portal.as_ref().unwrap().target,
        PortalTarget::Anchor(_)
    ));
}
