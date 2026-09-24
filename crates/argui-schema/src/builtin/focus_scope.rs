//! Unpainted focus and keyboard boundary for declarative controls.

use argui_ui::{
    CheckedState, Element, EventType, FocusContainment, FocusPolicy, FocusScope, FocusTarget,
    InitialFocus, Interaction, KeyboardActivation, Role, SemanticAction, SemanticState,
    SemanticValue, Semantics, UserSelect,
};

use super::focus_scope_parse::{
    parse_activation, parse_containment, parse_current, parse_live, parse_role,
};
use super::{
    BLUR, BUSY, CAPTURE_KEY_INPUT, CHECKED, CHILDREN, CLICK, CommonProperty, ENABLED, EXPANDED,
    FOCUS, FOCUS_CONTAINMENT, FOCUS_ON_CLICK, FOCUS_ON_TAB, FOCUS_VISIBLE, HAS_FOCUS,
    INITIAL_FOCUS, KEY_INPUT, KEYBOARD_ACTIVATION, RESTORE_FOCUS, SEMANTIC_ACTION,
    SEMANTIC_ACTIVE_DESCENDANT, SEMANTIC_CAN_COLLAPSE, SEMANTIC_CAN_DECREMENT, SEMANTIC_CAN_EXPAND,
    SEMANTIC_CAN_INCREMENT, SEMANTIC_CAN_SCROLL_INTO_VIEW, SEMANTIC_CAN_SET_VALUE,
    SEMANTIC_CONTROLS, SEMANTIC_CURRENT, SEMANTIC_DESCRIBED_BY, SEMANTIC_DESCRIPTION,
    SEMANTIC_EXPANDABLE, SEMANTIC_FOCUSABLE, SEMANTIC_INVALID, SEMANTIC_LABEL,
    SEMANTIC_LABELLED_BY, SEMANTIC_LIVE, SEMANTIC_MAXIMUM_VALUE, SEMANTIC_MINIMUM_VALUE,
    SEMANTIC_MULTISELECTABLE, SEMANTIC_NUMERIC_VALUE, SEMANTIC_PRESSED, SEMANTIC_READ_ONLY,
    SEMANTIC_REQUIRED, SEMANTIC_ROLE, SEMANTIC_SELECTED, SEMANTIC_VALUE, SEMANTIC_VALUE_STEP,
    apply_common, common_event, common_property, optional_bool,
};
use crate::{
    NativeElementInput, NativeSchema, ObservationKind, PropertySchema, SchemaError, SchemaRegistry,
    SchemaValue, SlotArity, SlotSchema, ValueType,
};

