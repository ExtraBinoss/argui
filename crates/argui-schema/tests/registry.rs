use argui_core::Color;
use argui_schema::{
    EventId, EventSchema, NativeElementInput, NativeEventValue, NativeSchema, NativeSlotValue,
    NativeTypeId, PropertyId, PropertySchema, SchemaError, SchemaRegistry, SchemaValue, SlotArity,
    SlotId, SlotSchema, StylePartId, StylePartSchema, ValueType, VariantId, VariantSchema, builtin,
};
use argui_ui::{
    Element, ElementKind, EventHandler, EventHandlerId, EventOwnerId, EventType, length,
};

#[test]
fn generic_ids_construct_native_elements_without_name_dispatch() {
    let registry = builtin::registry().unwrap();
    let child = registry
        .construct(
            builtin::TEXT,
            &NativeElementInput::new().property(
                builtin::CONTENT,
                SchemaValue::String("schema driven".into()),
            ),
        )
        .unwrap();
    let root = registry
        .construct(
            builtin::COLUMN,
            &NativeElementInput::new()
                .property(builtin::GAP, SchemaValue::Float(12.0))
                .property(builtin::BACKGROUND, SchemaValue::Color(Color::BLACK))
                .slot(NativeSlotValue::new(builtin::CHILDREN, [child])),
        )
        .unwrap();

    assert_eq!(root.children.len(), 1);
    assert_eq!(root.style.gap.width, length(12.0));
    assert!(matches!(root.children[0].kind, ElementKind::Text { .. }));
    assert_eq!(registry.schema_named("Column").unwrap().id, builtin::COLUMN);
}

#[test]
fn every_builtin_visual_element_exposes_group_opacity() {
    let registry = builtin::registry().unwrap();
    for native in registry.schemas() {
        assert!(
            native
                .properties
                .iter()
                .any(|property| property.id == builtin::OPACITY
                    && property.value_type == ValueType::Float),
            "{} lacks group opacity",
            native.name
        );
    }
    let text = registry
        .construct(
            builtin::TEXT,
            &NativeElementInput::new()
                .property(builtin::CONTENT, SchemaValue::String("fade".into()))
                .property(builtin::OPACITY, SchemaValue::Float(0.25)),
        )
        .unwrap();
    assert_eq!(text.layer.as_ref().unwrap().opacity, 0.25);
}

#[test]
fn registry_types_and_adapter_level_cross_property_rules_are_enforced() {
    let registry = builtin::registry().unwrap();
    let error = registry
        .construct(
            builtin::TEXT,
            &NativeElementInput::new().property(builtin::CONTENT, SchemaValue::Bool(true)),
        )
        .unwrap_err();
    assert!(matches!(error, SchemaError::PropertyType { .. }));

    let error = registry
        .construct(builtin::TEXT, &NativeElementInput::new())
        .unwrap_err();
    assert!(matches!(error, SchemaError::Adapter(_)));
}

#[test]
fn registry_rejects_ambiguous_canonical_metadata() {
    let mut registry = SchemaRegistry::new();
    let schema = NativeSchema::new(NativeTypeId::from_raw(90), "Probe", "Probe schema")
        .property(PropertySchema::new(
            PropertyId::from_raw(1),
            "first",
            ValueType::Float,
            "First property",
        ))
        .property(PropertySchema::new(
            PropertyId::from_raw(1),
            "second",
            ValueType::Float,
            "Second property",
        ));
    let error = registry
        .register(schema, |_input: &NativeElementInput| {
            Ok(Element::container([]))
        })
        .unwrap_err();
    assert!(matches!(error, SchemaError::DuplicateMemberId { .. }));
}

#[test]
fn required_slot_arity_is_enforced() {
    let mut registry = SchemaRegistry::new();
    let slot = SlotId::from_raw(1);
    let schema =
        NativeSchema::new(NativeTypeId::from_raw(91), "OneChild", "One child").slot(SlotSchema {
            id: slot,
            name: "content".into(),
            arity: SlotArity::Required,
            documentation: "Required content.".into(),
        });
    registry
        .register(schema, move |input: &NativeElementInput| {
            Ok(Element::container(input.children(slot).iter().cloned()))
        })
        .unwrap();
    assert!(matches!(
        registry.construct(NativeTypeId::from_raw(91), &NativeElementInput::new()),
        Err(SchemaError::SlotArity { actual: 0, .. })
    ));
}

