use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Modifiers};
use argui_ui::{ClickEvent, Element, Role, UiEvent, UiEventKind, UiTree};
use argui_widgets::{List, ListState, shadcn};

fn event(key: Option<&str>, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        key.map(str::to_owned),
        kind,
    )
}
fn input(key: Key, modifiers: Modifiers) -> KeyInput {
    KeyInput {
        key,
        modifiers,
        state: KeyState::Pressed,
        repeat: false,
        text: None,
    }
}
fn key(key: Key, modifiers: Modifiers) -> UiEventKind {
    UiEventKind::KeyInput(input(key, modifiers))
}
fn click(modifiers: Modifiers) -> UiEventKind {
    UiEventKind::Click(ClickEvent::keyboard(input(Key::Enter, modifiers)))
}
fn command() -> Modifiers {
    Modifiers {
        control: true,
        ..Modifiers::default()
    }
}
fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

#[test]
fn click_toggle_ranges_and_mutations_preserve_controlled_selection() {
    let mut state = ListState::default();
    state.select(2, 10, true, Modifiers::default());
    state.select(5, 10, true, shift());
    assert_eq!(state.selected, (2..=5).collect());
    state.select(3, 10, true, command());
    assert_eq!(state.selected, [2, 4, 5].into());
    state.select(
        8,
        10,
        true,
        Modifiers {
            shift: true,
            ..command()
        },
    );
    assert_eq!(state.selected, (2..=8).collect());
    state.insert(4, 2);
    assert_eq!(state.active, Some(10));
    assert_eq!(state.selected, [2, 3, 6, 7, 8, 9, 10].into());
    state.remove(3..7, 8);
    assert_eq!(state.selected, [2, 3, 4, 5, 6].into());
    assert_eq!(state.active, Some(6));
    state.remove(6..8, 6);
    assert_eq!(state.active, Some(5));
    state.select(0, 6, true, shift());
    assert_eq!(state.selected, (0..=5).collect());
    state.remove(0..6, 0);
    assert!(state.selected.is_empty());
    assert_eq!(state.active, None);
    state.select(3, 0, true, Modifiers::default());
    assert_eq!(state, ListState::default());
}

#[test]
fn keyboard_selection_is_scoped_and_supports_focus_without_selection() {
    let mut state = ListState::default();
    for (key_input, modifiers, active, selected) in [
        (Key::ArrowDown, Modifiers::default(), 0, vec![0]),
        (Key::ArrowUp, Modifiers::default(), 0, vec![0]),
        (Key::End, shift(), 5, vec![0, 1, 2, 3, 4, 5]),
        (Key::Home, command(), 0, vec![0, 1, 2, 3, 4, 5]),
        (
            Key::Character(" ".into()),
            command(),
            0,
            vec![1, 2, 3, 4, 5],
        ),
        (Key::ArrowDown, Modifiers::default(), 1, vec![1]),
        (
            Key::Character("a".into()),
            command(),
            1,
            vec![0, 1, 2, 3, 4, 5],
        ),
    ] {
        state = List::new("list", 6)
            .selection(&state, true)
            .action(&event(Some("list"), key(key_input, modifiers)))
            .unwrap();
        assert_eq!(state.active, Some(active));
        assert_eq!(state.selected, selected.into_iter().collect());
    }
    let list = List::new("list", 6).selection(&state, true);
    for event in [
        event(None, click(Modifiers::default())),
        event(Some("other::row::0"), click(Modifiers::default())),
        event(Some("list::row::bad"), click(Modifiers::default())),
        event(Some("list::row::99"), click(Modifiers::default())),
        event(Some("list"), click(Modifiers::default())),
        event(Some("list"), key(Key::Escape, Modifiers::default())),
        event(Some("list"), UiEventKind::Focused),
        event(
            Some("list"),
            UiEventKind::KeyInput(KeyInput {
                state: KeyState::Released,
                ..input(Key::End, Modifiers::default())
            }),
        ),
    ] {
        assert_eq!(list.action(&event), None);
    }
    assert_eq!(
        List::new("list", 0).action(&event(Some("list"), key(Key::End, Modifiers::default()))),
        None
    );
    let clicked = list
        .action(&event(Some("list::row::3"), click(Modifiers::default())))
        .unwrap();
    assert_eq!(clicked.selected, [3].into());
}

#[test]
fn single_selection_and_semantics_report_the_full_set() {
    let mut state = ListState::default();
    state.select(1, 3, false, shift());
    state.select(2, 3, false, command());
    assert_eq!(state.selected, [2].into());
    let themes = shadcn(Color::WHITE);
    let root = List::new("items", 3)
        .label("Items")
        .selection(&state, true)
        .build(themes.resolve(ColorScheme::Light), |index| {
            Element::text(index.to_string())
        });
    assert_eq!(
        root.semantics.as_ref().unwrap().label.as_deref(),
        Some("Items")
    );
    assert!(root.semantics.as_ref().unwrap().state.multiselectable);
    let row = &root.children[2];
    assert_eq!(row.semantics.as_ref().unwrap().role, Role::Option);
    assert!(row.semantics.as_ref().unwrap().state.selected);
    assert_eq!(row.semantics.as_ref().unwrap().position_in_set, Some(3));
    assert_eq!(row.semantics.as_ref().unwrap().set_size, Some(3));
    assert!(row.interaction.as_ref().unwrap().focusable);
    assert!(!root.children[0].interaction.as_ref().unwrap().focusable);
}

#[test]
fn accessible_names_survive_row_decoration_and_enter_activates() {
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Light);
    let list = List::new("items", 3);
    let root = list.build(theme, |index| match index {
        0 => Element::text("Plain row"),
        1 => Element::text("Visual")
            .semantics(argui_ui::Semantics::new(Role::Text).label("Explicit label")),
        _ => Element::column([Element::text("Content")]),
    });
    assert_eq!(
        root.children[0]
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Plain row")
    );
    assert_eq!(
        root.children[1]
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Explicit label")
    );
    let state = list
        .action(&event(Some("items"), key(Key::Enter, Modifiers::default())))
        .unwrap();
    assert_eq!(state.selected, [0].into());
}
