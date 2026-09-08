use argui_core::{Key, KeyInput, KeyState, Modifiers};
use argui_ui::{Element, Role, UiEvent, UiEventKind, UiTree, UserSelect};
use argui_widgets::{Select, SelectAction, SelectBehavior, SelectOption, SelectPart};

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(tree.node_ids()[0], Some(key.into()), kind)
}

#[test]
fn animated_select_exit_is_not_a_focus_trap_or_click_target() {
    let palette = argui_widgets::shadcn(argui_core::Color::WHITE);
    let theme = palette.resolve(argui_core::ColorScheme::Dark);
    let mut presence = argui_widgets::Presence::default();
    presence.set_open(true, true);
    presence.set_open(false, false);
    let build = |presence: &argui_widgets::Presence| {
        Select::new("animated", "Choose", [SelectOption::new("First")], Some(0))
            .presence(presence)
            .build(theme)
    };
    let closing = build(&presence);
    let list = &closing.children[1];
    assert!(list.focus_scope.is_none());
    assert_eq!(list.hit_test.pointer_events, argui_ui::PointerEvents::None);
    assert!(!list.children[0].interaction.as_ref().unwrap().enabled);
    presence.set_open(true, false);
    let reopened = build(&presence);
    assert!(reopened.children[1].focus_scope.is_some());
    assert!(
        reopened.children[1].children[0]
            .interaction
            .as_ref()
            .unwrap()
            .enabled
    );
    presence.set_open(false, true);
    assert_eq!(build(&presence).children.len(), 1);
}

