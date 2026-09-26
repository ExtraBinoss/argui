//! Shared semantic properties for every built-in native primitive.

use argui_ui::{
    CheckedState, Element, EventType, FocusPolicy, KeyboardActivation, Orientation, PopupKind,
    SemanticAction, SemanticValue, SortDirection,
};

use super::{
    BUSY, CHECKED, CLICK, EXPANDED, FOCUS_ON_TAB, KEYBOARD_ACTIVATION, SEMANTIC_ACTION,
    SEMANTIC_ACTIVE_DESCENDANT, SEMANTIC_CAN_COLLAPSE, SEMANTIC_CAN_DECREMENT, SEMANTIC_CAN_EXPAND,
    SEMANTIC_CAN_INCREMENT, SEMANTIC_CAN_SCROLL_INTO_VIEW, SEMANTIC_CAN_SET_VALUE,
    SEMANTIC_CHECKED_STATE, SEMANTIC_CONTROLS, SEMANTIC_CURRENT, SEMANTIC_DESCRIBED_BY,
    SEMANTIC_DESCRIPTION, SEMANTIC_DISABLED, SEMANTIC_FOCUSABLE, SEMANTIC_HAS_POPUP,
    SEMANTIC_HIDDEN, SEMANTIC_INVALID, SEMANTIC_LABEL, SEMANTIC_LABELLED_BY, SEMANTIC_LEVEL,
    SEMANTIC_LIVE, SEMANTIC_MAXIMUM_VALUE, SEMANTIC_MINIMUM_VALUE, SEMANTIC_MODAL,
    SEMANTIC_MULTISELECTABLE, SEMANTIC_NUMERIC_VALUE, SEMANTIC_ORIENTATION,
    SEMANTIC_POSITION_IN_SET, SEMANTIC_PRESSED, SEMANTIC_READ_ONLY, SEMANTIC_REQUIRED,
    SEMANTIC_ROLE, SEMANTIC_SELECTED, SEMANTIC_SET_SIZE, SEMANTIC_SORT, SEMANTIC_VALUE,
    SEMANTIC_VALUE_STEP, focus_scope_parse,
};
use crate::{NativeElementInput, PropertyId, SchemaError, SchemaValue};

pub(crate) use super::accessibility_contract::{events, properties};

