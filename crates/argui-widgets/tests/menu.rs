use argui_core::{Color, Key, KeyInput, KeyState};
use argui_ui::{
    ActionId, ActionInvocation, ActionState, Element, FocusTarget, Role, Shortcut, UiEvent,
    UiEventKind, UiTree,
};
use argui_widgets::{Menu, MenuItem, MenuResponse, shadcn};

fn menu(open: bool) -> Menu {
    Menu::new(
        "commands",
        "Commands",
        open,
        (0..3)
            .map(|i| {
                MenuItem::new(
                    ActionInvocation::new(ActionId("save")),
                    ActionState::new(format!("Item {i}"))
                        .enabled(i != 1)
                        .shortcut(Shortcut::primary("s")),
                )
            })
            .collect(),
    )
}
fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}
fn keyboard(key: Key) -> UiEventKind {
    UiEventKind::KeyInput(KeyInput {
        key,
        state: KeyState::Pressed,
        modifiers: Default::default(),
        repeat: false,
        text: None,
    })
}

#[test]
fn menu_uses_widget_buttons_action_invocations_and_semantic_disabled_items() {
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let root = menu(true).build(Element::text("Open"), theme);
    let tree = UiTree::new(root);
    let items = tree
        .node_ids()
        .iter()
        .enumerate()
        .filter_map(|(i, _)| tree.element_at(i))
        .filter(|element| {
            element
                .semantics
                .as_ref()
                .is_some_and(|semantics| semantics.role == Role::MenuItem)
        })
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 3);
    assert!(!items[1].interaction.as_ref().unwrap().enabled);
    assert_eq!(
        items[0].action,
        Some(ActionInvocation::new(ActionId("save")))
    );
    assert_eq!(
        menu(false)
            .build(Element::text("Open"), theme)
            .children
            .len(),
        1
    );
}

#[test]
fn menu_keyboard_wraps_skips_disabled_and_handles_dismissal() {
    let menu = menu(true);
    for (target, key, expected) in [
        ("commands::item::0", Key::ArrowDown, "commands::item::2"),
        ("commands::item::2", Key::ArrowDown, "commands::item::0"),
        ("commands::item::0", Key::ArrowUp, "commands::item::2"),
        ("commands::item::2", Key::Home, "commands::item::0"),
        ("commands::item::0", Key::End, "commands::item::2"),
    ] {
        assert_eq!(
            menu.response(&event(target, keyboard(key))),
            Some(MenuResponse::Focus(FocusTarget::Key(expected.into())))
        );
    }
    assert_eq!(
        menu.response(&event("commands::item::0", keyboard(Key::Escape))),
        Some(MenuResponse::Close)
    );
    assert_eq!(
        menu.response(&event(
            "commands",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )),
        Some(MenuResponse::Toggle)
    );
    assert_eq!(
        menu.response(&event(
            "commands::item::0",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )),
        Some(MenuResponse::Close)
    );
    assert!(
        menu.response(&event("commands::item::0", keyboard(Key::Other)))
            .is_none()
    );
    assert!(
        Menu::new("empty", "Empty", true, Vec::new())
            .response(&event("empty", keyboard(Key::ArrowDown)))
            .is_none()
    );
}