#[test]
fn native_event_handlers_are_validated_and_attached_by_id() {
    let registry = builtin::registry().unwrap();
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(7), 3));
    let area = registry
        .construct(
            builtin::TOUCH_AREA,
            &NativeElementInput::new()
                .property(builtin::KEY, SchemaValue::String("save".into()))
                .event(NativeEventValue::new(builtin::CLICK, handler)),
        )
        .unwrap();
    assert_eq!(area.event_listeners.len(), 1);
    assert_eq!(area.event_listeners[0].event, EventType::Click);

    let unknown = registry
        .construct(
            builtin::TOUCH_AREA,
            &NativeElementInput::new()
                .property(builtin::KEY, SchemaValue::String("save".into()))
                .event(NativeEventValue::new(EventId::from_raw(99), handler)),
        )
        .unwrap_err();
    assert!(matches!(unknown, SchemaError::UnknownEvent { .. }));
}

/// A composed control can disable focus while retaining accessible semantics.
#[test]
fn disabled_focus_scope_has_noninteractive_accessible_semantics() {
    let registry = builtin::registry().unwrap();
    let disabled = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(builtin::KEY, SchemaValue::String("disabled".into()))
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("button".into()))
                .property(
                    builtin::SEMANTIC_LABEL,
                    SchemaValue::String("Disabled".into()),
                )
                .property(builtin::ENABLED, SchemaValue::Bool(false)),
        )
        .unwrap();
    assert!(
        disabled
            .semantics
            .as_ref()
            .is_some_and(|semantics| semantics.state.disabled)
    );
    assert!(
        disabled
            .interaction
            .as_ref()
            .is_some_and(|interaction| !interaction.enabled)
    );
}

/// A generic focus scope exposes composed selection semantics.
#[test]
fn focus_scope_exposes_busy_and_expanded_selection_semantics() {
    let registry = builtin::registry().unwrap();
    let trigger = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(builtin::KEY, SchemaValue::String("theme".into()))
                .property(
                    builtin::SEMANTIC_ROLE,
                    SchemaValue::String("combo_box".into()),
                )
                .property(builtin::SEMANTIC_LABEL, SchemaValue::String("Theme".into()))
                .property(builtin::BUSY, SchemaValue::Bool(true))
                .property(builtin::SEMANTIC_EXPANDABLE, SchemaValue::Bool(true))
                .property(builtin::EXPANDED, SchemaValue::Bool(true)),
        )
        .unwrap();
    let semantics = trigger.semantics.as_ref().unwrap();
    assert_eq!(semantics.role, argui_ui::Role::ComboBox);
    assert!(semantics.state.busy);
    assert_eq!(semantics.state.expanded, Some(true));
    assert!(
        trigger
            .interaction
            .as_ref()
            .is_some_and(|value| value.enabled)
    );
}

/// The generic popup primitive creates an anchored portal with dismiss handling.
#[test]
fn popup_window_keeps_its_anchor_and_dismiss_listener() {
    let registry = builtin::registry().unwrap();
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(9), 1));
    let panel = registry
        .construct(
            builtin::POPUP_WINDOW,
            &NativeElementInput::new()
                .property(builtin::ANCHOR, SchemaValue::String("theme-anchor".into()))
                .property(builtin::KEY, SchemaValue::String("theme-options".into()))
                .event(NativeEventValue::new(builtin::DISMISS, handler)),
        )
        .unwrap();
    assert!(panel.portal.is_some());
    assert_eq!(panel.event_listeners[0].event, EventType::Dismiss);
}

#[test]
fn two_way_property_requires_a_compatible_declared_event() {
    let mut registry = SchemaRegistry::new();
    let event = EventId::from_raw(1);
    let schema = NativeSchema::new(NativeTypeId::from_raw(92), "Editor", "Editor")
        .property(
            PropertySchema::new(PropertyId::from_raw(1), "value", ValueType::String, "Value")
                .changed_by(event),
        )
        .event(
            EventSchema::new(event, "input", EventType::Input, "Input event")
                .payload(ValueType::Bool),
        );
    let error = registry
        .register(schema, |_input: &NativeElementInput| {
            Ok(Element::container([]))
        })
        .unwrap_err();
    assert!(matches!(error, SchemaError::InvalidChangeEvent { .. }));
}

