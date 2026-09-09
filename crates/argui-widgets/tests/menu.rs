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
                    i.to_string(),
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

#[test]
fn nested_menus_open_close_and_toggle_typed_entries() {
    use argui_ui::CheckedState;
    use argui_widgets::MenuItemKind;
    let nested = MenuItem::entry(
        "options",
        ActionState::new("Options"),
        MenuItemKind::Submenu(vec![
            MenuItem::entry(
                "check",
                ActionState::new("Check"),
                MenuItemKind::Checkbox(CheckedState::Mixed),
            ),
            MenuItem::entry(
                "radio",
                ActionState::new("Radio"),
                MenuItemKind::Radio {
                    group: "group".into(),
                    selected: false,
                },
            ),
        ]),
    );
    let mut menu = Menu::new("nested", "Nested", true, vec![nested]);
    let response = menu
        .response(&event("nested::item::options", keyboard(Key::ArrowRight)))
        .unwrap();
    assert_eq!(
        response,
        MenuResponse::Submenu {
            path: vec!["options".into()],
            focus: "nested::item::check".into()
        }
    );
    menu.path = vec!["options".into()];
    assert_eq!(
        menu.response(&event(
            "nested::item::check",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )),
        Some(MenuResponse::Checked {
            id: "check".into(),
            checked: CheckedState::Checked
        })
    );
    assert_eq!(
        menu.response(&event(
            "nested::item::radio",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )),
        Some(MenuResponse::Radio {
            id: "radio".into(),
            group: "group".into()
        })
    );
    assert_eq!(
        menu.response(&event("nested::item::check", keyboard(Key::Escape))),
        Some(MenuResponse::Submenu {
            path: vec![],
            focus: "nested::item::options".into()
        })
    );
    menu.rtl = true;
    assert!(matches!(
        menu.response(&event("nested::item::check", keyboard(Key::ArrowRight))),
        Some(MenuResponse::Submenu { .. })
    ));
    let themes = shadcn(Color::WHITE);
    let tree = UiTree::new(menu.build(
        Element::text("Open"),
        themes.resolve(argui_core::ColorScheme::Light),
    ));
    let semantics = tree.semantic_tree(&[], 1.0);
    assert!(
        semantics
            .nodes
            .iter()
            .any(|node| node.semantics.role == Role::MenuItemCheckBox
                && node.semantics.state.checked == Some(CheckedState::Mixed))
    );
}

#[path = "menu/intent.rs"]
mod intent;

#[test]
fn arrow_keys_open_edges_and_tab_dismisses_without_consuming_navigation() {
    for (key, id) in [(Key::ArrowDown, "0"), (Key::ArrowUp, "2")] {
        assert_eq!(
            menu(false).response(&event("commands", keyboard(key))),
            Some(MenuResponse::Open {
                focus: format!("commands::item::{id}").into()
            })
        );
    }
    let tab = event("commands::item::2", keyboard(Key::Tab));
    assert_eq!(menu(true).response(&tab), Some(MenuResponse::Close));
    assert!(!tab.default_prevented());
}

#[test]
fn hover_resolves_groups_and_rejects_hidden_disabled_or_missing_entries() {
    use argui_core::{Point, PointerEvent, PointerPhase};
    use argui_widgets::{MenuIntent, MenuItemKind};
    use std::time::Duration;
    let mut menu = Menu::new(
        "menu",
        "Menu",
        true,
        vec![MenuItem::entry(
            "group",
            ActionState::new("Group"),
            MenuItemKind::Group(vec![
                MenuItem::entry(
                    "disabled",
                    ActionState::new("Disabled").enabled(false),
                    MenuItemKind::Checkbox(argui_ui::CheckedState::Unchecked),
                ),
                MenuItem::entry(
                    "sub",
                    ActionState::new("Sub"),
                    MenuItemKind::Submenu(vec![MenuItem::entry(
                        "child",
                        ActionState::new("Child"),
                        MenuItemKind::Checkbox(argui_ui::CheckedState::Unchecked),
                    )]),
                ),
                MenuItem::entry(
                    "empty",
                    ActionState::new("Empty"),
                    MenuItemKind::Submenu(vec![]),
                ),
            ]),
        )],
    );
    for id in ["disabled", "child", "empty", "missing"] {
        assert!(menu.hover_response(id).is_none());
    }
    let mut intent = MenuIntent::default();
    let delay = Duration::from_millis(200);
    for (target, phase, expected) in [
        ("other", PointerPhase::Entered, false),
        ("menu::item::sub", PointerPhase::Moved, false),
        ("menu::item::disabled", PointerPhase::Entered, false),
        ("menu::item::sub", PointerPhase::Entered, true),
    ] {
        assert_eq!(
            menu.schedule_hover(
                &event(
                    target,
                    UiEventKind::Pointer(PointerEvent::mouse(phase, Point::default()))
                ),
                &mut intent,
                Duration::ZERO,
                delay
            ),
            expected
        );
    }
    assert_eq!(intent.take_due(delay), Some("sub".into()));
    let Some(MenuResponse::Submenu { path, .. }) = menu.hover_response("sub") else {
        panic!("submenu");
    };
    menu.path = path;
    assert!(menu.hover_response("child").is_some());
    assert_eq!(
        menu.response(&event("menu::item::sub", keyboard(Key::ArrowLeft))),
        None
    );
    assert_eq!(
        menu.response(&event("menu::item::sub", keyboard(Key::Escape))),
        Some(MenuResponse::Close)
    );
    assert_eq!(
        menu.response(&event("menu::item::child", keyboard(Key::ArrowLeft))),
        Some(MenuResponse::Submenu {
            path: vec![],
            focus: "menu::item::sub".into()
        })
    );
    let themes = shadcn(Color::WHITE);
    assert!(
        UiTree::new(menu.build(
            Element::text("Open"),
            themes.resolve(argui_core::ColorScheme::Light)
        ))
        .semantic_diagnostics()
        .is_empty()
    );
    menu.open = false;
    assert!(menu.hover_response("sub").is_none());
}
