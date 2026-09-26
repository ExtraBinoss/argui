//! Display-linked property loops for native visual primitives.

use argui_animation::{
    CubicBezier, Direction, Duration, Easing, Interpolate, Iterations, Keyframe, Keyframes, Motion,
    Timeline, Timing,
};
use argui_core::Transform2D;
use argui_paint::Fill;
use argui_ui::{Element, ExpandedDimension, property};

use super::{
    BACKGROUND, GAP, LOOP_BACKGROUND, LOOP_GAP, LOOP_HOLD, LOOP_MS, LOOP_OPACITY, LOOP_PLAYING,
    LOOP_RADIUS, LOOP_SCALE, LOOP_TRANSLATE_X, LOOP_TRANSLATE_Y, LOOP_WIDTH, RADII, ROTATION,
    ROTATION_LOOP_MS, WIDTH,
};
use crate::{NativeElementInput, PropertyId, PropertySchema, SchemaError, SchemaValue, ValueType};

/// Declares one rotation driver and coordinated alternating native property loops.
#[must_use]
pub(super) fn properties() -> [PropertySchema; 12] {
    [
        control(
            ROTATION_LOOP_MS,
            "rotationLoopMs",
            ValueType::Float,
            "Duration of one clockwise rotation in milliseconds.",
        ),
        control(
            LOOP_MS,
            "loopMs",
            ValueType::Float,
            "Duration of one alternating native motion leg in milliseconds.",
        ),
        control(
            LOOP_PLAYING,
            "loopPlaying",
            ValueType::Bool,
            "Whether native property loops advance or preserve their current phase.",
        ),
        control(
            LOOP_TRANSLATE_X,
            "loopTranslateX",
            ValueType::Float,
            "Horizontal translation at the far end of the loop in logical pixels.",
        ),
        control(
            LOOP_TRANSLATE_Y,
            "loopTranslateY",
            ValueType::Float,
            "Vertical translation at the far end of the loop in logical pixels.",
        ),
        control(
            LOOP_SCALE,
            "loopScale",
            ValueType::Float,
            "Uniform transform scale at the far end of the loop.",
        ),
        control(
            LOOP_OPACITY,
            "loopOpacity",
            ValueType::Float,
            "Group opacity at the far end of the loop.",
        ),
        control(
            LOOP_BACKGROUND,
            "loopBackground",
            ValueType::Color,
            "Solid background color at the far end of the loop.",
        ),
        control(
            LOOP_HOLD,
            "loopHold",
            ValueType::Bool,
            "Hold the transform target before returning to its start on each native loop.",
        ),
        control(
            LOOP_WIDTH,
            "loopWidth",
            ValueType::Float,
            "Width at the far end of a native layout loop in logical pixels.",
        ),
        control(
            LOOP_RADIUS,
            "loopRadius",
            ValueType::Float,
            "Corner radius at the far end of a native paint loop in logical pixels.",
        ),
        control(
            LOOP_GAP,
            "loopGap",
            ValueType::Float,
            "Child spacing at the far end of a native layout loop in logical pixels.",
        ),
    ]
}

/// Creates one declarative control property with `id`, `name`, typed `value_type` and `docs`.
fn control(
    id: PropertyId,
    name: &'static str,
    value_type: ValueType,
    docs: &str,
) -> PropertySchema {
    PropertySchema::new(id, name, value_type, docs).not_animatable()
}

