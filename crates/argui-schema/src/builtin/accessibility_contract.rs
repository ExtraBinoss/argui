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
            "accessibleName",
            ValueType::String,
            "Accessible name.",
        ),
        (
            SEMANTIC_DESCRIPTION,
            "accessibleDescription",
            ValueType::String,
            "Accessible description.",
        ),
        (
            SEMANTIC_VALUE,
            "accessibleValue",
            ValueType::String,
            "Accessible text value.",
        ),
        (
            SEMANTIC_NUMERIC_VALUE,
            "numericValue",
            ValueType::Float,
            "Accessible numeric value.",
        ),
        (
            SEMANTIC_MINIMUM_VALUE,
            "minimumValue",
            ValueType::Float,
            "Minimum numeric value.",
        ),
        (
            SEMANTIC_MAXIMUM_VALUE,
            "maximumValue",
            ValueType::Float,
            "Maximum numeric value.",
        ),
        (
            SEMANTIC_VALUE_STEP,
            "valueStep",
            ValueType::Float,
            "Numeric value increment.",
        ),
        (
            SEMANTIC_DISABLED,
            "accessibleDisabled",
            ValueType::Bool,
            "Disabled semantic and interaction state.",
        ),
        (
            SEMANTIC_HIDDEN,
            "accessibleHidden",
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
            "focusOnTabNavigation",
            ValueType::Bool,
            "Include a focusable element in Tab order.",
        ),
        (
            KEYBOARD_ACTIVATION,
            "keyboardActivation",
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
            "checkedState",
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
            "pressedState",
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
            "readOnly",
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
            "positionInSet",
            ValueType::Int,
            "One-based item position.",
        ),
        (
            SEMANTIC_SET_SIZE,
            "setSize",
            ValueType::Int,
            "Number of items in the set.",
        ),
        (
            SEMANTIC_HAS_POPUP,
            "hasPopup",
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
            "activeDescendant",
            ValueType::String,
            "Key of active descendant.",
        ),
        (
            SEMANTIC_LABELLED_BY,
            "labelledBy",
            ValueType::String,
            "Key of naming element.",
        ),
        (
            SEMANTIC_DESCRIBED_BY,
            "describedBy",
            ValueType::String,
            "Key of describing element.",
        ),
        (
            SEMANTIC_CAN_INCREMENT,
            "canIncrement",
            ValueType::Bool,
            "Expose increment action.",
        ),
        (
            SEMANTIC_CAN_DECREMENT,
            "canDecrement",
            ValueType::Bool,
            "Expose decrement action.",
        ),
        (
            SEMANTIC_CAN_SET_VALUE,
            "canSetValue",
            ValueType::Bool,
            "Expose set-value action.",
        ),
        (
            SEMANTIC_CAN_EXPAND,
            "canExpand",
            ValueType::Bool,
            "Expose expand action.",
        ),
        (
            SEMANTIC_CAN_COLLAPSE,
            "canCollapse",
            ValueType::Bool,
            "Expose collapse action.",
        ),
        (
            SEMANTIC_CAN_SCROLL_INTO_VIEW,
            "canScrollIntoView",
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
        common_event(SEMANTIC_ACTION, "semanticAction", EventType::SemanticAction),
    ]
}
