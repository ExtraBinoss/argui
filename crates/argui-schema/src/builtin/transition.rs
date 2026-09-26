//! Native target transitions shared by visual primitives.

use argui_animation::{CubicBezier, Duration, Easing, Transition, Tween};
use argui_ui::{Element, StyleTransition};

use super::{TRANSITION_MS, TRANSITION_SPRING, TRANSITION_TIMING_FUNCTION};
use crate::{NativeElementInput, PropertySchema, SchemaError, SchemaValue, ValueType};

/// Declares timed tweens, their easing, and the mutually exclusive spring driver.
#[must_use]
pub(super) fn properties() -> [PropertySchema; 3] {
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
        PropertySchema::new(
            TRANSITION_TIMING_FUNCTION,
            "transitionTimingFunction",
            ValueType::String,
            "CSS linear or cubic-bezier timing function for a timed native transition.",
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
/// Returns for conflicting drivers, invalid easing, or a duration outside 1–60,000 ms.
pub(super) fn apply(
    mut element: Element,
    input: &NativeElementInput,
    name: &str,
) -> Result<Element, SchemaError> {
    let spring = matches!(input.get(TRANSITION_SPRING), Some(SchemaValue::Bool(true)));
    let timing = match input.get(TRANSITION_TIMING_FUNCTION) {
        Some(SchemaValue::String(value)) => Some(parse_timing_function(value)?),
        _ => None,
    };
    if spring && input.get(TRANSITION_MS).is_some() {
        return Err(SchemaError::Adapter(format!(
            "{name} cannot combine transition_ms and transition_spring"
        )));
    }
    if timing.is_some() && (spring || input.get(TRANSITION_MS).is_none()) {
        return Err(SchemaError::Adapter(format!(
            "{name} transitionTimingFunction requires transitionMs without transitionSpring"
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
        element = element.transition(StyleTransition::new(Transition::tween(
            Tween::new(Duration::from_millis(milliseconds.round() as u64))
                .easing(timing.unwrap_or(Easing::Linear)),
        )));
    }
    Ok(element)
}

/// Parses a CSS timing function used by a timed native transition.
///
/// * `value` — `linear` or a four-coordinate `cubic-bezier(...)` value.
///
/// # Errors
///
/// Returns a schema error when the name or Bézier coordinates are invalid.
fn parse_timing_function(value: &str) -> Result<Easing, SchemaError> {
    if value == "linear" {
        return Ok(Easing::Linear);
    }
    let coordinates = value
        .strip_prefix("cubic-bezier(")
        .and_then(|inner| inner.strip_suffix(')'))
        .and_then(|inner| {
            inner
                .split(',')
                .map(|part| part.trim().parse::<f32>().ok())
                .collect::<Option<Vec<_>>>()
        })
        .filter(|parts| parts.len() == 4)
        .ok_or_else(|| SchemaError::Adapter("invalid transitionTimingFunction".into()))?;
    let curve = CubicBezier::new(
        coordinates[0],
        coordinates[1],
        coordinates[2],
        coordinates[3],
    )
    .map_err(|_| SchemaError::Adapter("invalid transitionTimingFunction".into()))?;
    Ok(Easing::CubicBezier(curve))
}