/// Binds native transform, opacity, color, width, radius, and gap loops from `input` to `element`.
///
/// Returns the resulting element, with inactive playback when `loop_playing` is false.
///
/// # Errors
///
/// Returns a schema error for invalid timing, geometry, opacity, or incompatible loop inputs.
pub(super) fn apply(
    mut element: Element,
    input: &NativeElementInput,
) -> Result<Element, SchemaError> {
    let rotation_ms = duration(input, ROTATION_LOOP_MS, "rotationLoopMs")?;
    let loop_ms = duration(input, LOOP_MS, "loopMs")?;
    let playing = !matches!(input.get(LOOP_PLAYING), Some(SchemaValue::Bool(false)));
    let hold = matches!(input.get(LOOP_HOLD), Some(SchemaValue::Bool(true)));
    let x = scalar(input, LOOP_TRANSLATE_X, "loopTranslateX")?;
    let y = scalar(input, LOOP_TRANSLATE_Y, "loopTranslateY")?;
    let scale = scalar(input, LOOP_SCALE, "loopScale")?;
    let opacity = scalar(input, LOOP_OPACITY, "loopOpacity")?;
    let width = scalar(input, LOOP_WIDTH, "loopWidth")?;
    let radius = scalar(input, LOOP_RADIUS, "loopRadius")?;
    let gap = scalar(input, LOOP_GAP, "loopGap")?;
    let background = match input.get(LOOP_BACKGROUND) {
        Some(SchemaValue::Color(color)) => Some(*color),
        _ => None,
    };
    if loop_ms.is_none()
        && (x.is_some()
            || y.is_some()
            || scale.is_some()
            || opacity.is_some()
            || background.is_some()
            || width.is_some()
            || radius.is_some()
            || gap.is_some())
    {
        return Err(SchemaError::Adapter("loop target requires loop_ms".into()));
    }
    if rotation_ms.is_some() && (x.is_some() || y.is_some() || scale.is_some()) {
        return Err(SchemaError::Adapter(
            "rotation_loop_ms cannot combine with another transform loop".into(),
        ));
    }
    if hold && (loop_ms.is_none() || (x.is_none() && y.is_none() && scale.is_none())) {
        return Err(SchemaError::Adapter(
            "loop_hold requires loop_ms and a transform target".into(),
        ));
    }
    if scale.is_some_and(|value| value <= 0.0) {
        return Err(SchemaError::Adapter("loop_scale must be positive".into()));
    }
    if opacity.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
        return Err(SchemaError::Adapter(
            "loop_opacity must be between 0 and 1".into(),
        ));
    }
    if [width, radius, gap]
        .into_iter()
        .flatten()
        .any(|value| value < 0.0)
    {
        return Err(SchemaError::Adapter(
            "loop layout and radius targets must be nonnegative".into(),
        ));
    }
    let base_rotation = match input.get(ROTATION) {
        Some(SchemaValue::Float(degrees)) => degrees.to_radians(),
        _ => 0.0,
    };
    let base = Transform2D::IDENTITY.rotate(base_rotation);
    if let Some(milliseconds) = rotation_ms {
        let motion = motion(
            base,
            base.rotate(base_rotation + std::f32::consts::TAU),
            milliseconds,
            Direction::Normal,
            playing,
            false,
        )?;
        element = element.bind(property::Transform, motion);
    } else if let Some(milliseconds) = loop_ms
        && (x.is_some() || y.is_some() || scale.is_some())
    {
        let target = base
            .translate(x.unwrap_or(0.0), y.unwrap_or(0.0))
            .scale(scale.unwrap_or(1.0), scale.unwrap_or(1.0));
        let motion = motion(
            base,
            target,
            milliseconds,
            Direction::Alternate,
            playing,
            hold,
        )?;
        element = element.bind(property::Transform, motion);
    }
    if let (Some(milliseconds), Some(target)) = (loop_ms, opacity) {
        element = element.bind(
            property::LayerOpacity,
            motion(
                1.0,
                target,
                milliseconds,
                Direction::Alternate,
                playing,
                false,
            )?,
        );
    }
    if let (Some(milliseconds), Some(target)) = (loop_ms, background) {
        let from = match input.get(BACKGROUND) {
            Some(SchemaValue::Brush(Fill::Solid(color))) => *color,
            _ => {
                return Err(SchemaError::Adapter(
                    "loop_background requires a solid background".into(),
                ));
            }
        };
        element = element.bind(
            property::BackgroundColor,
            motion(
                from,
                target,
                milliseconds,
                Direction::Alternate,
                playing,
                false,
            )?,
        );
    }
    if let (Some(milliseconds), Some(target)) = (loop_ms, width) {
        let from = match input.get(WIDTH) {
            Some(SchemaValue::Dimension(value)) => match value.expand() {
                ExpandedDimension::Length(pixels) => pixels,
                _ => {
                    return Err(SchemaError::Adapter(
                        "loop_width requires a pixel width".into(),
                    ));
                }
            },
            _ => {
                return Err(SchemaError::Adapter(
                    "loop_width requires a pixel width".into(),
                ));
            }
        };
        element = element.bind(
            property::WidthPx,
            motion(
                from,
                target,
                milliseconds,
                Direction::Alternate,
                playing,
                false,
            )?,
        );
    }
    if let (Some(milliseconds), Some(target)) = (loop_ms, radius) {
        let from = match input.get(RADII) {
            Some(SchemaValue::Radii(value)) => value.as_array(),
            _ => return Err(SchemaError::Adapter("loopRadius requires radii".into())),
        };
        element = element.bind(
            property::CornerRadii,
            motion(
                from,
                [target; 4],
                milliseconds,
                Direction::Alternate,
                playing,
                false,
            )?,
        );
    }
    if let (Some(milliseconds), Some(target)) = (loop_ms, gap) {
        let from = match input.get(GAP) {
            Some(SchemaValue::Float(value)) => *value,
            _ => return Err(SchemaError::Adapter("loop_gap requires gap".into())),
        };
        element = element.bind(
            property::Gap,
            motion(
                from,
                target,
                milliseconds,
                Direction::Alternate,
                playing,
                false,
            )?,
        );
    }
    Ok(element)
}

