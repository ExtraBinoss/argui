//! Accessible native switch behavior for DSL-composed visuals.

use argui_ui::{
    CheckedState, CursorIcon, Element, EventType, FocusPolicy, GestureSet, Interaction,
    KeyboardActivation, Role, SemanticAction, SemanticState, Semantics, UserSelect,
};

use super::{
    CHECKED, CHILDREN, CLICK, CommonProperty, ENABLED, LABEL, SWITCH, apply_common,
    apply_container, apply_events, common_event, common_property, optional_bool, required_string,
};
use crate::{
    NativeElementInput, NativeSchema, PropertySchema, SchemaError, SchemaRegistry, SchemaValue,
    SlotArity, SlotSchema, ValueType,
};

/// Registers a controlled accessible switch whose visual children are DSL-owned.
///
/// * `registry` — canonical native registry.
///
/// # Errors
///
/// Returns when the schema conflicts with another built-in declaration.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        SWITCH,
        "SwitchControl",
        "Accessible on/off switch behavior.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Tooltip))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::Background))
    .property(common_property(CommonProperty::Padding))
    .property(
        PropertySchema::new(
            LABEL,
            "label",
            ValueType::String,
            "Accessible switch label.",
        )
        .required(),
    )
    .property(
        PropertySchema::new(CHECKED, "checked", ValueType::Bool, "Current on/off state.")
            .default_value(SchemaValue::Bool(false)),
    )
    .property(
        PropertySchema::new(
            ENABLED,
            "enabled",
            ValueType::Bool,
            "Whether activation is allowed.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .event(common_event(CLICK, "click", EventType::Click))
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Visual switch content.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let key = required_string(input, super::KEY, "key")?;
        let label = required_string(input, LABEL, "label")?;
        let enabled = optional_bool(input, ENABLED).unwrap_or(true);
        let checked = optional_bool(input, CHECKED).unwrap_or(false);
        let children = input
            .children(CHILDREN)
            .iter()
            .cloned()
            .map(|child| child.semantic_hidden(true));
        let element = Element::row(children)
            .keyed(key.clone())
            .user_select(UserSelect::None)
            .interaction(
                Interaction::default()
                    .enabled(enabled)
                    .focus_policy(if enabled {
                        FocusPolicy::TabStop
                    } else {
                        FocusPolicy::None
                    })
                    .cursor(if enabled {
                        CursorIcon::Pointer
                    } else {
                        CursorIcon::NotAllowed
                    })
                    .gestures(GestureSet::default().tap(argui_ui::TapGesture::default()))
                    .keyboard_activation(KeyboardActivation::EnterOrSpace),
            )
            .semantics(
                Semantics::new(Role::Switch)
                    .label(label.clone())
                    .state(SemanticState {
                        disabled: !enabled,
                        checked: Some(if checked {
                            CheckedState::Checked
                        } else {
                            CheckedState::Unchecked
                        }),
                        ..SemanticState::default()
                    })
                    .action(SemanticAction::Click)
                    .action(SemanticAction::Focus),
            );
        Ok(apply_events(
            apply_container(apply_common(element, input), input),
            input,
        ))
    })
}
