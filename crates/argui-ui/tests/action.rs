use argui_core::{Key, KeyInput, KeyState, Modifiers};
use argui_ui::{
    ActionBinding, ActionError, ActionId, ActionInvocation, ActionScope, ActionState, ClickEvent,
    Element, EventHandlerId, EventListener, EventOwnerId, EventType, FocusScope, InitialFocus,
    Interaction, Shortcut, UiEventKind, UiTree,
};

const SAVE: ActionId = ActionId("app.save");

#[test]
fn disabled_menu_item_cannot_bypass_availability_with_an_explicit_enabled_origin() {
    let mut tree = nested(true);
    let origin = tree.node_ids()[2];
    let menu = tree.node_ids()[3];
    let mut root = tree.root().clone();
    root.children[1] = root.children[1]
        .clone()
        .action_from(ActionInvocation::new(SAVE).at(origin))
        .interaction(Interaction::default().enabled(false));
    tree.replace(root);
    assert!(
        tree.action_state(ActionInvocation::new(SAVE).at(origin))
            .unwrap()
            .enabled
    );
    assert!(
        tree.event_deliveries(menu, UiEventKind::Click(ClickEvent::accessibility()))
            .is_empty()
    );
}

#[test]
fn builtins_keep_document_select_all_copy_and_override_routing_without_editor_focus() {
    let mut tree = UiTree::new(Element::column([
        Element::text("Hello"),
        Element::text("world"),
    ]));
    assert!(
        tree.action_state(ActionInvocation::new(ActionId::SELECT_ALL))
            .unwrap()
            .enabled
    );
    tree.invoke_action(ActionInvocation::new(ActionId::SELECT_ALL));
    assert!(
        tree.invoke_action(ActionInvocation::new(ActionId::COPY))
            .clipboard
            .is_some()
    );
    for id in [
        ActionId::CUT,
        ActionId::PASTE,
        ActionId::UNDO,
        ActionId::REDO,
    ] {
        assert!(
            !tree
                .action_state(ActionInvocation::new(id))
                .unwrap()
                .enabled
        );
    }
    tree.replace(
        Element::container([])
            .action_scope(ActionScope::new([binding(ActionId::COPY, 7, true)]).unwrap()),
    );
    let update = tree.apply_command(argui_ui::UiCommand::Selection {
        target: None,
        command: argui_ui::SelectionCommand::Copy,
    });
    assert_eq!(update.events[0].current_handler().unwrap().slot(), 7);
}
fn binding(id: ActionId, slot: u32, enabled: bool) -> ActionBinding {
    ActionBinding {
        id,
        state: ActionState::new("Save")
            .enabled(enabled)
            .shortcut(Shortcut::primary("s")),
        listener: EventListener::new(
            EventType::Action,
            EventHandlerId::new(EventOwnerId(1), slot),
        ),
    }
}
fn scope(slot: u32, enabled: bool) -> ActionScope {
    ActionScope::new([binding(SAVE, slot, enabled)]).unwrap()
}
fn nested(enabled: bool) -> UiTree {
    UiTree::new(
        Element::column([
            Element::column([Element::container([]).keyed("editor")])
                .action_scope(scope(2, enabled)),
            Element::container([]).keyed("menu"),
        ])
        .action_scope(scope(1, true)),
    )
}
fn pressed(key: &str) -> KeyInput {
    KeyInput {
        key: Key::Character(key.into()),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Default::default()
        },
        repeat: false,
        text: None,
    }
}

#[test]
fn scopes_reject_duplicate_identity_and_shortcut_conflicts_even_when_disabled() {
    assert_eq!(
        ActionScope::new([binding(SAVE, 0, true), binding(SAVE, 1, false)]).unwrap_err(),
        ActionError::DuplicateId(SAVE)
    );
    assert!(matches!(
        ActionScope::new([binding(SAVE, 0, true), binding(ActionId("other"), 1, false)]),
        Err(ActionError::ShortcutConflict(_))
    ));
    assert_eq!(
        Shortcut::primary("k").label(),
        if cfg!(target_os = "macos") {
            "⌘+K"
        } else {
            "Ctrl+K"
        }
    );
    assert!(Shortcut::primary("k").matches(&pressed("K")));
    assert!(!Shortcut::primary("k").shift().matches(&pressed("k")));
    let mut input = pressed("k");
    input.repeat = true;
    assert!(!Shortcut::primary("k").matches(&input));
    input.repeat = false;
    input.state = KeyState::Released;
    assert!(!Shortcut::primary("k").matches(&input));
}

