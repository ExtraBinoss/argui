use argui_schema::{NativeElementInput, NativeEventValue, SchemaError, SchemaValue, builtin};
use argui_ui::{
    CheckedState, EventHandler, EventHandlerId, EventOwnerId, EventType, FocusPolicy, Role,
    SemanticAction, SemanticValue,
};

#[test]
fn every_builtin_exposes_the_same_semantic_contract() {
    let registry = builtin::registry().unwrap();
    for native in registry.schemas() {
        for name in [
            "role",
            "accessible_name",
            "accessible_description",
            "labelled_by",
            "described_by",
            "controls",
            "focusable",
            "keyboard_activation",
            "numeric_value",
            "checked_state",
            "can_increment",
        ] {
            assert!(
                native
                    .properties
                    .iter()
                    .any(|property| property.name.as_str() == name),
                "{}.{} is unavailable",
                native.name,
                name
            );
        }
        assert!(
            native
                .events
                .iter()
                .any(|event| event.name.as_str() == "semantic_action")
        );
    }
}

#[test]
fn visual_primitives_gain_semantics_only_when_authored() {
    let registry = builtin::registry().unwrap();
    let decorative = registry
        .construct(builtin::RECTANGLE, &NativeElementInput::new())
        .unwrap();
    assert!(decorative.semantics.is_none());

    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(7), 1));
    let button = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("button".into()))
                .property(builtin::SEMANTIC_LABEL, SchemaValue::String("Save".into()))
                .property(builtin::SEMANTIC_FOCUSABLE, SchemaValue::Bool(true))
                .property(
                    builtin::KEYBOARD_ACTIVATION,
                    SchemaValue::String("enter_or_space".into()),
                )
                .event(NativeEventValue::new(builtin::CLICK, handler)),
        )
        .unwrap();
    let semantics = button.semantics.as_ref().unwrap();
    assert_eq!(semantics.role, Role::Button);
    assert_eq!(semantics.label.as_deref(), Some("Save"));
    assert!(semantics.actions.contains(&SemanticAction::Focus));
    assert!(semantics.actions.contains(&SemanticAction::Click));
    assert_eq!(
        button.interaction.as_ref().unwrap().focus_policy,
        FocusPolicy::TabStop
    );
    assert!(
        button
            .event_listeners
            .iter()
            .any(|listener| listener.event == EventType::Click)
    );

    let heading = registry
        .construct(
            builtin::TEXT,
            &NativeElementInput::new()
                .property(builtin::TEXT_VALUE, SchemaValue::String("Results".into()))
                .property(
                    builtin::SEMANTIC_ROLE,
                    SchemaValue::String("heading".into()),
                )
                .property(
                    builtin::SEMANTIC_LABEL,
                    SchemaValue::String("Results".into()),
                )
                .property(builtin::SEMANTIC_LEVEL, SchemaValue::Int(2)),
        )
        .unwrap();
    assert_eq!(heading.semantics.as_ref().unwrap().role, Role::Heading);
    assert_eq!(heading.semantics.as_ref().unwrap().level, Some(2));
    assert!(heading.interaction.is_none());
}

#[test]
fn numeric_ranges_and_mixed_state_reach_native_semantics() {
    let registry = builtin::registry().unwrap();
    let slider = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("slider".into()))
                .property(builtin::SEMANTIC_NUMERIC_VALUE, SchemaValue::Float(42.0))
                .property(builtin::SEMANTIC_MINIMUM_VALUE, SchemaValue::Float(0.0))
                .property(builtin::SEMANTIC_MAXIMUM_VALUE, SchemaValue::Float(100.0))
                .property(builtin::SEMANTIC_VALUE_STEP, SchemaValue::Float(2.0))
                .property(builtin::SEMANTIC_CAN_INCREMENT, SchemaValue::Bool(true)),
        )
        .unwrap();
    let semantics = slider.semantics.as_ref().unwrap();
    assert_eq!(
        semantics.value,
        Some(SemanticValue::Number {
            value: 42.0,
            minimum: Some(0.0),
            maximum: Some(100.0),
            step: Some(2.0),
        })
    );
    assert!(semantics.actions.contains(&SemanticAction::Increment));

    let checkbox = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(
                    builtin::SEMANTIC_ROLE,
                    SchemaValue::String("check_box".into()),
                )
                .property(
                    builtin::SEMANTIC_CHECKED_STATE,
                    SchemaValue::String("mixed".into()),
                ),
        )
        .unwrap();
    assert_eq!(
        checkbox.semantics.as_ref().unwrap().state.checked,
        Some(CheckedState::Mixed)
    );
}