#[test]
fn payloadless_change_events_are_reserved_for_string_text_edits() {
    for (value_type, event_type) in [
        (ValueType::Bool, EventType::TextEdit),
        (ValueType::String, EventType::Input),
    ] {
        let event = EventId::from_raw(1);
        let schema = NativeSchema::new(NativeTypeId::from_raw(93), "Editor", "Editor")
            .property(
                PropertySchema::new(PropertyId::from_raw(1), "value", value_type, "Value")
                    .changed_by(event),
            )
            .event(EventSchema::new(
                event,
                "changed",
                event_type,
                "Change event",
            ));
        let error = SchemaRegistry::new()
            .register(schema, |_input: &NativeElementInput| {
                Ok(Element::container([]))
            })
            .unwrap_err();
        assert!(matches!(error, SchemaError::InvalidChangeEvent { .. }));
    }
}

#[test]
fn registry_rejects_duplicate_names_ids_defaults_and_member_names() {
    let mut registry = SchemaRegistry::new();
    let original = NativeSchema::new(NativeTypeId::from_raw(120), "Original", "first");
    registry
        .register(original.clone(), |_input: &NativeElementInput| {
            Ok(Element::container([]))
        })
        .unwrap();
    assert!(matches!(
        registry.register(original.clone(), |_input: &NativeElementInput| Ok(
            Element::container([])
        )),
        Err(SchemaError::DuplicateNativeId(_))
    ));
    assert!(matches!(
        registry.register(
            NativeSchema::new(NativeTypeId::from_raw(121), "Original", "second"),
            |_input: &NativeElementInput| Ok(Element::container([]))
        ),
        Err(SchemaError::DuplicateNativeName(_))
    ));
    assert_eq!(registry.schemas().count(), 1);
    assert!(registry.schema(NativeTypeId::from_raw(999)).is_none());
    assert!(registry.schema_named("missing").is_none());
    assert!(matches!(
        registry.construct(NativeTypeId::from_raw(999), &NativeElementInput::new()),
        Err(SchemaError::UnknownNative(_))
    ));

    let wrong_default = NativeSchema::new(NativeTypeId::from_raw(122), "WrongDefault", "bad")
        .property(
            PropertySchema::new(PropertyId::from_raw(1), "size", ValueType::Float, "size")
                .default_value(SchemaValue::Bool(true)),
        );
    assert!(matches!(
        registry.register(wrong_default, |_input: &NativeElementInput| Ok(
            Element::container([])
        )),
        Err(SchemaError::InvalidDefault { .. })
    ));

    let duplicate_property_name =
        NativeSchema::new(NativeTypeId::from_raw(123), "DuplicateProperty", "bad")
            .property(PropertySchema::new(
                PropertyId::from_raw(1),
                "value",
                ValueType::Bool,
                "first",
            ))
            .property(PropertySchema::new(
                PropertyId::from_raw(2),
                "value",
                ValueType::Bool,
                "second",
            ));
    assert!(matches!(
        registry.register(duplicate_property_name, |_input: &NativeElementInput| Ok(
            Element::container([])
        )),
        Err(SchemaError::DuplicateMemberName { .. })
    ));

    let duplicate_event = NativeSchema::new(NativeTypeId::from_raw(124), "DuplicateEvent", "bad")
        .event(EventSchema::new(
            EventId::from_raw(1),
            "one",
            EventType::Click,
            "one",
        ))
        .event(EventSchema::new(
            EventId::from_raw(1),
            "two",
            EventType::Click,
            "two",
        ));
    assert!(matches!(
        registry.register(duplicate_event, |_input: &NativeElementInput| Ok(
            Element::container([])
        )),
        Err(SchemaError::DuplicateMemberId { kind: "event", .. })
    ));

    let duplicate_slot = NativeSchema::new(NativeTypeId::from_raw(125), "DuplicateSlot", "bad")
        .slot(SlotSchema {
            id: SlotId::from_raw(1),
            name: "child".into(),
            arity: SlotArity::Many,
            documentation: String::new(),
        })
        .slot(SlotSchema {
            id: SlotId::from_raw(2),
            name: "child".into(),
            arity: SlotArity::Many,
            documentation: String::new(),
        });
    assert!(matches!(
        registry.register(duplicate_slot, |_input: &NativeElementInput| Ok(
            Element::container([])
        )),
        Err(SchemaError::DuplicateMemberName { kind: "slot", .. })
    ));

    let duplicate_variant =
        NativeSchema::new(NativeTypeId::from_raw(126), "DuplicateVariant", "bad")
            .variant(VariantSchema {
                id: VariantId::from_raw(1),
                name: "on".into(),
                documentation: String::new(),
            })
            .variant(VariantSchema {
                id: VariantId::from_raw(1),
                name: "off".into(),
                documentation: String::new(),
            });
    assert!(matches!(
        registry.register(duplicate_variant, |_input: &NativeElementInput| Ok(
            Element::container([])
        )),
        Err(SchemaError::DuplicateMemberId {
            kind: "variant",
            ..
        })
    ));

    let duplicate_part = NativeSchema::new(NativeTypeId::from_raw(127), "DuplicatePart", "bad")
        .style_part(StylePartSchema {
            id: StylePartId::from_raw(1),
            name: "body".into(),
            documentation: String::new(),
        })
        .style_part(StylePartSchema {
            id: StylePartId::from_raw(2),
            name: "body".into(),
            documentation: String::new(),
        });
    assert!(matches!(
        registry.register(duplicate_part, |_input: &NativeElementInput| Ok(
            Element::container([])
        )),
        Err(SchemaError::DuplicateMemberName {
            kind: "style part",
            ..
        })
    ));
}

