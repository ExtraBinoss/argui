//! Declarative opt-in to native, commit-only resizing on TouchArea.

use argui_ui::{ResizeAxis, ResizeHandle};

use super::{RESIZE_AXIS, RESIZE_MAXIMUM, RESIZE_MINIMUM, RESIZE_TARGET, RESIZE_TRAILING};
use crate::{NativeElementInput, PropertySchema, SchemaError, SchemaValue, ValueType};

/// Describes the five native resize properties; absent target disables resizing.
pub(super) fn properties() -> [PropertySchema; 5] {
    [
        PropertySchema::new(
            RESIZE_TARGET,
            "resizeTarget",
            ValueType::String,
            "Unique public ID of the pane resized directly by the engine.",
        ),
        PropertySchema::new(
            RESIZE_AXIS,
            "resizeAxis",
            ValueType::String,
            "Pointer coordinate and target dimension controlled by native resizing.",
        )
        .allowed_values(&["horizontal", "vertical"]),
        PropertySchema::new(
            RESIZE_MINIMUM,
            "resizeMinimum",
            ValueType::Float,
            "Minimum pane size in logical pixels.",
        ),
        PropertySchema::new(
            RESIZE_MAXIMUM,
            "resizeMaximum",
            ValueType::Float,
            "Maximum pane size in logical pixels.",
        ),
        PropertySchema::new(
            RESIZE_TRAILING,
            "resizeTrailing",
            ValueType::Bool,
            "Reverse pointer displacement for a pane after the divider.",
        ),
    ]
}

/// Parses `input` into native resize configuration, or `None` for an ordinary area.
///
/// # Errors
/// Returns an adapter error for incomplete configuration, an empty target, or invalid bounds.
pub(super) fn parse(input: &NativeElementInput) -> Result<Option<ResizeHandle>, SchemaError> {
    let Some(SchemaValue::String(target)) = input.get(RESIZE_TARGET) else {
        if [RESIZE_AXIS, RESIZE_MINIMUM, RESIZE_MAXIMUM, RESIZE_TRAILING]
            .iter()
            .any(|id| input.get(*id).is_some())
        {
            return Err(SchemaError::Adapter(
                "TouchArea resize properties require resizeTarget".into(),
            ));
        }
        return Ok(None);
    };
    let axis = match input.get(RESIZE_AXIS) {
        Some(SchemaValue::String(axis)) if axis == "horizontal" => ResizeAxis::Horizontal,
        Some(SchemaValue::String(axis)) if axis == "vertical" => ResizeAxis::Vertical,
        _ => {
            return Err(SchemaError::Adapter(
                "TouchArea resizing requires resizeAxis".into(),
            ));
        }
    };
    let (Some(SchemaValue::Float(minimum)), Some(SchemaValue::Float(maximum))) =
        (input.get(RESIZE_MINIMUM), input.get(RESIZE_MAXIMUM))
    else {
        return Err(SchemaError::Adapter(
            "TouchArea resizing requires resizeMinimum and resizeMaximum".into(),
        ));
    };
    if target.trim().is_empty()
        || !minimum.is_finite()
        || !maximum.is_finite()
        || *minimum < 0.0
        || maximum < minimum
    {
        return Err(SchemaError::Adapter("TouchArea resize target must be nonempty and bounds finite with 0 <= minimum <= maximum".into()));
    }
    Ok(Some(ResizeHandle {
        target: target.clone(),
        axis,
        minimum: *minimum,
        maximum: *maximum,
        trailing: matches!(input.get(RESIZE_TRAILING), Some(SchemaValue::Bool(true))),
    }))
}
