//! Validated GPU brush values accepted from TSX presentation code.

use argui_core::{Color, Point};
use argui_paint::Fill;
use serde_json::Value;

use super::number;

/// Decodes a solid color or bounded gradient; returns `None` for malformed input.
///
/// `value` is the JSON payload carried by a `Brush` wire value.
pub(super) fn brush(value: &Value) -> Option<Fill> {
    if let Some(literal) = value.as_str() {
        return Color::from_literal(literal).ok().map(Fill::Solid);
    }
    let object = value.as_object()?;
    let stops = object.get("stops")?.as_array()?;
    if !(2..=64).contains(&stops.len()) {
        return None;
    }
    let mut colors = Vec::with_capacity(stops.len());
    let mut offsets = Vec::with_capacity(stops.len());
    for stop in stops {
        let stop = stop.as_object()?;
        if stop.len() != 2 {
            return None;
        }
        offsets.push(number(stop.get("offset")?)?);
        colors.push(Color::from_literal(stop.get("color")?.as_str()?).ok()?);
    }
    let space = match object.get("space") {
        Some(value) => value.as_str()?,
        None => "oklab",
    };
    match object.get("kind")?.as_str()? {
        "linear" if fields(object, &["kind", "angle", "stops", "space"]) => {
            Fill::linear_gradient(&colors, &offsets, number(object.get("angle")?)?, space).ok()
        }
        "radial" if fields(object, &["kind", "center", "radius", "stops", "space"]) => {
            let center = point(object.get("center")?)?;
            let radius = point(object.get("radius")?)?;
            if radius.x <= 0.0 || radius.y <= 0.0 {
                return None;
            }
            Fill::radial_gradient(&colors, &offsets, center, radius, space).ok()
        }
        _ => None,
    }
}

/// Reads a finite two-dimensional point from `value`, or returns `None`.
fn point(value: &Value) -> Option<Point> {
    let object = value.as_object()?;
    if object.len() != 2 {
        return None;
    }
    Some(Point::new(
        number(object.get("x")?)?,
        number(object.get("y")?)?,
    ))
}

/// Returns whether `object` contains only the supported field `names`.
fn fields(object: &serde_json::Map<String, Value>, names: &[&str]) -> bool {
    object.keys().all(|name| names.contains(&name.as_str()))
}
