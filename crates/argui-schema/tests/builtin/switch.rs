use argui_schema::{NativeElementInput, NativeSlotValue, SchemaError, SchemaValue, builtin};
use argui_ui::{CheckedState, Element, FocusPolicy, Role};

#[test]
fn switch_defaults_to_enabled_and_unchecked() {
    let registry = builtin::registry().unwrap();
    let child = Element::text("On");
    let control = registry
        .construct(
            builtin::SWITCH,
            &NativeElementInput::new()
                .property(builtin::KEY, SchemaValue::String("wifi".into()))
                .property(builtin::LABEL, SchemaValue::String("Wi-Fi".into()))
                .slot(NativeSlotValue::new(builtin::CHILDREN, [child])),
        )
        .unwrap();
    let semantics = control.semantics.as_ref().unwrap();
    assert_eq!(semantics.role, Role::Switch);
    assert_eq!(semantics.state.checked, Some(CheckedState::Unchecked));
    assert!(!semantics.state.disabled);
    assert!(control.interaction.as_ref().unwrap().enabled);
    assert_eq!(
        control.interaction.as_ref().unwrap().focus_policy,
        FocusPolicy::TabStop
    );
    assert!(control.children[0].semantic_hidden);
}

#[test]
fn checked_disabled_switch_preserves_state_without_accepting_input() {
    let registry = builtin::registry().unwrap();
    let control = registry
        .construct(
            builtin::SWITCH,
            &NativeElementInput::new()
                .property(builtin::KEY, SchemaValue::String("wifi".into()))
                .property(builtin::LABEL, SchemaValue::String("Wi-Fi".into()))
                .property(builtin::CHECKED, SchemaValue::Bool(true))
                .property(builtin::ENABLED, SchemaValue::Bool(false)),
        )
        .unwrap();
    let semantics = control.semantics.as_ref().unwrap();
    assert_eq!(semantics.state.checked, Some(CheckedState::Checked));
    assert!(semantics.state.disabled);
    assert!(!control.interaction.as_ref().unwrap().enabled);
    assert_eq!(
        control.interaction.as_ref().unwrap().focus_policy,
        FocusPolicy::None
    );
}

#[test]
fn switch_requires_accessible_label_and_stable_key() {
    let registry = builtin::registry().unwrap();
    let missing_label = registry.construct(
        builtin::SWITCH,
        &NativeElementInput::new().property(builtin::KEY, SchemaValue::String("wifi".into())),
    );
    assert!(matches!(
        missing_label,
        Err(SchemaError::MissingProperty { .. })
    ));
    let missing_key = registry.construct(
        builtin::SWITCH,
        &NativeElementInput::new().property(builtin::LABEL, SchemaValue::String("Wi-Fi".into())),
    );
    assert!(matches!(missing_key, Err(SchemaError::Adapter(_))));
}
