//! Shared accessibility property and event metadata.

use super::*;
use crate::EventSchema;

/// Returns semantic metadata appended to each built-in primitive's schema.
///
/// Existing native properties with the same name or ID take precedence.
pub(crate) fn properties() -> Vec<PropertySchema> {
    [
        (SEMANTIC_ROLE, "role", ValueType::String, "Accessible role."),
        (
            SEMANTIC_LABEL,
            "accessible_name",
            ValueType::String,
            "Accessible name.",
        ),
        (
            SEMANTIC_DESCRIPTION,
            "accessible_description",
            ValueType::String,
            "Accessible description.",
        ),
        (
            SEMANTIC_VALUE,
            "accessible_value",
            ValueType::String,
            "Accessible text value.",
        ),
        (
            SEMANTIC_NUMERIC_VALUE,
            "numeric_value",
            ValueType::Float,
            "Accessible numeric value.",
        ),
        (
            SEMANTIC_MINIMUM_VALUE,
            "minimum_value",
            ValueType::Float,
            "Minimum numeric value.",
        ),
        (
            SEMANTIC_MAXIMUM_VALUE,
            "maximum_value",
            ValueType::Float,
            "Maximum numeric value.",
        ),
        (
            SEMANTIC_VALUE_STEP,
            "value_step",
            ValueType::Float,
            "Numeric value increment.",
        ),
        (
            SEMANTIC_DISABLED,
            "accessible_disabled",
            ValueType::Bool,
            "Disabled semantic and interaction state.",
        ),
        (
            SEMANTIC_HIDDEN,
            "accessible_hidden",
            ValueType::Bool,
            "Hide this semantic subtree.",
        ),
        (
            SEMANTIC_FOCUSABLE,
            "focusable",
            ValueType::Bool,
            "Permit focus on this element.",
        ),
        (
            FOCUS_ON_TAB,
            "focus_on_tab_navigation",
            ValueType::Bool,
            "Include a focusable element in Tab order.",
        ),
        (
            KEYBOARD_ACTIVATION,
            "keyboard_activation",
            ValueType::String,
            "Keyboard click policy.",
        ),
        (
            SEMANTIC_SELECTED,
            "selected",
            ValueType::Bool,
            "Selected state.",
        ),
        (CHECKED, "checked", ValueType::Bool, "Checked state."),
        (
            SEMANTIC_CHECKED_STATE,
            "checked_state",
            ValueType::String,
            "Unchecked, checked, or mixed state.",
        ),
        (EXPANDED, "expanded", ValueType::Bool, "Expanded state."),
        (BUSY, "busy", ValueType::Bool, "Busy state."),
        (
            SEMANTIC_CURRENT,
            "current",
            ValueType::String,
            "Current item kind.",
        ),
        (
            SEMANTIC_PRESSED,
            "pressed_state",
            ValueType::Bool,
            "Persistent toggle state.",
        ),
        (
            SEMANTIC_REQUIRED,
            "required",
            ValueType::Bool,
            "Required value state.",
        ),
        (
            SEMANTIC_READ_ONLY,
            "read_only",
            ValueType::Bool,
            "Read-only value state.",
        ),
        (
            SEMANTIC_MULTISELECTABLE,
            "multiselectable",
            ValueType::Bool,
            "Multiple selection state.",
        ),
        (
            SEMANTIC_INVALID,
            "invalid",
            ValueType::Bool,
            "Invalid value state.",
        ),
        (
            SEMANTIC_LIVE,
            "live",
            ValueType::String,
            "Announcement policy.",
        ),
        (
            SEMANTIC_MODAL,
            "modal",
            ValueType::Bool,
            "Modal dialog state.",
        ),
        (
            SEMANTIC_ORIENTATION,
            "orientation",
            ValueType::String,
            "Horizontal or vertical orientation.",
        ),
        (
            SEMANTIC_LEVEL,
            "level",
            ValueType::Int,
            "One-based heading or hierarchy level.",
        ),
        (
            SEMANTIC_POSITION_IN_SET,
            "position_in_set",
            ValueType::Int,
            "One-based item position.",
        ),
        (
            SEMANTIC_SET_SIZE,
            "set_size",
            ValueType::Int,
            "Number of items in the set.",
        ),
        (
            SEMANTIC_HAS_POPUP,
            "has_popup",
            ValueType::String,
            "Kind of controlled popup.",
        ),
        (SEMANTIC_SORT, "sort", ValueType::String, "Sort direction."),
        (
            SEMANTIC_CONTROLS,
            "controls",
            ValueType::String,
            "Key of controlled element.",
        ),
        (
            SEMANTIC_ACTIVE_DESCENDANT,
            "active_descendant",
            ValueType::String,
            "Key of active descendant.",
        ),
        (
            SEMANTIC_LABELLED_BY,
            "labelled_by",
            ValueType::String,
            "Key of naming element.",
        ),
        (
            SEMANTIC_DESCRIBED_BY,
            "described_by",
            ValueType::String,
            "Key of describing element.",
        ),
        (
            SEMANTIC_CAN_INCREMENT,
            "can_increment",
            ValueType::Bool,
            "Expose increment action.",
        ),
        (
            SEMANTIC_CAN_DECREMENT,
            "can_decrement",
            ValueType::Bool,
            "Expose decrement action.",
        ),
        (
            SEMANTIC_CAN_SET_VALUE,
            "can_set_value",
            ValueType::Bool,
            "Expose set-value action.",
        ),
        (
            SEMANTIC_CAN_EXPAND,
            "can_expand",
            ValueType::Bool,
            "Expose expand action.",
        ),
        (
            SEMANTIC_CAN_COLLAPSE,
            "can_collapse",
            ValueType::Bool,
            "Expose collapse action.",
        ),
        (
            SEMANTIC_CAN_SCROLL_INTO_VIEW,
            "can_scroll_into_view",
            ValueType::Bool,
            "Expose scroll action.",
        ),
    ]
    .into_iter()
    .map(|(id, name, ty, description)| {
        PropertySchema::new(id, name, ty, description).not_animatable()
    })
    .collect()
}

/// Returns accessible action events available on every built-in primitive.
pub(crate) fn events() -> [EventSchema; 2] {
    [
        common_event(CLICK, "click", EventType::Click),
        common_event(
            SEMANTIC_ACTION,
            "semantic_action",
            EventType::SemanticAction,
        ),
    ]
}
