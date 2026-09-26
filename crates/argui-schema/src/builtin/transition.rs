//! Native target transitions shared by visual primitives.

use argui_animation::{Duration, Transition, Tween};
use argui_ui::{Element, StyleTransition};

use super::{TRANSITION_MS, TRANSITION_SPRING};
use crate::{NativeElementInput, PropertySchema, SchemaError, SchemaValue, ValueType};

/// Declares the mutually exclusive tween and spring target drivers.
#[must_use]
pub(super) fn properties() -> [PropertySchema; 2] {
    [
        PropertySchema::new(
            TRANSITION_MS,
            "transitionMs",
            ValueType::Float,
            "Duration of native visual transitions when authored target values change.",
        ),
        PropertySchema::new(
            TRANSITION_SPRING,
            "transitionSpring",
            ValueType::Bool,
            "Retarget authored visual values with the native damped spring driver.",
        ),
    ]
}

/// Installs a validated native target transition on an element.
///
/// * `element` — visual element whose authored values may change.
/// * `input` — validated native properties.
/// * `name` — primitive name used in errors.
///
/// # Errors
///
/// Returns for conflicting drivers or a duration outside 1–60,000 ms.
pub(super) fn apply(
    mut element: Element,
    input: &NativeElementInput,
    name: &str,
) -> Result<Element, SchemaError> {
    let spring = matches!(input.get(TRANSITION_SPRING), Some(SchemaValue::Bool(true)));
    if spring && input.get(TRANSITION_MS).is_some() {
        return Err(SchemaError::Adapter(format!(
            "{name} cannot combine transition_ms and transition_spring"
        )));
    }
    if spring {
        element = element.transition(StyleTransition::new(Transition::spring()));
    }
    if let Some(SchemaValue::Float(milliseconds)) = input.get(TRANSITION_MS) {
        if !milliseconds.is_finite() || !(1.0..=60_000.0).contains(milliseconds) {
            return Err(SchemaError::Adapter(format!(
                "{name} requires transition_ms between 1 and 60000"
            )));
        }
        element = element.transition(StyleTransition::new(Transition::tween(Tween::new(
            Duration::from_millis(milliseconds.round() as u64),
        ))));
    }
    Ok(element)
}
