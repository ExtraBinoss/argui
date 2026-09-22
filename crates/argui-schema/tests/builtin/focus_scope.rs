use argui_schema::{
    NativeElementInput, NativeSlotValue, ObservationKind, SchemaError, SchemaValue, builtin,
};
use argui_ui::{Element, FocusContainment, FocusPolicy, InitialFocus, KeyboardActivation, Role};

#[test]
fn focus_scope_exposes_keyboard_focus_without_painting_a_control() {
    let registry = builtin::registry().unwrap();
    let scope = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(
                    builtin::KEYBOARD_ACTIVATION,
                    SchemaValue::String("enter_or_space".into()),
                )
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("button".into()))
                .property(
                    builtin::SEMANTIC_LABEL,
                    SchemaValue::String("Run action".into()),
                )
                .slot(NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::text("Run action")],
                )),
        )
        .unwrap();
    let interaction = scope.interaction.as_ref().unwrap();
    assert_eq!(interaction.focus_policy, FocusPolicy::TabStop);
    assert!(interaction.focus_on_descendant_press);
    assert_eq!(
        interaction.keyboard_activation,
        KeyboardActivation::EnterOrSpace
    );
    assert_eq!(scope.semantics.as_ref().unwrap().role, Role::Button);
    assert!(scope.paint.quad.background.is_none());
    assert_eq!(scope.children.len(), 1);
    let schema = registry.schema(builtin::FOCUS_SCOPE).unwrap();
    assert_eq!(
        schema
            .properties
            .iter()
            .find(|property| property.id == builtin::HAS_FOCUS)
            .unwrap()
            .observation,
        Some(ObservationKind::Focused)
    );
}

#[test]
fn modal_focus_scope_can_target_and_restore_focus() {
    let registry = builtin::registry().unwrap();
    let scope = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(
                    builtin::FOCUS_CONTAINMENT,
                    SchemaValue::String("modal".into()),
                )
                .property(
                    builtin::INITIAL_FOCUS,
                    SchemaValue::String("confirm".into()),
                )
                .property(builtin::RESTORE_FOCUS, SchemaValue::Bool(false))
                .property(builtin::FOCUS_ON_TAB, SchemaValue::Bool(false)),
        )
        .unwrap();
    let focus = scope.focus_scope.as_ref().unwrap();
    assert_eq!(focus.containment, FocusContainment::Modal);
    assert!(matches!(focus.initial, Some(InitialFocus::Target(_))));
    assert!(!focus.restore);
    assert_eq!(
        scope.interaction.as_ref().unwrap().focus_policy,
        FocusPolicy::Programmatic
    );
}

#[test]
fn focus_scope_validates_policies_and_exposes_disabled_semantics() {
    let registry = builtin::registry().unwrap();
    let disabled = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(builtin::ENABLED, SchemaValue::Bool(false))
                .property(
                    builtin::FOCUS_CONTAINMENT,
                    SchemaValue::String("trap".into()),
                )
                .property(builtin::INITIAL_FOCUS, SchemaValue::String("first".into()))
                .property(
                    builtin::KEYBOARD_ACTIVATION,
                    SchemaValue::String("enter".into()),
                )
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("menu".into()))
                .property(builtin::BUSY, SchemaValue::Bool(true))
                .property(builtin::CHECKED, SchemaValue::Bool(false))
                .property(builtin::SEMANTIC_EXPANDABLE, SchemaValue::Bool(true))
                .property(builtin::EXPANDED, SchemaValue::Bool(true)),
        )
        .unwrap();
    assert_eq!(
        disabled.interaction.as_ref().unwrap().focus_policy,
        FocusPolicy::None
    );
    assert_eq!(
        disabled.focus_scope.as_ref().unwrap().containment,
        FocusContainment::Trap
    );
    assert_eq!(
        disabled.focus_scope.as_ref().unwrap().initial,
        Some(InitialFocus::First)
    );
    assert_eq!(disabled.semantics.as_ref().unwrap().role, Role::Menu);
    let state = &disabled.semantics.as_ref().unwrap().state;
    assert!(state.disabled);
    assert!(state.busy);
    assert_eq!(state.expanded, Some(true));

    for (property, value) in [
        (builtin::FOCUS_CONTAINMENT, "unknown"),
        (builtin::KEYBOARD_ACTIVATION, "space_only"),
        (builtin::SEMANTIC_ROLE, "unknown_role"),
    ] {
        let error = registry
            .construct(
                builtin::FOCUS_SCOPE,
                &NativeElementInput::new()
                    .property(property, SchemaValue::String(value.into()))
                    .property(builtin::SEMANTIC_LABEL, SchemaValue::String("label".into())),
            )
            .unwrap_err();
        assert!(matches!(error, SchemaError::Adapter(_)), "{value}");
    }
}