/// Reads an optional positive `id` duration in milliseconds from `input`.
///
/// # Errors
///
/// Returns a schema error for a duration outside 1–60,000 ms or a nonfinite value.
fn duration(
    input: &NativeElementInput,
    id: PropertyId,
    name: &str,
) -> Result<Option<u64>, SchemaError> {
    let Some(SchemaValue::Float(value)) = input.get(id) else {
        return Ok(None);
    };
    if !value.is_finite() || !(1.0..=60_000.0).contains(value) {
        return Err(SchemaError::Adapter(format!(
            "{name} must be between 1 and 60000"
        )));
    }
    Ok(Some(value.round() as u64))
}

/// Reads an optional finite scalar `id` from `input` for the named loop target.
///
/// # Errors
///
/// Returns a schema error for a nonfinite value.
fn scalar(
    input: &NativeElementInput,
    id: PropertyId,
    name: &str,
) -> Result<Option<f32>, SchemaError> {
    let Some(SchemaValue::Float(value)) = input.get(id) else {
        return Ok(None);
    };
    if !value.is_finite() {
        return Err(SchemaError::Adapter(format!("{name} must be finite")));
    }
    Ok(Some(*value))
}

/// Creates a typed infinite timeline from `from` to `to` over `milliseconds`.
/// `direction` chooses continuous rotation or alternate motion; alternate and
/// held motion ease to rest at each turnaround, while rotation stays linear.
/// `playing` controls the initial pause state. `hold` adds a plateau and a
/// return keyframe to the transform sequence. Returns a shared native motion
/// or a timing error.
///
/// # Errors
///
/// Returns a schema error if the typed keyframes or timing are invalid.
fn motion<T: Clone + Interpolate>(
    from: T,
    to: T,
    milliseconds: u64,
    direction: Direction,
    playing: bool,
    hold: bool,
) -> Result<Motion<T>, SchemaError> {
    let easing = if hold || direction == Direction::Alternate {
        Easing::CubicBezier(
            CubicBezier::new(0.42, 0.0, 0.58, 1.0).expect("constant CSS Bézier is valid"),
        )
    } else {
        Easing::Linear
    };
    let frames = if hold {
        Keyframes::new([
            Keyframe::new(0.0, from.clone()).easing(easing.clone()),
            Keyframe::new(0.35, to.clone()),
            Keyframe::new(0.65, to).easing(easing),
            Keyframe::new(1.0, from.clone()),
        ])
    } else {
        Keyframes::new([
            Keyframe::new(0.0, from.clone()).easing(easing),
            Keyframe::new(1.0, to),
        ])
    }
    .map_err(|error| SchemaError::Adapter(error.to_string()))?;
    let timing = Timing::new(Duration::from_millis(milliseconds))
        .iterations(Iterations::Infinite)
        .direction(if hold { Direction::Normal } else { direction });
    let motion = Motion::new(from);
    motion.play(
        Timeline::new(frames, timing).map_err(|error| SchemaError::Adapter(error.to_string()))?,
    );
    if !playing {
        motion.pause();
    }
    Ok(motion)
}