/// Applies explicitly authored semantic properties after native construction.
///
/// `element` is the element produced by its original adapter, and `input` is the
/// validated property and event set. Returns that element with shared semantics.
///
/// # Errors
///
/// Returns an adapter error for an unsupported role, state, or keyboard policy.
pub(crate) fn apply(
    mut element: Element,
    input: &NativeElementInput,
) -> Result<Element, SchemaError> {
    const SEMANTIC_IDS: &[PropertyId] = &[
        SEMANTIC_ROLE,
        SEMANTIC_LABEL,
        SEMANTIC_DESCRIPTION,
        SEMANTIC_VALUE,
        SEMANTIC_NUMERIC_VALUE,
        SEMANTIC_MINIMUM_VALUE,
        SEMANTIC_MAXIMUM_VALUE,
        SEMANTIC_VALUE_STEP,
        SEMANTIC_DISABLED,
        SEMANTIC_HIDDEN,
        SEMANTIC_FOCUSABLE,
        FOCUS_ON_TAB,
        KEYBOARD_ACTIVATION,
        SEMANTIC_ORIENTATION,
        SEMANTIC_LEVEL,
        SEMANTIC_POSITION_IN_SET,
        SEMANTIC_SET_SIZE,
        SEMANTIC_MODAL,
        SEMANTIC_HAS_POPUP,
        SEMANTIC_SORT,
        SEMANTIC_SELECTED,
        CHECKED,
        SEMANTIC_CHECKED_STATE,
        EXPANDED,
        BUSY,
        SEMANTIC_CURRENT,
        SEMANTIC_PRESSED,
        SEMANTIC_REQUIRED,
        SEMANTIC_READ_ONLY,
        SEMANTIC_MULTISELECTABLE,
        SEMANTIC_INVALID,
        SEMANTIC_LIVE,
        SEMANTIC_CONTROLS,
        SEMANTIC_ACTIVE_DESCENDANT,
        SEMANTIC_LABELLED_BY,
        SEMANTIC_DESCRIBED_BY,
        SEMANTIC_CAN_INCREMENT,
        SEMANTIC_CAN_DECREMENT,
        SEMANTIC_CAN_SET_VALUE,
        SEMANTIC_CAN_EXPAND,
        SEMANTIC_CAN_COLLAPSE,
        SEMANTIC_CAN_SCROLL_INTO_VIEW,
    ];
    let authored = input
        .properties
        .iter()
        .any(|(id, _)| SEMANTIC_IDS.contains(id));
    if !authored
        && !input
            .events
            .iter()
            .any(|event| event.id == CLICK || event.id == SEMANTIC_ACTION)
    {
        return Ok(element);
    }
    let mut semantics = element.semantics.as_deref().cloned().unwrap_or_default();
    if let Some(value) = string(input, SEMANTIC_ROLE) {
        semantics.role = focus_scope_parse::parse_role(value)?;
    }
    if let Some(value) = string(input, SEMANTIC_LABEL) {
        semantics.label = Some(value.to_owned());
    }
    if let Some(value) = string(input, SEMANTIC_DESCRIPTION) {
        semantics.description = Some(value.to_owned());
    }
    if let Some(value) = string(input, SEMANTIC_VALUE) {
        semantics.value = Some(SemanticValue::Text(value.to_owned()));
    }
    let numeric = number(input, SEMANTIC_NUMERIC_VALUE);
    let minimum = number(input, SEMANTIC_MINIMUM_VALUE);
    let maximum = number(input, SEMANTIC_MAXIMUM_VALUE);
    let step = number(input, SEMANTIC_VALUE_STEP);
    if numeric.is_none() && (minimum.is_some() || maximum.is_some() || step.is_some()) {
        return Err(SchemaError::Adapter(
            "numeric_value is required when numeric bounds or step are set".into(),
        ));
    }
    if [numeric, minimum, maximum, step]
        .into_iter()
        .flatten()
        .any(|v| !v.is_finite())
        || minimum.zip(maximum).is_some_and(|(low, high)| low > high)
        || step.is_some_and(|value| value <= 0.0)
    {
        return Err(SchemaError::Adapter(
            "invalid numeric accessibility range".into(),
        ));
    }
    if let Some(value) = numeric {
        if input.get(SEMANTIC_VALUE).is_some() {
            return Err(SchemaError::Adapter(
                "accessible_value and numeric_value are mutually exclusive".into(),
            ));
        }
        semantics.value = Some(SemanticValue::Number {
            value,
            minimum,
            maximum,
            step,
        });
    }
    if let Some(value) = boolean(input, SEMANTIC_DISABLED) {
        semantics.state.disabled |= value;
        if value {
            let interaction = element
                .interaction
                .take()
                .unwrap_or_default()
                .enabled(false);
            element = element.interaction(interaction);
        }
    }
    if let Some(value) = boolean(input, SEMANTIC_HIDDEN) {
        element = element.semantic_hidden(value);
    }
    for (property, target) in [
        (SEMANTIC_SELECTED, &mut semantics.state.selected),
        (SEMANTIC_REQUIRED, &mut semantics.state.required),
        (SEMANTIC_READ_ONLY, &mut semantics.state.read_only),
        (
            SEMANTIC_MULTISELECTABLE,
            &mut semantics.state.multiselectable,
        ),
        (SEMANTIC_INVALID, &mut semantics.state.invalid),
        (SEMANTIC_MODAL, &mut semantics.state.modal),
    ] {
        if let Some(value) = boolean(input, property) {
            *target = value;
        }
    }
    if let Some(value) = boolean(input, SEMANTIC_PRESSED) {
        semantics.state.pressed = Some(value);
    }
    if let Some(value) = boolean(input, CHECKED) {
        semantics.state.checked = Some(if value {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        });
    }
    if let Some(value) = string(input, SEMANTIC_CHECKED_STATE) {
        if input.get(CHECKED).is_some() {
            return Err(SchemaError::Adapter(
                "checked and checked_state are mutually exclusive".into(),
            ));
        }
        semantics.state.checked = Some(match value {
            "unchecked" => CheckedState::Unchecked,
            "checked" => CheckedState::Checked,
            "mixed" => CheckedState::Mixed,
            _ => {
                return Err(SchemaError::Adapter(format!(
                    "unknown checked_state `{value}`"
                )));
            }
        });
    }
    if let Some(value) = boolean(input, EXPANDED) {
        semantics.state.expanded = Some(value);
    }
    if let Some(value) = boolean(input, BUSY) {
        semantics.state.busy = value;
    }
    if let Some(value) = string(input, SEMANTIC_CURRENT) {
        semantics.state.current = Some(focus_scope_parse::parse_current(value)?);
    }
    if let Some(value) = string(input, SEMANTIC_LIVE) {
        semantics.live = focus_scope_parse::parse_live(value)?;
    }
    if let Some(value) = string(input, SEMANTIC_ORIENTATION) {
        semantics.orientation = Some(match value {
            "horizontal" => Orientation::Horizontal,
            "vertical" => Orientation::Vertical,
            _ => {
                return Err(SchemaError::Adapter(format!(
                    "unknown orientation `{value}`"
                )));
            }
        });
    }
    for (id, target) in [
        (SEMANTIC_LEVEL, &mut semantics.level),
        (SEMANTIC_POSITION_IN_SET, &mut semantics.position_in_set),
        (SEMANTIC_SET_SIZE, &mut semantics.set_size),
    ] {
        if let Some(SchemaValue::Int(value)) = input.get(id) {
            *target = Some(
                u32::try_from(*value)
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        SchemaError::Adapter("semantic index must be positive".into())
                    })?,
            );
        }
    }
    if let Some(value) = string(input, SEMANTIC_HAS_POPUP) {
        semantics.popup = Some(match value {
            "menu" => PopupKind::Menu,
            "listBox" => PopupKind::ListBox,
            "tree" => PopupKind::Tree,
            "grid" => PopupKind::Grid,
            "dialog" => PopupKind::Dialog,
            _ => return Err(SchemaError::Adapter(format!("unknown has_popup `{value}`"))),
        });
    }
    if let Some(value) = string(input, SEMANTIC_SORT) {
        semantics.sort = Some(match value {
            "ascending" => SortDirection::Ascending,
            "descending" => SortDirection::Descending,
            _ => return Err(SchemaError::Adapter(format!("unknown sort `{value}`"))),
        });
    }
    let focusable = boolean(input, SEMANTIC_FOCUSABLE);
    let tab_stop = boolean(input, FOCUS_ON_TAB).unwrap_or(true);
    let activation = string(input, KEYBOARD_ACTIVATION)
        .map(focus_scope_parse::parse_activation)
        .transpose()?;
    if focusable.is_some() || input.get(FOCUS_ON_TAB).is_some() || activation.is_some() {
        let mut interaction = element.interaction.take().unwrap_or_default();
        if let Some(value) = focusable {
            interaction.focus_policy = if value {
                if tab_stop {
                    FocusPolicy::TabStop
                } else {
                    FocusPolicy::Programmatic
                }
            } else {
                FocusPolicy::None
            };
            if !value {
                semantics
                    .actions
                    .retain(|action| *action != SemanticAction::Focus);
            }
        } else if input.get(FOCUS_ON_TAB).is_some() && interaction.focus_policy.is_focusable() {
            interaction.focus_policy = if tab_stop {
                FocusPolicy::TabStop
            } else {
                FocusPolicy::Programmatic
            };
        }
        if let Some(value) = activation {
            interaction.keyboard_activation = value;
        }
        if interaction.focus_policy.is_focusable() && interaction.enabled {
            semantics = semantics.action(SemanticAction::Focus);
        }
        element = element.interaction(interaction);
    }
    let enabled = !semantics.state.disabled
        && element
            .interaction
            .as_ref()
            .is_none_or(|interaction| interaction.enabled);
    if enabled
        && (activation.is_some_and(|value| value != KeyboardActivation::None)
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
        match boolean(input, property) {
            Some(true)
                if enabled
                    && (!semantics.state.read_only || action != SemanticAction::SetValue) =>
            {
                semantics = semantics.action(action);
            }
            Some(false) => semantics.actions.retain(|existing| *existing != action),
            _ => {}
        }
    }
    if semantics.state.read_only {
        semantics.actions.retain(|action| {
            !matches!(
                action,
                SemanticAction::SetValue | SemanticAction::Increment | SemanticAction::Decrement
            )
        });
    }
    if !enabled {
        semantics.actions.clear();
    }
    if authored || element.semantics.is_some() {
        element = element.semantics(semantics);
    }
    if let Some(key) = string(input, SEMANTIC_CONTROLS) {
        element = element.controls(key.split_whitespace());
    }
    if let Some(key) = string(input, SEMANTIC_ACTIVE_DESCENDANT) {
        element = element.active_descendant(key);
    }
    if let Some(key) = string(input, SEMANTIC_LABELLED_BY) {
        element = element.labelled_by(key.split_whitespace());
    }
    if let Some(key) = string(input, SEMANTIC_DESCRIBED_BY) {
        element = element.described_by(key.split_whitespace());
    }
    for event in &input.events {
        let kind = match event.id {
            CLICK => EventType::Click,
            SEMANTIC_ACTION => EventType::SemanticAction,
            _ => continue,
        };
        if !element
            .event_listeners
            .iter()
            .any(|listener| listener.event == kind)
        {
            element = element.on(event.handler.listener(kind));
        }
    }
    Ok(element)
}

/// Reads an optional string from validated native `input` at `id`.
fn string(input: &NativeElementInput, id: PropertyId) -> Option<&str> {
    match input.get(id) {
        Some(SchemaValue::String(value)) => Some(value),
        _ => None,
    }
}

/// Reads an optional boolean from validated native `input` at `id`.
fn boolean(input: &NativeElementInput, id: PropertyId) -> Option<bool> {
    match input.get(id) {
        Some(SchemaValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

/// Reads an optional finite-width numeric value from validated native `input` at `id`.
fn number(input: &NativeElementInput, id: PropertyId) -> Option<f64> {
    match input.get(id) {
        Some(SchemaValue::Float(value)) => Some(f64::from(*value)),
        _ => None,
    }
}