/// Registers a generic focus scope with keyboard activation and observable focus.
///
/// * `registry` — native registry receiving the scope metadata and adapter.
///
/// # Errors
///
/// Returns a schema error when built-in identifiers or metadata conflict.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        super::FOCUS_SCOPE,
        "FocusScope",
        "Unpainted focus boundary with optional keyboard activation and accessibility semantics.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Tooltip))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::X))
    .property(common_property(CommonProperty::Y))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::BackdropFilter))
    .property(common_property(CommonProperty::DesktopBackdropTint))
    .property(common_property(CommonProperty::DesktopBackdropFallback))
    .property(common_property(CommonProperty::Visible))
    .property(
        PropertySchema::new(
            ENABLED,
            "enabled",
            ValueType::Bool,
            "Whether this scope can focus.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(
        PropertySchema::new(
            FOCUS_ON_CLICK,
            "focus_on_click",
            ValueType::Bool,
            "Focus this scope when a nonfocusable descendant is pressed.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(
        PropertySchema::new(
            FOCUS_ON_TAB,
            "focus_on_tab_navigation",
            ValueType::Bool,
            "Include this scope in sequential keyboard focus.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(
        PropertySchema::new(
            HAS_FOCUS,
            "has_focus",
            ValueType::Bool,
            "Scope owns keyboard focus.",
        )
        .observed(ObservationKind::Focused),
    )
    .property(
        PropertySchema::new(
            FOCUS_VISIBLE,
            "focus_visible",
            ValueType::Bool,
            "Keyboard focus indication is active.",
        )
        .observed(ObservationKind::FocusVisible),
    )
    .property(PropertySchema::new(
        FOCUS_CONTAINMENT,
        "containment",
        ValueType::String,
        "Focus containment: none, trap, or modal.",
    ))
    .property(PropertySchema::new(
        RESTORE_FOCUS,
        "restore_focus",
        ValueType::Bool,
        "Restore prior focus when the scope disappears.",
    ))
    .property(PropertySchema::new(
        INITIAL_FOCUS,
        "initial_focus",
        ValueType::String,
        "Initial child key, or first for the first focusable child.",
    ))
    .property(PropertySchema::new(
        KEYBOARD_ACTIVATION,
        "keyboard_activation",
        ValueType::String,
        "Keys generating click: none, enter, or enter_or_space.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_ROLE,
        "role",
        ValueType::String,
        "Accessible role for a composed control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_LABEL,
        "accessible_name",
        ValueType::String,
        "Accessible name for a composed control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_DESCRIPTION,
        "accessible_description",
        ValueType::String,
        "Accessible description for a composed control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_VALUE,
        "accessible_value",
        ValueType::String,
        "Current text value announced by assistive technology.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_NUMERIC_VALUE,
        "numeric_value",
        ValueType::Float,
        "Current numeric value announced by assistive technology.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_MINIMUM_VALUE,
        "minimum_value",
        ValueType::Float,
        "Lower bound of numeric_value.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_MAXIMUM_VALUE,
        "maximum_value",
        ValueType::Float,
        "Upper bound of numeric_value.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_VALUE_STEP,
        "value_step",
        ValueType::Float,
        "Increment of numeric_value.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_FOCUSABLE,
        "focusable",
        ValueType::Bool,
        "Whether this scope can receive programmatic or accessibility focus.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_PRESSED,
        "pressed_state",
        ValueType::Bool,
        "Persistent pressed state of a toggle button.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_REQUIRED,
        "required",
        ValueType::Bool,
        "Whether a value is required.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_READ_ONLY,
        "read_only",
        ValueType::Bool,
        "Whether the value cannot be edited.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_MULTISELECTABLE,
        "multiselectable",
        ValueType::Bool,
        "Whether the set supports multiple selections.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CAN_INCREMENT,
        "can_increment",
        ValueType::Bool,
        "Expose an accessible increment action.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CAN_DECREMENT,
        "can_decrement",
        ValueType::Bool,
        "Expose an accessible decrement action.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CAN_SET_VALUE,
        "can_set_value",
        ValueType::Bool,
        "Expose an accessible set-value action.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CAN_EXPAND,
        "can_expand",
        ValueType::Bool,
        "Expose an accessible expand action.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CAN_COLLAPSE,
        "can_collapse",
        ValueType::Bool,
        "Expose an accessible collapse action.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CAN_SCROLL_INTO_VIEW,
        "can_scroll_into_view",
        ValueType::Bool,
        "Expose an accessible scroll-into-view action.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_SELECTED,
        "selected",
        ValueType::Bool,
        "Whether this item is selected in its set.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CURRENT,
        "current",
        ValueType::String,
        "Current item kind: true, page, step, location, date, or time.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CONTROLS,
        "controls",
        ValueType::String,
        "Key of the element controlled by this control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_ACTIVE_DESCENDANT,
        "active_descendant",
        ValueType::String,
        "Key of the active child in a composite control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_LABELLED_BY,
        "labelled_by",
        ValueType::String,
        "Key of the element providing this control's accessible name.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_DESCRIBED_BY,
        "described_by",
        ValueType::String,
        "Key of the element providing this control's description.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_INVALID,
        "invalid",
        ValueType::Bool,
        "Whether the composed control has an invalid value.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_LIVE,
        "live",
        ValueType::String,
        "Announcement policy: off, polite, or assertive.",
    ))
    .property(PropertySchema::new(
        BUSY,
        "busy",
        ValueType::Bool,
        "Whether the composed control is busy.",
    ))
    .property(PropertySchema::new(
        CHECKED,
        "checked",
        ValueType::Bool,
        "Checked state for accessible toggle roles.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_EXPANDABLE,
        "expandable",
        ValueType::Bool,
        "Whether expanded state applies to this control.",
    ))
    .property(PropertySchema::new(
        EXPANDED,
        "expanded",
        ValueType::Bool,
        "Current expanded state when expandable is true.",
    ))
    .event(common_event(CLICK, "click", EventType::Click))
    .event(common_event(
        SEMANTIC_ACTION,
        "semantic_action",
        EventType::SemanticAction,
    ))
    .event(common_event(FOCUS, "focus", EventType::Focus))
    .event(common_event(BLUR, "blur", EventType::Blur))
    .event(common_event(KEY_INPUT, "key", EventType::Key))
    .event(common_event(
        CAPTURE_KEY_INPUT,
        "capture_key",
        EventType::Key,
    ))
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Visual and interaction descendants in this focus boundary.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let enabled = optional_bool(input, ENABLED).unwrap_or(true)
            && !optional_bool(input, super::SEMANTIC_DISABLED).unwrap_or(false);
        let focusable = optional_bool(input, SEMANTIC_FOCUSABLE).unwrap_or(true);
        let focus_policy = if !enabled || !focusable {
            FocusPolicy::None
        } else if optional_bool(input, FOCUS_ON_TAB).unwrap_or(true) {
            FocusPolicy::TabStop
        } else {
            FocusPolicy::Programmatic
        };
        let activation = match input.get(KEYBOARD_ACTIVATION) {
            Some(SchemaValue::String(value)) => parse_activation(value)?,
            _ => KeyboardActivation::None,
        };
        let containment = match input.get(FOCUS_CONTAINMENT) {
            Some(SchemaValue::String(value)) => parse_containment(value)?,
            _ => FocusContainment::None,
        };
        let initial = match input.get(INITIAL_FOCUS) {
            Some(SchemaValue::String(value)) if value == "first" => Some(InitialFocus::First),
            Some(SchemaValue::String(value)) if value.is_empty() => None,
            Some(SchemaValue::String(value)) => {
                Some(InitialFocus::Target(FocusTarget::Key(value.clone())))
            }
            _ => None,
        };
        let focus_scope = FocusScope {
            containment,
            initial,
            restore: optional_bool(input, RESTORE_FOCUS).unwrap_or(true),
        };
        let interaction = Interaction::default()
            .enabled(enabled)
            .focus_policy(focus_policy)
            .focus_on_descendant_press(optional_bool(input, FOCUS_ON_CLICK).unwrap_or(true))
            .keyboard_activation(activation);
        let mut element =
            apply_common(Element::container(input.children(CHILDREN).to_vec()), input)?
                .focus_scope(focus_scope)
                .interaction(interaction);
        if activation != KeyboardActivation::None
            || input.events.iter().any(|event| event.id == CLICK)
        {
            element = element.user_select(UserSelect::None);
        }
        let has_semantics = [
            SEMANTIC_ROLE,
            SEMANTIC_FOCUSABLE,
            super::SEMANTIC_DISABLED,
            SEMANTIC_LABEL,
            SEMANTIC_DESCRIPTION,
            SEMANTIC_VALUE,
            SEMANTIC_NUMERIC_VALUE,
            SEMANTIC_PRESSED,
            SEMANTIC_REQUIRED,
            SEMANTIC_READ_ONLY,
            SEMANTIC_MULTISELECTABLE,
            BUSY,
            CHECKED,
            SEMANTIC_EXPANDABLE,
            EXPANDED,
            SEMANTIC_CAN_INCREMENT,
            SEMANTIC_CAN_DECREMENT,
            SEMANTIC_CAN_SET_VALUE,
            SEMANTIC_CAN_EXPAND,
            SEMANTIC_CAN_COLLAPSE,
            SEMANTIC_CAN_SCROLL_INTO_VIEW,
            SEMANTIC_SELECTED,
            SEMANTIC_CURRENT,
            SEMANTIC_CONTROLS,
            SEMANTIC_ACTIVE_DESCENDANT,
            SEMANTIC_LABELLED_BY,
            SEMANTIC_DESCRIBED_BY,
            SEMANTIC_INVALID,
            SEMANTIC_LIVE,
        ]
        .iter()
        .any(|id| input.get(*id).is_some());
        if has_semantics {
            let role = match input.get(SEMANTIC_ROLE) {
                Some(SchemaValue::String(name)) => parse_role(name)?,
                _ => Role::Generic,
            };
            let mut semantics = Semantics::new(role).state(SemanticState {
                disabled: !enabled,
                busy: optional_bool(input, BUSY).unwrap_or(false),
                selected: optional_bool(input, SEMANTIC_SELECTED).unwrap_or(false),
                pressed: optional_bool(input, SEMANTIC_PRESSED),
                required: optional_bool(input, SEMANTIC_REQUIRED).unwrap_or(false),
                read_only: optional_bool(input, SEMANTIC_READ_ONLY).unwrap_or(false),
                multiselectable: optional_bool(input, SEMANTIC_MULTISELECTABLE).unwrap_or(false),
                invalid: optional_bool(input, SEMANTIC_INVALID).unwrap_or(false),
                current: match input.get(SEMANTIC_CURRENT) {
                    Some(SchemaValue::String(value)) => Some(parse_current(value)?),
                    _ => None,
                },
                checked: optional_bool(input, CHECKED).map(|value| {
                    if value {
                        CheckedState::Checked
                    } else {
                        CheckedState::Unchecked
                    }
                }),
                expanded: optional_bool(input, SEMANTIC_EXPANDABLE)
                    .filter(|value| *value)
                    .map(|_| optional_bool(input, EXPANDED).unwrap_or(false)),
                ..SemanticState::default()
            });
            if focus_policy.is_focusable() {
                semantics = semantics.action(SemanticAction::Focus);
            }
            if let Some(SchemaValue::String(name)) = input.get(SEMANTIC_LABEL) {
                semantics = semantics.label(name.clone());
            }
            if let Some(SchemaValue::String(description)) = input.get(SEMANTIC_DESCRIPTION) {
                semantics = semantics.description(description.clone());
            }
            if let Some(SchemaValue::String(value)) = input.get(SEMANTIC_VALUE) {
                semantics = semantics.value(SemanticValue::Text(value.clone()));
            }
            if let Some(SchemaValue::Float(value)) = input.get(SEMANTIC_NUMERIC_VALUE) {
                if input.get(SEMANTIC_VALUE).is_some() {
                    return Err(SchemaError::Adapter(
                        "FocusScope accepts either accessible_value or numeric_value".into(),
                    ));
                }
                let bound = |id| match input.get(id) {
                    Some(SchemaValue::Float(value)) => Some(f64::from(*value)),
                    _ => None,
                };
                semantics = semantics.value(SemanticValue::Number {
                    value: f64::from(*value),
                    minimum: bound(SEMANTIC_MINIMUM_VALUE),
                    maximum: bound(SEMANTIC_MAXIMUM_VALUE),
                    step: bound(SEMANTIC_VALUE_STEP),
                });
            }
            if let Some(SchemaValue::String(value)) = input.get(SEMANTIC_LIVE) {
                semantics = semantics.live(parse_live(value)?);
            }
            if enabled
                && (activation != KeyboardActivation::None
                    || input.events.iter().any(|event| event.id == CLICK))
            {
                semantics = semantics.action(SemanticAction::Click);
            }
            for (property, action) in [
                (SEMANTIC_CAN_INCREMENT, SemanticAction::Increment),
                (SEMANTIC_CAN_DECREMENT, SemanticAction::Decrement),
                (SEMANTIC_CAN_SET_VALUE, SemanticAction::SetValue),
                (SEMANTIC_CAN_EXPAND, SemanticAction::Expand),
                (SEMANTIC_CAN_COLLAPSE, SemanticAction::Collapse),
                (
                    SEMANTIC_CAN_SCROLL_INTO_VIEW,
                    SemanticAction::ScrollIntoView,
                ),
            ] {
                if enabled && optional_bool(input, property) == Some(true) {
                    semantics = semantics.action(action);
                }
            }
            element = element.semantics(semantics);
            if let Some(SchemaValue::String(key)) = input.get(SEMANTIC_CONTROLS) {
                element = element.controls([key.as_str()]);
            }
            if let Some(SchemaValue::String(key)) = input.get(SEMANTIC_ACTIVE_DESCENDANT) {
                element = element.active_descendant(key.as_str());
            }
            if let Some(SchemaValue::String(key)) = input.get(SEMANTIC_LABELLED_BY) {
                element = element.labelled_by([key.as_str()]);
            }
            if let Some(SchemaValue::String(key)) = input.get(SEMANTIC_DESCRIBED_BY) {
                element = element.described_by([key.as_str()]);
            }
        }
        for event in &input.events {
            let listener = match event.id {
                CLICK => event.handler.listener(EventType::Click),
                SEMANTIC_ACTION => event.handler.listener(EventType::SemanticAction),
                FOCUS => event.handler.direct_listener(EventType::Focus),
                BLUR => event.handler.direct_listener(EventType::Blur),
                KEY_INPUT => event.handler.listener(EventType::Key),
                CAPTURE_KEY_INPUT => event.handler.listener(EventType::Key).capture(true),
                _ => continue,
            };
            element = element.on(listener);
        }
        Ok(element)
    })
}