#[test]
fn nearest_scope_wins_and_disabled_local_command_blocks_parent() {
    let mut tree = nested(true);
    let editor = tree.node_ids()[2];
    let invoke = ActionInvocation::new(SAVE).at(editor);
    let events = tree.invoke_action(invoke).events;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].current_handler().unwrap().slot(), 2);
    let root = nested(false).root().clone();
    tree.replace(root);
    assert!(!tree.action_state(invoke).unwrap().enabled);
    assert!(tree.invoke_action(invoke).events.is_empty());
    let root = tree.node_ids()[0];
    assert_eq!(
        tree.invoke_action(ActionInvocation::new(SAVE).at(root))
            .events[0]
            .current_handler()
            .unwrap()
            .slot(),
        1
    );
}

#[test]
fn captured_origin_is_used_from_a_menu_and_removed_origins_fail_closed() {
    let mut tree = nested(true);
    let origin = tree.node_ids()[2];
    let invoke = ActionInvocation::new(SAVE).at(origin);
    let menu = tree.node_ids()[3];
    let mut next = tree.root().clone();
    next.children[1] = next.children[1].clone().action_from(invoke);
    tree.replace(next);
    let click = tree.event_deliveries(menu, UiEventKind::Click(ClickEvent::accessibility()));
    assert!(click[0].should_dispatch());
    assert!(!click[0].should_dispatch());
    let UiEventKind::Action(invocation) = click[0].kind else {
        panic!("action default");
    };
    assert_eq!(
        tree.invoke_action(invocation).events[0]
            .current_handler()
            .unwrap()
            .slot(),
        2
    );
    tree.replace(Element::column([Element::container([])]).action_scope(scope(1, true)));
    assert_eq!(tree.action_state(invoke), Err(ActionError::StaleOrigin));
    assert!(tree.invoke_action(invoke).events.is_empty());
}

#[test]
fn prevent_default_cancels_action_but_stop_propagation_does_not_duplicate_it() {
    let mut tree = UiTree::new(
        Element::container([])
            .action(SAVE)
            .action_scope(scope(1, true))
            .on(EventListener::new(
                EventType::Click,
                EventHandlerId::new(EventOwnerId(1), 2),
            )),
    );
    let node = tree.node_ids()[0];
    let events = tree.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility()));
    assert_eq!(events.len(), 2);
    assert!(events[0].prevent_default());
    assert!(!events[1].should_dispatch());
    let events = tree.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility()));
    events[0].stop_propagation();
    assert!(events[1].should_dispatch());
}

#[test]
fn keyboard_without_focus_can_use_window_actions_but_ignores_other_keys() {
    let mut tree = UiTree::new(Element::container([]).action_scope(scope(1, true)));
    let events = tree.keyboard_event(&pressed("s"), &[]).events;
    assert_eq!(events.len(), 1);
    assert!(events[0].should_dispatch());
    assert!(events[0].default_prevented());
    let UiEventKind::Action(invocation) = events[0].kind else {
        panic!("shortcut action");
    };
    assert_eq!(tree.invoke_action(invocation).events.len(), 1);
    assert!(tree.keyboard_event(&pressed("q"), &[]).events.is_empty());
}

#[test]
fn modal_stops_resolution_and_blocks_stale_outside_menu_origins() {
    let mut tree = UiTree::new(
        Element::column([
            Element::container([]).keyed("outside"),
            Element::column([Element::container([])])
                .focus_scope(FocusScope::modal(InitialFocus::First)),
        ])
        .action_scope(scope(1, true)),
    );
    let outside = tree.node_ids()[1];
    let inside = tree.node_ids()[3];
    assert!(
        tree.invoke_action(ActionInvocation::new(SAVE).at(outside))
            .events
            .is_empty()
    );
    assert!(
        tree.invoke_action(ActionInvocation::new(SAVE).at(inside))
            .events
            .is_empty()
    );
    let mut next = tree.root().clone();
    next.children[1] = next.children[1].clone().action_scope(scope(3, true));
    tree.replace(next);
    assert_eq!(
        tree.invoke_action(ActionInvocation::new(SAVE).at(inside))
            .events[0]
            .current_handler()
            .unwrap()
            .slot(),
        3
    );
}

#[test]
fn hidden_and_disabled_origins_never_dispatch() {
    for hidden in [false, true] {
        let child = Element::container([])
            .interaction(Interaction::default().enabled(hidden))
            .display(if hidden {
                argui_ui::Display::None
            } else {
                argui_ui::Display::Flex
            });
        let mut tree = UiTree::new(Element::column([child]).action_scope(scope(1, true)));
        assert!(
            tree.invoke_action(ActionInvocation::new(SAVE).at(tree.node_ids()[1]))
                .events
                .is_empty()
        );
    }
}