#[test]
fn construction_rejects_duplicate_missing_and_unknown_inputs_without_calling_adapter() {
    let mut registry = SchemaRegistry::new();
    let id = NativeTypeId::from_raw(130);
    let required = PropertyId::from_raw(1);
    let event = EventId::from_raw(1);
    let optional = SlotId::from_raw(1);
    let mandatory = SlotId::from_raw(2);
    let schema = NativeSchema::new(id, "Validated", "validation fixture")
        .property(PropertySchema::new(required, "value", ValueType::String, "value").required())
        .event(EventSchema::new(event, "click", EventType::Click, "click"))
        .slot(SlotSchema {
            id: optional,
            name: "optional".into(),
            arity: SlotArity::Optional,
            documentation: String::new(),
        })
        .slot(SlotSchema {
            id: mandatory,
            name: "mandatory".into(),
            arity: SlotArity::Required,
            documentation: String::new(),
        });
    registry
        .register(schema, |_input: &NativeElementInput| {
            Ok(Element::container([]))
        })
        .unwrap();
    let valid = NativeElementInput::new()
        .property(required, SchemaValue::String("ok".into()))
        .slot(NativeSlotValue::new(mandatory, [Element::container([])]));
    assert!(registry.construct(id, &valid).is_ok());
    assert!(matches!(
        registry.construct(id, &NativeElementInput::new()),
        Err(SchemaError::MissingProperty { .. })
    ));
    assert!(matches!(
        registry.construct(
            id,
            &valid
                .clone()
                .property(required, SchemaValue::String("twice".into()))
        ),
        Err(SchemaError::DuplicateProperty(_))
    ));
    assert!(matches!(
        registry.construct(
            id,
            &valid
                .clone()
                .property(PropertyId::from_raw(99), SchemaValue::Bool(true))
        ),
        Err(SchemaError::UnknownProperty { .. })
    ));
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(4), 1));
    assert!(matches!(
        registry.construct(
            id,
            &valid
                .clone()
                .event(NativeEventValue::new(event, handler))
                .event(NativeEventValue::new(event, handler))
        ),
        Err(SchemaError::DuplicateEvent(_))
    ));
    assert!(matches!(
        registry.construct(
            id,
            &valid
                .clone()
                .event(NativeEventValue::new(EventId::from_raw(99), handler))
        ),
        Err(SchemaError::UnknownEvent { .. })
    ));
    assert!(matches!(
        registry.construct(
            id,
            &valid
                .clone()
                .slot(NativeSlotValue::new(mandatory, [Element::container([])]))
        ),
        Err(SchemaError::DuplicateSlot(_))
    ));
    assert!(matches!(
        registry.construct(
            id,
            &valid
                .clone()
                .slot(NativeSlotValue::new(SlotId::from_raw(99), []))
        ),
        Err(SchemaError::UnknownSlot { .. })
    ));
    assert!(matches!(
        registry.construct(
            id,
            &valid.clone().slot(NativeSlotValue::new(
                optional,
                [Element::container([]), Element::container([])]
            ))
        ),
        Err(SchemaError::SlotArity { actual: 2, .. })
    ));
    assert!(
        registry
            .construct(id, &valid.clone().slot(NativeSlotValue::new(optional, [])))
            .is_ok()
    );
    assert!(matches!(
        registry.construct(
            id,
            &NativeElementInput::new()
                .property(required, SchemaValue::String("ok".into()))
                .slot(NativeSlotValue::new(mandatory, []))
        ),
        Err(SchemaError::SlotArity { actual: 0, .. })
    ));
}