#[test]
fn outside_pointer_does_not_close_an_unrelated_select() {
    let select =
        SelectBehavior::new("select", "Choose", [SelectOption::new("First")], Some(0)).open(true);
    let outside = UiEventKind::PointerOutside(argui_core::PointerEvent::mouse(
        argui_core::PointerPhase::Pressed,
        argui_core::Point::default(),
    ));
    assert_eq!(select.action(&event("other::list", outside.clone())), None);
    assert_eq!(
        select.action(&event(&select.list_key(), outside)),
        Some(SelectAction::Close)
    );
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

#[test]
fn closed_select_falls_back_to_label_and_keeps_trigger_content() {
    let themes = argui_widgets::shadcn(argui_core::Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Light);
    let trailing = Element::text("⌄").keyed("arrow");
    let element = Select::new(
        "invalid-selection",
        "Choose a backend",
        [SelectOption::new("Vulkan")],
        Some(usize::MAX),
    )
    .trailing(trailing.clone())
    .build(theme);
    assert_eq!(element.children.len(), 1);
    let trigger = &element.children[0];
    assert_eq!(trigger.key.as_deref(), Some("invalid-selection"));
    assert_eq!(trigger.children.len(), 2);
    assert_eq!(trigger.children[0].children.len(), 0);
    assert!(matches!(
        &trigger.children[0].kind,
        argui_ui::ElementKind::Text { content, .. } if content.as_str() == "Choose a backend"
    ));
    assert_eq!(trigger.children[1], trailing);
    assert_eq!(
        trigger.semantics.as_ref().unwrap().state.expanded,
        Some(false)
    );
}

#[test]
fn select_behavior_decorates_disabled_and_invalid_options_without_activation() {
    let options = [
        SelectOption::new("Alpha"),
        SelectOption::new("Beta").enabled(false),
    ];
    let behavior = SelectBehavior::new("mode", "Mode", options, Some(0))
        .open(true)
        .highlighted(0);
    let trigger = behavior.decorate(SelectPart::Trigger, Element::container([]));
    assert_eq!(trigger.key.as_deref(), Some("mode"));
    assert_eq!(
        trigger.semantics.as_ref().unwrap().state.expanded,
        Some(true)
    );
    assert!(trigger.interaction.as_ref().unwrap().focusable);
    let value = behavior.decorate(SelectPart::Value, Element::text("Mode"));
    assert!(value.semantic_hidden);
    let list = behavior.decorate(SelectPart::List, Element::container([]));
    assert_eq!(list.key.as_deref(), Some("mode::list"));
    assert_eq!(list.semantics.as_ref().unwrap().role, Role::ListBox);
    assert_eq!(
        list.semantics.as_ref().unwrap().orientation,
        Some(argui_ui::Orientation::Vertical)
    );

    let selected = behavior.decorate(SelectPart::Option(0), Element::container([]));
    assert!(selected.interaction.as_ref().unwrap().enabled);
    assert!(selected.semantics.as_ref().unwrap().state.selected);
    assert_eq!(
        selected.semantics.as_ref().unwrap().position_in_set,
        Some(1)
    );
    let disabled = behavior.decorate(SelectPart::Option(1), Element::container([]));
    assert!(!disabled.interaction.as_ref().unwrap().enabled);
    assert_eq!(
        disabled.interaction.as_ref().unwrap().cursor,
        argui_ui::CursorIcon::NotAllowed
    );
    assert!(disabled.semantics.as_ref().unwrap().state.disabled);
    let invalid = behavior.decorate(SelectPart::Option(10), Element::container([]));
    assert!(!invalid.interaction.as_ref().unwrap().focusable);
    assert_eq!(
        invalid.semantics.as_ref().unwrap().label.as_deref(),
        Some("")
    );
    assert_eq!(
        invalid.semantics.as_ref().unwrap().position_in_set,
        Some(11)
    );

    assert_eq!(
        behavior.action(&event(
            "mode::option::0",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )),
        Some(SelectAction::Select(0))
    );
    assert_eq!(
        behavior.action(&event(
            "mode::option::1",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )),
        None
    );
    assert_eq!(
        behavior.action(&event(
            "mode::option::not-a-number",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )),
        None
    );
    assert_eq!(
        behavior.action(&event(
            "mode::option::0",
            UiEventKind::PointerOutside(argui_core::PointerEvent::mouse(
                argui_core::PointerPhase::Pressed,
                argui_core::Point::default(),
            )),
        )),
        None
    );
}

#[test]
fn select_keyboard_actions_cover_edges_queries_and_unrelated_targets() {
    let options = [
        SelectOption::new("Alpha"),
        SelectOption::new("Beta").enabled(false),
        SelectOption::new("Gamma"),
    ];
    let behavior = |highlighted| {
        SelectBehavior::new("mode", "Mode", options.iter().cloned(), None)
            .open(true)
            .highlighted(highlighted)
    };
    assert_eq!(
        behavior(0).action(&event("mode", key(Key::Escape))),
        Some(SelectAction::Close)
    );
    assert_eq!(
        behavior(0).action(&event("mode::list", key(Key::Enter))),
        Some(SelectAction::Select(0))
    );
    assert_eq!(
        behavior(0).action(&event("mode::list", key(Key::Home))),
        Some(SelectAction::Highlight(0))
    );
    assert_eq!(
        behavior(0).action(&event("mode::list", key(Key::End))),
        Some(SelectAction::Highlight(2))
    );
    assert_eq!(
        behavior(2).action(&event("mode::option::2", key(Key::ArrowUp))),
        Some(SelectAction::Highlight(0))
    );
    assert_eq!(
        behavior(0).action(&event("mode::option::0", key(Key::Character("g".into())),)),
        Some(SelectAction::Highlight(2))
    );
    assert_eq!(
        behavior(0).action(&event("mode::option::0", key(Key::Character("z".into())),)),
        None
    );
    assert_eq!(
        behavior(0).action(&event("other", key(Key::ArrowDown))),
        None
    );
    assert_eq!(
        behavior(0).action(&event(
            "mode",
            key_state(Key::ArrowDown, KeyState::Released),
        )),
        None
    );

    let all_disabled = SelectBehavior::new(
        "disabled",
        "Disabled",
        [SelectOption::new("Nope").enabled(false)],
        None,
    )
    .open(true);
    assert_eq!(
        all_disabled.action(&event("disabled", key(Key::Home))),
        None
    );
    assert_eq!(
        all_disabled.action(&event("disabled", key(Key::ArrowDown))),
        None
    );
}

#[test]
fn select_can_disable_the_theme_overlay_blur() {
    let themes = argui_widgets::shadcn(argui_core::Color::srgb(0.2, 0.5, 0.9));
    let mut theme = themes.resolve(argui_core::ColorScheme::Dark).clone();
    theme.overlay_blur = 0.0;
    let select = Select::new("plain", "Plain", [SelectOption::new("First")], Some(0))
        .open(true)
        .build(&theme);
    let layer = select.children[1].layer.as_ref().unwrap();
    assert!(layer.backdrop_filters.is_empty());
    assert_eq!(layer.shadows, theme.overlay_shadows);
}
