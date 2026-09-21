//! Gradient-specific transition values and application.

use argui_core::Point;
use argui_paint::{Fill, GradientStop, GradientStops};

use crate::binding::GradientPointTarget;
use crate::state::StateValue;
use crate::{PropertyKey, StylePropertyValue};

use super::{ResolvedProperty, value};

/// Collects animatable geometry and stops from a gradient fill.
///
/// `background` is the source fill, and `values` receives its animatable properties.
pub(super) fn gradient_values(background: &Fill, values: &mut Vec<StylePropertyValue>) {
    if let Fill::Conic(gradient) = background {
        values.push(value(
            PropertyKey::GradientPoint(GradientPointTarget::ConicCenter),
            StateValue::Point(gradient.center),
        ));
        values.push(value(
            PropertyKey::GradientAngle,
            StateValue::F32(gradient.start_angle),
        ));
        for (index, stop) in gradient.stops.as_slice().iter().enumerate() {
            values.push(value(
                PropertyKey::GradientStopOffset(index),
                StateValue::F32(stop.offset),
            ));
            values.push(value(
                PropertyKey::GradientStopColor(index),
                StateValue::Color(stop.color),
            ));
        }
        return;
    }
    let (first, second, stops) = match background {
        Fill::Linear(gradient) => (
            (GradientPointTarget::LinearStart, gradient.start),
            (GradientPointTarget::LinearEnd, gradient.end),
            &gradient.stops,
        ),
        Fill::Radial(gradient) => (
            (GradientPointTarget::RadialCenter, gradient.center),
            (GradientPointTarget::RadialRadius, gradient.radius),
            &gradient.stops,
        ),
        Fill::Solid(_) | Fill::Bilinear(_) | Fill::Conic(_) => return,
    };
    values.push(value(
        PropertyKey::GradientPoint(first.0),
        StateValue::Point(first.1),
    ));
    values.push(value(
        PropertyKey::GradientPoint(second.0),
        StateValue::Point(second.1),
    ));
    for (index, stop) in stops.as_slice().iter().enumerate() {
        values.push(value(
            PropertyKey::GradientStopOffset(index),
            StateValue::F32(stop.offset),
        ));
        values.push(value(
            PropertyKey::GradientStopColor(index),
            StateValue::Color(stop.color),
        ));
    }
}

/// Reports whether `key` addresses gradient geometry or stops.
pub(super) fn is_gradient_property(key: &PropertyKey) -> bool {
    matches!(
        key,
        PropertyKey::GradientPoint(_)
            | PropertyKey::GradientAngle
            | PropertyKey::GradientStopOffset(_)
            | PropertyKey::GradientStopColor(_)
    )
}

/// Removes old gradient entries from `values` before replacing the fill.
pub(super) fn remove_gradient_values(values: &mut Vec<ResolvedProperty>) {
    values.retain(|value| !is_gradient_property(&value.property.key));
}

/// Applies one animated point to a compatible gradient fill.
///
/// `fill` is unchanged when its variant does not match `target`.
pub(super) fn apply_gradient_point(
    fill: &mut Option<Fill>,
    target: GradientPointTarget,
    value: Point,
) {
    match (fill, target) {
        (Some(Fill::Linear(gradient)), GradientPointTarget::LinearStart) => gradient.start = value,
        (Some(Fill::Linear(gradient)), GradientPointTarget::LinearEnd) => gradient.end = value,
        (Some(Fill::Radial(gradient)), GradientPointTarget::RadialCenter) => {
            gradient.center = value;
        }
        (Some(Fill::Radial(gradient)), GradientPointTarget::RadialRadius) => {
            gradient.radius = value;
        }
        (Some(Fill::Conic(gradient)), GradientPointTarget::ConicCenter) => {
            gradient.center = value;
        }
        _ => {}
    }
}

/// Applies `apply` to one stop of a linear or radial gradient.
///
/// Invalid stop indices or resulting stop arrays leave `fill` unchanged.
pub(super) fn apply_gradient_stop(
    fill: &mut Option<Fill>,
    index: usize,
    apply: impl FnOnce(&mut GradientStop),
) {
    let stops = match fill {
        Some(Fill::Linear(gradient)) => &mut gradient.stops,
        Some(Fill::Radial(gradient)) => &mut gradient.stops,
        _ => return,
    };
    let mut resolved = stops.as_slice().to_vec();
    let Some(stop) = resolved.get_mut(index) else {
        return;
    };
    apply(stop);
    if let Ok(resolved) = GradientStops::from_vec(resolved) {
        *stops = resolved;
    }
}
