//! Native one-shot popup entrances with reduced-motion support.

use super::{OPENING_MS, OPENING_SCALE, OPENING_TRANSLATE_Y};
use crate::{NativeElementInput, PropertyId, PropertySchema, SchemaError, SchemaValue, ValueType};
use argui_animation::{
    CubicBezier, Duration, Easing, FillMode, Interpolate, Keyframe, Keyframes, Motion, Timeline,
    Timing,
};
use argui_ui::{Element, property};

pub(super) fn properties() -> [PropertySchema; 3] {
    [
        PropertySchema::new(OPENING_MS, "openingMs", ValueType::Float,
            "Duration of a one-shot fade and transform entrance, 1–60000 milliseconds; omitted for instant presentation.").not_animatable(),
        PropertySchema::new(OPENING_SCALE, "openingScale", ValueType::Float,
            "Positive starting scale for the popup entrance, pivoted at its anchor; defaults to 1.").not_animatable(),
        PropertySchema::new(OPENING_TRANSLATE_Y, "openingTranslateY", ValueType::Float,
            "Starting vertical offset in logical pixels when below its anchor; reversed above it; defaults to 0.").not_animatable(),
    ]
}

pub(super) fn apply(
    mut element: Element,
    input: &NativeElementInput,
) -> Result<Element, SchemaError> {
    let Some(milliseconds) = number(input, OPENING_MS) else {
        if number(input, OPENING_SCALE).is_some() || number(input, OPENING_TRANSLATE_Y).is_some() {
            return Err(SchemaError::Adapter(
                "PopupWindow opening transform requires openingMs".into(),
            ));
        }
        return Ok(element);
    };
    if !milliseconds.is_finite() || !(1.0..=60_000.0).contains(&milliseconds) {
        return Err(SchemaError::Adapter(
            "PopupWindow requires openingMs between 1 and 60000".into(),
        ));
    }
    let scale = number(input, OPENING_SCALE).unwrap_or(1.0);
    let y = number(input, OPENING_TRANSLATE_Y).unwrap_or(0.0);
    if !scale.is_finite() || scale <= 0.0 || !y.is_finite() {
        return Err(SchemaError::Adapter(
            "PopupWindow openingScale must be positive and opening transforms finite".into(),
        ));
    }
    let target = element.transform;
    let mut from = target;
    from.scale.x *= scale;
    from.scale.y *= scale;
    from.translation.y += y;
    let opacity = element.layer.as_ref().map_or(1.0, |layer| layer.opacity);
    if let Some(portal) = &mut element.portal {
        portal.transform_from_anchor = true;
    }
    element = element.bind(
        property::LayerOpacity,
        motion(0.0, opacity, milliseconds as u64)?,
    );
    if from != target {
        element = element.bind(
            property::Transform,
            motion(from, target, milliseconds as u64)?,
        );
    }
    Ok(element)
}

fn number(input: &NativeElementInput, id: PropertyId) -> Option<f32> {
    match input.get(id) {
        Some(SchemaValue::Float(value)) => Some(*value),
        _ => None,
    }
}

fn motion<T: Clone + Interpolate>(
    from: T,
    to: T,
    milliseconds: u64,
) -> Result<Motion<T>, SchemaError> {
    let easing = Easing::CubicBezier(
        CubicBezier::new(0.2, 0.0, 0.0, 1.0).expect("constant CSS Bézier is valid"),
    );
    let frames = Keyframes::new([
        Keyframe::new(0.0, from.clone()).easing(easing),
        Keyframe::new(1.0, to),
    ])
    .map_err(|error| SchemaError::Adapter(error.to_string()))?;
    let timeline = Timeline::new(
        frames,
        Timing::new(Duration::from_millis(milliseconds)).fill(FillMode::Both),
    )
    .map_err(|error| SchemaError::Adapter(error.to_string()))?;
    let motion = Motion::new(from);
    motion.play(timeline);
    Ok(motion)
}
