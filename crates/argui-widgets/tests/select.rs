use argui_core::{Key, KeyInput, KeyState, Modifiers};
use argui_ui::{Element, Role, UiEvent, UiEventKind, UiTree, UserSelect};
use argui_widgets::{Select, SelectAction, SelectBehavior, SelectOption};

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(tree.node_ids()[0], Some(key.into()), kind)
}

fn key_state(key: Key, state: KeyState) -> UiEventKind {
    UiEventKind::KeyInput(KeyInput {
        key,
        state,
        modifiers: Modifiers::default(),
        repeat: false,
        text: None,
    })
}

fn key(key: Key) -> UiEventKind {
    key_state(key, KeyState::Pressed)
}

#[test]
fn select_navigation_skips_disabled_options_and_wraps() {
    let options = [
        SelectOption::new("Vulkan"),
        SelectOption::new("DX12").enabled(false),
        SelectOption::new("Metal"),
    ];
    let behavior = |highlighted| {
        SelectBehavior::new("backend", "Backend", options.iter().cloned(), None)
            .highlighted(highlighted)
    };
    assert_eq!(
        behavior(0).action(&event("backend", key(Key::ArrowDown))),
        Some(SelectAction::Highlight(2))
    );
    assert_eq!(
        behavior(2).action(&event("backend", key(Key::ArrowDown))),
        Some(SelectAction::Highlight(0))
    );
    assert_eq!(
        behavior(0).action(&event("backend::list", key(Key::ArrowUp))),
        Some(SelectAction::Highlight(2))
    );
    assert_eq!(
        behavior(0).action(&event("backend::option::0", key(Key::ArrowDown))),
        Some(SelectAction::Highlight(2))
    );
    assert_eq!(
        behavior(0).action(&event("backend", key(Key::Character("m".into())))),
        Some(SelectAction::Highlight(2))
    );
    assert_eq!(
        behavior(0).action(&event(
            "backend::option::1",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )),
        None
    );
    assert_eq!(
        behavior(0).action(&event(
            "backend",
            key_state(Key::ArrowDown, KeyState::Released),
        )),
        None
    );
    assert_eq!(
        SelectBehavior::new("backend", "Backend", [], None)
            .action(&event("backend", key(Key::ArrowDown))),
        None
    );
}

#[test]
fn open_select_is_an_anchored_trapped_listbox() {
    let theme = argui_widgets::shadcn(argui_core::Color::srgb(0.2, 0.5, 0.9));
    let element = Select::new(
        "backend",
        "Backend",
        [
            SelectOption::new("Vulkan"),
            SelectOption::new("Metal").enabled(false),
        ],
        Some(0),
    )
    .open(true)
    .highlighted(1)
    .build(theme.resolve(argui_core::ColorScheme::Dark));
    assert_eq!(element.children.len(), 2);
    assert_eq!(
        element.children[0].semantics.as_ref().unwrap().role,
        Role::Button
    );
    assert_eq!(element.children[0].user_select, UserSelect::None);
    assert_eq!(
        element.children[1].semantics.as_ref().unwrap().role,
        Role::ListBox
    );
    assert!(element.children[1].portal.is_some());
    assert!(element.children[1].focus_scope.is_some());
    assert!(
        element.children[1]
            .children
            .iter()
            .all(|option| option.user_select == UserSelect::None)
    );
}