#[test]
fn malformed_numeric_and_checked_states_are_rejected() {
    let registry = builtin::registry().unwrap();
    for input in [
        NativeElementInput::new()
            .property(builtin::SEMANTIC_MINIMUM_VALUE, SchemaValue::Float(0.0)),
        NativeElementInput::new().property(
            builtin::SEMANTIC_NUMERIC_VALUE,
            SchemaValue::Float(f32::NAN),
        ),
        NativeElementInput::new()
            .property(builtin::SEMANTIC_NUMERIC_VALUE, SchemaValue::Float(2.0))
            .property(builtin::SEMANTIC_VALUE_STEP, SchemaValue::Float(0.0)),
        NativeElementInput::new().property(
            builtin::SEMANTIC_CHECKED_STATE,
            SchemaValue::String("unknown".into()),
        ),
        NativeElementInput::new().property(builtin::SEMANTIC_LEVEL, SchemaValue::Int(0)),
        NativeElementInput::new().property(
            builtin::SEMANTIC_HAS_POPUP,
            SchemaValue::String("unknown".into()),
        ),
    ] {
        assert!(matches!(
            registry.construct(builtin::RECTANGLE, &input),
            Err(SchemaError::Adapter(_))
        ));
    }
}

#[test]
fn semantic_relations_structure_and_hidden_state_apply_to_any_primitive() {
    let registry = builtin::registry().unwrap();
    let list = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("list".into()))
                .property(
                    builtin::SEMANTIC_LABELLED_BY,
                    SchemaValue::String("first second".into()),
                )
                .property(
                    builtin::SEMANTIC_ORIENTATION,
                    SchemaValue::String("vertical".into()),
                )
                .property(builtin::SEMANTIC_POSITION_IN_SET, SchemaValue::Int(2))
                .property(builtin::SEMANTIC_SET_SIZE, SchemaValue::Int(5))
                .property(builtin::SEMANTIC_HIDDEN, SchemaValue::Bool(true)),
        )
        .unwrap();
    assert!(list.semantic_hidden);
    assert_eq!(list.semantic_bindings.labelled_by.len(), 2);
    let semantics = list.semantics.as_ref().unwrap();
    assert_eq!(semantics.orientation, Some(argui_ui::Orientation::Vertical));
    assert_eq!(semantics.position_in_set, Some(2));
    assert_eq!(semantics.set_size, Some(5));
}

