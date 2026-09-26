use argui_core::{Key, KeyInput, KeyState, Modifiers};
use argui_schema::{NativeElementInput, NativeEventValue, SchemaError, SchemaValue, builtin};
use argui_ui::{
    Element, EventHandler, EventHandlerId, EventOwnerId, EventType, Position, UiEventKind, UiTree,
    length,
};

#[test]
fn key_binding_is_layout_neutral_and_activates_from_a_sibling() {
    let registry = builtin::registry().unwrap();
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(40), 0));
    let binding = registry
        .construct(
            builtin::KEY_BINDING,
            &NativeElementInput::new()
                .property(
                    builtin::SHORTCUT,
                    SchemaValue::String("Primary+Shift+K".into()),
                )
                .event(NativeEventValue::new(builtin::ACTIVATED, handler)),
        )
        .unwrap();
    assert_eq!(binding.style.position, Position::Absolute);
    assert_eq!(binding.style.size.width, length(0.0));
    assert_eq!(binding.style.size.height, length(0.0));
    assert!(binding.paint.quad.background.is_none());
    assert!(binding.children.is_empty());
    assert_eq!(binding.event_listeners.len(), 1);
    assert_eq!(binding.event_listeners[0].event, EventType::Key);

    let mut tree = UiTree::new(Element::container([Element::container([]), binding]));
    let target = tree.node_id_at(1).unwrap();
    let input = KeyInput {
        key: Key::Character("k".into()),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            shift: true,
            ..Modifiers::default()
        },
        repeat: false,
        text: None,
    };
    let deliveries = tree.event_deliveries(target, UiEventKind::KeyInput(input.clone()));
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].current_target(), tree.node_id_at(2).unwrap());
    assert_eq!(tree.keyboard_event(&input, &[]).events.len(), 1);
}

#[test]
fn disabled_binding_has_no_listener_and_invalid_chords_fail() {
    let registry = builtin::registry().unwrap();
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(41), 0));
    let disabled = registry
        .construct(
            builtin::KEY_BINDING,
            &NativeElementInput::new()
                .property(builtin::SHORTCUT, SchemaValue::String("Escape".into()))
                .property(builtin::ENABLED, SchemaValue::Bool(false))
                .event(NativeEventValue::new(builtin::ACTIVATED, handler)),
        )
        .unwrap();
    assert!(disabled.event_listeners.is_empty());

    for chord in ["", "Primary+Primary+K", "Shift+Bogus", "K+Shift", "F25"] {
        let error = registry
            .construct(
                builtin::KEY_BINDING,
                &NativeElementInput::new()
                    .property(builtin::SHORTCUT, SchemaValue::String(chord.into())),
            )
            .unwrap_err();
        assert!(matches!(error, SchemaError::Adapter(_)), "{chord}");
    }
}

#[test]
fn shortcut_spellings_cover_navigation_function_keys_and_modifier_aliases() {
    let registry = builtin::registry().unwrap();
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(42), 0));
    for chord in [
        "Esc",
        "Return",
        "Tab",
        "Space",
        "Backspace",
        "Delete",
        "Home",
        "End",
        "PageUp",
        "pageDown",
        "Left",
        "ArrowLeft",
        "Right",
        "ArrowRight",
        "Up",
        "ArrowUp",
        "Down",
        "ArrowDown",
        "ContextMenu",
        "F1",
        "f24",
        "A",
        "Ctrl+Shift+Alt+K",
        "Control+K",
        "Command+K",
        "Cmd+K",
        "Meta+K",
        "Option+K",
        "Primary+K",
    ] {
        let element = registry
            .construct(
                builtin::KEY_BINDING,
                &NativeElementInput::new()
                    .property(builtin::SHORTCUT, SchemaValue::String(chord.into()))
                    .event(NativeEventValue::new(builtin::ACTIVATED, handler)),
            )
            .unwrap_or_else(|error| panic!("{chord}: {error}"));
        assert_eq!(element.event_listeners.len(), 1, "{chord}");
    }
    for chord in [
        "Primary+",
        "Shift+Shift+K",
        "Alt+Option+K",
        "K+Alt",
        "K+J",
        "F0",
        "F999",
        "NotAKey",
        "Ctrl+Control+K",
        "Command+Cmd+K",
        "Meta+Primary+K",
        "K+Primary",
        "K+Ctrl",
        "K+Control",
        "K+Command",
        "K+Cmd",
        "K+Meta",
        "K+Option",
        "Escape+Enter",
    ] {
        assert!(
            registry
                .construct(
                    builtin::KEY_BINDING,
                    &NativeElementInput::new()
                        .property(builtin::SHORTCUT, SchemaValue::String(chord.into())),
                )
                .is_err(),
            "{chord}"
        );
    }
    let hidden = registry
        .construct(
            builtin::KEY_BINDING,
            &NativeElementInput::new()
                .property(builtin::SHORTCUT, SchemaValue::String("Escape".into()))
                .property(builtin::VISIBLE, SchemaValue::Bool(false))
                .event(NativeEventValue::new(builtin::ACTIVATED, handler)),
        )
        .unwrap();
    assert!(hidden.event_listeners.is_empty());
}
