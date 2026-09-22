use super::*;

#[test]
fn pressed_pointer_move_filter_accepts_active_contacts_only() {
    let listener = EventListener::new(
        EventType::PointerMove,
        EventHandlerId::new(EventOwnerId(7), 0),
    )
    .filter(argui_ui::EventFilter::PressedPointerMove);
    let mut tree = UiTree::new(Element::container([]).on(listener));
    let node = tree.node_id_at(0).unwrap();
    let idle = PointerEvent::mouse(PointerPhase::Moved, Point::new(4.0, 5.0));
    let mut events = |pointer| tree.event_deliveries(node, UiEventKind::Pointer(pointer));

    assert!(events(idle).is_empty());
    assert_eq!(events(PointerEvent { buttons: 1, ..idle }).len(), 1);
    assert_eq!(
        events(PointerEvent {
            kind: argui_core::PointerKind::Touch,
            ..idle
        })
        .len(),
        1
    );
    assert_eq!(
        events(PointerEvent {
            kind: argui_core::PointerKind::Pen,
            pressure: Some(0.5),
            ..idle
        })
        .len(),
        1
    );
    assert!(
        events(PointerEvent {
            phase: PointerPhase::Pressed,
            buttons: 1,
            ..idle
        })
        .is_empty()
    );
}

mod shortcut {
    use super::*;

    #[test]
    fn typed_shortcut_filters_bubbled_key_events_and_preserves_cancellation() {
        let binding = EventListener::new(EventType::Key, EventHandlerId::new(EventOwnerId(9), 0))
            .shortcut(argui_ui::Shortcut::new(Key::Escape).alt());
        let mut tree = UiTree::new(Element::container([Element::container([])]).on(binding));
        let root = tree.node_id_at(0).unwrap();
        let child = tree.node_id_at(1).unwrap();
        let input = KeyInput {
            key: Key::Escape,
            state: KeyState::Pressed,
            modifiers: argui_core::Modifiers {
                alt: true,
                ..argui_core::Modifiers::default()
            },
            repeat: false,
            text: None,
        };

        let matched = tree.event_deliveries(child, UiEventKind::KeyInput(input.clone()));
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].target, child);
        assert_eq!(matched[0].current_target(), root);
        assert_eq!(matched[0].phase(), EventPhase::Bubble);
        assert!(matched[0].prevent_default());
        assert!(matched[0].default_prevented());

        for rejected in [
            KeyInput {
                repeat: true,
                ..input.clone()
            },
            KeyInput {
                state: KeyState::Released,
                ..input.clone()
            },
            KeyInput {
                modifiers: argui_core::Modifiers::default(),
                ..input.clone()
            },
            KeyInput {
                key: Key::Enter,
                ..input
            },
        ] {
            assert!(
                tree.event_deliveries(child, UiEventKind::KeyInput(rejected))
                    .is_empty()
            );
        }
    }

    #[test]
    fn global_shortcut_follows_local_propagation_and_obeys_default_prevention() {
        let local = EventListener::new(EventType::Key, EventHandlerId::new(EventOwnerId(10), 0));
        let global = EventListener::new(EventType::Key, EventHandlerId::new(EventOwnerId(11), 0))
            .shortcut(argui_ui::Shortcut::new(Key::Escape))
            .global_key();
        let mut tree = UiTree::new(Element::container([
            Element::container([]).on(local),
            Element::container([]).on(global),
        ]));
        let focused = tree.node_id_at(1).unwrap();
        let binding = tree.node_id_at(2).unwrap();
        let input = KeyInput {
            key: Key::Escape,
            state: KeyState::Pressed,
            modifiers: argui_core::Modifiers::default(),
            repeat: false,
            text: None,
        };

        let deliveries = tree.event_deliveries(focused, UiEventKind::KeyInput(input));
        assert_eq!(deliveries.len(), 2);
        assert_eq!(deliveries[0].current_target(), focused);
        assert_eq!(deliveries[1].current_target(), binding);
        assert!(deliveries[0].should_dispatch());
        assert!(deliveries[0].prevent_default());
        assert!(!deliveries[1].should_dispatch());
    }

    #[test]
    fn global_shortcut_reaches_unfocused_tree_and_respects_propagation_stop() {
        let local = EventListener::new(EventType::Key, EventHandlerId::new(EventOwnerId(12), 0));
        let global = EventListener::new(EventType::Key, EventHandlerId::new(EventOwnerId(13), 0))
            .shortcut(argui_ui::Shortcut::new(Key::Escape))
            .global_key();
        let mut tree = UiTree::new(Element::container([
            Element::container([]).on(local),
            Element::container([]).on(global),
        ]));
        let target = tree.node_id_at(1).unwrap();
        let input = KeyInput {
            key: Key::Escape,
            state: KeyState::Pressed,
            modifiers: argui_core::Modifiers::default(),
            repeat: false,
            text: None,
        };

        let deliveries = tree.event_deliveries(target, UiEventKind::KeyInput(input.clone()));
        assert_eq!(deliveries.len(), 2);
        assert!(deliveries[0].should_dispatch());
        deliveries[0].stop_propagation();
        assert!(!deliveries[1].should_dispatch());

        let update = tree.keyboard_event(&input, &[]);
        assert_eq!(update.events.len(), 1);
        assert_eq!(
            update.events[0].current_target(),
            tree.node_id_at(2).unwrap()
        );
    }

    #[test]
    fn active_modal_scope_blocks_global_shortcuts_behind_it() {
        use argui_ui::{FocusScope, InitialFocus};

        let global = |owner| {
            EventListener::new(EventType::Key, EventHandlerId::new(EventOwnerId(owner), 0))
                .shortcut(argui_ui::Shortcut::new(Key::Escape))
                .global_key()
        };
        let mut tree = UiTree::new(Element::container([
            Element::container([]).on(global(20)),
            Element::container([Element::container([]).on(global(21))])
                .focus_scope(FocusScope::modal(InitialFocus::First)),
        ]));
        let input = KeyInput {
            key: Key::Escape,
            state: KeyState::Pressed,
            modifiers: argui_core::Modifiers::default(),
            repeat: false,
            text: None,
        };
        let deliveries =
            tree.event_deliveries(tree.node_id_at(3).unwrap(), UiEventKind::KeyInput(input));
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].current_target(), tree.node_id_at(3).unwrap());
    }
}
