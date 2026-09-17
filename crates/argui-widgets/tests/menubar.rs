use argui_core::{Key, KeyInput, KeyState};
use argui_ui::{ActionId, ActionInvocation, ActionState, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Menu, MenuItem, Menubar, MenubarResponse};

#[test]
fn menubar_wraps_in_both_directions_and_opens_first_and_last_entries() {
    let menus: Vec<_> = ["file", "edit"]
        .into_iter()
        .map(|id| {
            Menu::new(
                id,
                id,
                false,
                vec![
                    MenuItem::new(
                        "first",
                        ActionInvocation::new(ActionId("first")),
                        ActionState::new("First"),
                    ),
                    MenuItem::new(
                        "last",
                        ActionInvocation::new(ActionId("last")),
                        ActionState::new("Last"),
                    ),
                ],
            )
        })
        .collect();
    let id = UiTree::new(Element::container([])).node_ids()[0];
    let event = |key| {
        UiEvent::new(
            id,
            Some("file".into()),
            UiEventKind::KeyInput(KeyInput {
                key,
                state: KeyState::Pressed,
                modifiers: Default::default(),
                repeat: false,
                text: None,
            }),
        )
    };
    let bar = Menubar::new("bar", "Menu", &menus);
    for key in [Key::ArrowLeft, Key::ArrowRight] {
        assert_eq!(
            bar.response(&event(key)),
            Some(MenubarResponse::Focus { key: "edit".into() })
        );
    }
    for (key, target) in [(Key::ArrowDown, "first"), (Key::ArrowUp, "last")] {
        assert_eq!(
            bar.response(&event(key)),
            Some(MenubarResponse::Open {
                key: "file".into(),
                focus: format!("file::item::{target}").into()
            })
        );
    }
}

#[test]
fn open_menubar_moves_into_next_menu_and_uses_its_rtl_direction() {
    let mut first = Menu::new(
        "first",
        "First",
        true,
        vec![MenuItem::new(
            "a",
            ActionInvocation::new(ActionId("a")),
            ActionState::new("A"),
        )],
    );
    let second = Menu::new(
        "second",
        "Second",
        false,
        vec![MenuItem::new(
            "b",
            ActionInvocation::new(ActionId("b")),
            ActionState::new("B"),
        )],
    );
    first.rtl = false;
    let menus = [first, second];
    let bar = Menubar::new("bar", "Bar", &menus)
        .active(Some("first"))
        .rtl(true);
    let event = UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some("first::item::a".into()),
        UiEventKind::KeyInput(KeyInput {
            key: Key::ArrowLeft,
            state: KeyState::Pressed,
            modifiers: Default::default(),
            repeat: false,
            text: None,
        }),
    );
    assert_eq!(
        bar.response(&event),
        Some(MenubarResponse::Open {
            key: "second".into(),
            focus: "second::item::b".into()
        })
    );
}