#[test]
fn custom_control_states_actions_and_relations_are_explicit() {
    let registry = builtin::registry().unwrap();
    let element = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(
                    builtin::SEMANTIC_ROLE,
                    SchemaValue::String("combo_box".into()),
                )
                .property(
                    builtin::SEMANTIC_DESCRIPTION,
                    SchemaValue::String("Choose one".into()),
                )
                .property(builtin::SEMANTIC_VALUE, SchemaValue::String("Blue".into()))
                .property(builtin::SEMANTIC_SELECTED, SchemaValue::Bool(true))
                .property(builtin::SEMANTIC_PRESSED, SchemaValue::Bool(false))
                .property(builtin::SEMANTIC_REQUIRED, SchemaValue::Bool(true))
                .property(builtin::SEMANTIC_READ_ONLY, SchemaValue::Bool(true))
                .property(builtin::SEMANTIC_MULTISELECTABLE, SchemaValue::Bool(true))
                .property(builtin::SEMANTIC_INVALID, SchemaValue::Bool(true))
                .property(builtin::SEMANTIC_MODAL, SchemaValue::Bool(true))
                .property(
                    builtin::SEMANTIC_CURRENT,
                    SchemaValue::String("page".into()),
                )
                .property(builtin::SEMANTIC_LIVE, SchemaValue::String("polite".into()))
                .property(
                    builtin::SEMANTIC_HAS_POPUP,
                    SchemaValue::String("list_box".into()),
                )
                .property(
                    builtin::SEMANTIC_SORT,
                    SchemaValue::String("ascending".into()),
                )
                .property(
                    builtin::SEMANTIC_CONTROLS,
                    SchemaValue::String("first second".into()),
                )
                .property(
                    builtin::SEMANTIC_DESCRIBED_BY,
                    SchemaValue::String("hint".into()),
                )
                .property(
                    builtin::SEMANTIC_ACTIVE_DESCENDANT,
                    SchemaValue::String("option".into()),
                )
                .property(builtin::SEMANTIC_CAN_DECREMENT, SchemaValue::Bool(true))
                .property(builtin::SEMANTIC_CAN_SET_VALUE, SchemaValue::Bool(true))
                .property(builtin::SEMANTIC_CAN_EXPAND, SchemaValue::Bool(true))
                .property(builtin::SEMANTIC_CAN_COLLAPSE, SchemaValue::Bool(true))
                .property(
                    builtin::SEMANTIC_CAN_SCROLL_INTO_VIEW,
                    SchemaValue::Bool(true),
                ),
        )
        .unwrap();
    let semantics = element.semantics.as_ref().unwrap();
    assert_eq!(semantics.description.as_deref(), Some("Choose one"));
    assert_eq!(semantics.value, Some(SemanticValue::Text("Blue".into())));
    assert!(semantics.state.selected && semantics.state.required && semantics.state.invalid);
    assert!(semantics.state.read_only && semantics.state.modal && semantics.state.multiselectable);
    assert_eq!(semantics.state.pressed, Some(false));
    assert_eq!(semantics.state.current, Some(argui_ui::Current::Page));
    assert_eq!(semantics.live, argui_ui::LiveRegion::Polite);
    assert_eq!(semantics.popup, Some(argui_ui::PopupKind::ListBox));
    assert_eq!(semantics.sort, Some(argui_ui::SortDirection::Ascending));
    assert_eq!(element.semantic_bindings.controls.len(), 2);
    assert_eq!(element.semantic_bindings.described_by.len(), 1);
    assert!(element.semantic_bindings.active_descendant.is_some());
    for action in [
        SemanticAction::Expand,
        SemanticAction::Collapse,
        SemanticAction::ScrollIntoView,
    ] {
        assert!(semantics.actions.contains(&action));
    }
    for action in [SemanticAction::Decrement, SemanticAction::SetValue] {
        assert!(!semantics.actions.contains(&action));
    }
}

#[test]
fn explicit_false_removes_a_native_action() {
    let registry = builtin::registry().unwrap();
    let input = NativeElementInput::new()
        .property(builtin::KEY, SchemaValue::String("editor".into()))
        .property(builtin::SEMANTIC_CAN_SET_VALUE, SchemaValue::Bool(false));
    let editor = registry.construct(builtin::TEXT_INPUT, &input).unwrap();
    assert!(
        !editor
            .semantics
            .as_ref()
            .unwrap()
            .actions
            .contains(&SemanticAction::SetValue)
    );
}

#[test]
fn disabled_and_nonfocusable_controls_advertise_no_actions() {
    let registry = builtin::registry().unwrap();
    let element = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("button".into()))
                .property(builtin::SEMANTIC_FOCUSABLE, SchemaValue::Bool(false))
                .property(builtin::SEMANTIC_DISABLED, SchemaValue::Bool(true))
                .property(
                    builtin::KEYBOARD_ACTIVATION,
                    SchemaValue::String("enter_or_space".into()),
                )
                .property(builtin::SEMANTIC_CAN_EXPAND, SchemaValue::Bool(true)),
        )
        .unwrap();
    let semantics = element.semantics.as_ref().unwrap();
    assert!(semantics.state.disabled);
    assert!(semantics.actions.is_empty());
    assert_eq!(
        element.interaction.as_ref().unwrap().focus_policy,
        FocusPolicy::None
    );
}
