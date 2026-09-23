//! Motion-driver values, retained records, interpolation, and validation.

use argui_animation::{
    Interpolate, Keyframe, Keyframes, Motion, MotionState, MotionValue, Time, Timeline, Timing,
    Tween,
};
use argui_core::Color;
use argui_ui::{Dimension, ExpandedDimension};

use super::{PropertyAnimation, StateTransitionPolicy};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AnimatedDimension(pub(super) Dimension);

impl Interpolate for AnimatedDimension {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        let dimension = match (self.0.expand(), target.0.expand()) {
            (ExpandedDimension::Length(from), ExpandedDimension::Length(to)) => {
                Dimension::length(from.interpolate(to, progress))
            }
            (ExpandedDimension::Percent(from), ExpandedDimension::Percent(to)) => {
                Dimension::percent(from.interpolate(to, progress))
            }
            _ if progress >= 1.0 => target.0,
            _ => self.0,
        };
        Self(dimension)
    }
}

impl MotionValue for AnimatedDimension {
    fn zero() -> Self {
        Self(Dimension::length(0.0))
    }

    fn add(self, other: Self) -> Self {
        dimension_arithmetic(self, other, 1.0)
    }

    fn subtract(self, other: Self) -> Self {
        dimension_arithmetic(self, other, -1.0)
    }

    fn scale(self, factor: f64) -> Self {
        match self.0.expand() {
            ExpandedDimension::Length(value) => Self(Dimension::length(value * factor as f32)),
            ExpandedDimension::Percent(value) => Self(Dimension::percent(value * factor as f32)),
            _ => self,
        }
    }

    fn magnitude(self) -> f64 {
        match self.0.expand() {
            ExpandedDimension::Length(value) | ExpandedDimension::Percent(value) => {
                f64::from(value.abs())
            }
            _ => 0.0,
        }
    }
}

/// Performs spring arithmetic on a validated same-unit dimension pair.
///
/// * `left` — current dimension.
/// * `right` — displacement or velocity dimension.
/// * `sign` — `1` for addition or `-1` for subtraction.
fn dimension_arithmetic(
    left: AnimatedDimension,
    right: AnimatedDimension,
    sign: f32,
) -> AnimatedDimension {
    match (left.0.expand(), right.0.expand()) {
        (ExpandedDimension::Length(a), ExpandedDimension::Length(b)) => {
            AnimatedDimension(Dimension::length(a + sign * b))
        }
        (ExpandedDimension::Percent(a), ExpandedDimension::Percent(b)) => {
            AnimatedDimension(Dimension::percent(a + sign * b))
        }
        (ExpandedDimension::Length(0.0), ExpandedDimension::Percent(b)) => {
            AnimatedDimension(Dimension::percent(sign * b))
        }
        (ExpandedDimension::Percent(a), ExpandedDimension::Length(0.0)) => {
            AnimatedDimension(Dimension::percent(a))
        }
        _ => left,
    }
}

pub(super) struct Record<T> {
    motion: Motion<T>,
    target: T,
    specification: PropertyAnimation<T>,
    previous_active: Option<bool>,
}

pub(super) enum Entry {
    Number(Record<f64>),
    Color(Record<Color>),
    Dimension(Record<AnimatedDimension>),
}

impl Entry {
    /// Advances this entry, returning whether presentation changed.
    ///
    /// * `now` — host animation timestamp.
    pub(super) fn advance(&self, now: Time) -> bool {
        match self {
            Self::Number(record) => advance_record(record, now),
            Self::Color(record) => advance_record(record, now),
            Self::Dimension(record) => advance_record(record, now),
        }
    }

    /// Returns whether this entry still requires animation frames.
    pub(super) fn is_active(&self) -> bool {
        match self {
            Self::Number(record) => record.motion.is_active(),
            Self::Color(record) => record.motion.is_active(),
            Self::Dimension(record) => record.motion.is_active(),
        }
    }
}

/// Advances the native driver selected for one retained property record.
///
/// * `record` — typed motion and its current specification.
/// * `now` — monotonic presentation timestamp.
///
/// # Panics
///
/// Cannot panic when the timeline was validated by [`PropertyAnimation::new`].
fn advance_record<T: MotionValue + Interpolate + PartialEq>(record: &Record<T>, now: Time) -> bool {
    if record.specification.spring.is_some() {
        record.motion.advance_spring(now)
    } else {
        record
            .motion
            .advance(now)
            .expect("validated motion timeline advances")
    }
}

/// Initializes one typed motion without starting an implicit transition.
///
/// * `target` — declarative initial target.
/// * `specification` — timing and optional explicit endpoints.
/// * `reduced_motion` — accessibility preference.
pub(super) fn new_record<T>(
    target: T,
    specification: PropertyAnimation<T>,
    reduced_motion: bool,
) -> Record<T>
where
    T: Clone + Interpolate + MotionValue + PartialEq,
{
    let initial = specification
        .state_transition
        .as_ref()
        .and_then(|state| {
            (state.active
                && matches!(
                    state.policy,
                    StateTransitionPolicy::Enter | StateTransitionPolicy::InOut
                ))
            .then_some(state.base)
        })
        .or_else(|| specification.keyframes.first().map(|frame| frame.1))
        .or(specification.from)
        .unwrap_or(target);
    let motion = Motion::new(if reduced_motion { target } else { initial });
    let previous_active = specification
        .state_transition
        .as_ref()
        .map(|state| state.active);
    let mut record = Record {
        motion,
        target,
        specification: specification.clone(),
        previous_active,
    };
    if !reduced_motion
        && specification
            .state_transition
            .as_ref()
            .is_some_and(|state| {
                state.active
                    && matches!(
                        state.policy,
                        StateTransitionPolicy::Enter | StateTransitionPolicy::InOut
                    )
            })
    {
        start_transition(&mut record, target);
    } else if !reduced_motion
        && (specification.from.is_some()
            || specification.to.is_some()
            || !specification.keyframes.is_empty())
    {
        start_explicit(&mut record, target);
    }
    record
}

/// Samples an existing motion and retargets only when inputs change.
///
/// * `record` — persistent typed motion.
/// * `target` — latest declarative value.
/// * `specification` — latest animation parameters.
/// * `reduced_motion` — accessibility preference.
pub(super) fn sample_record<T>(
    record: &mut Record<T>,
    target: T,
    specification: PropertyAnimation<T>,
    reduced_motion: bool,
) -> T
where
    T: Clone + Interpolate + MotionValue + PartialEq,
{
    let playing = specification.playing;
    // Playback changes affect scheduling, not the timeline's specification.
    record.specification.playing = playing;
    sample_target(record, target, specification, reduced_motion);
    if !playing {
        record.motion.pause();
    } else if record.motion.state() == MotionState::Paused && !reduced_motion {
        record.motion.resume();
    }
    record.motion.value()
}

/// Applies `target`, driver `specification` and `reduced_motion` to `record`.
/// Returns the current presentation without changing playback pause policy.
fn sample_target<T>(
    record: &mut Record<T>,
    target: T,
    specification: PropertyAnimation<T>,
    reduced_motion: bool,
) -> T
where
    T: Clone + Interpolate + MotionValue + PartialEq,
{
    if reduced_motion {
        if record.motion.is_active() || record.motion.value() != target {
            record.motion.set(target);
        }
        record.target = target;
        record.specification = specification;
        record.previous_active = record
            .specification
            .state_transition
            .as_ref()
            .map(|state| state.active);
        return target;
    }
    if let Some(state) = &specification.state_transition {
        let edge = record
            .previous_active
            .is_some_and(|previous| previous != state.active);
        let animate_edge = edge
            && match state.policy {
                StateTransitionPolicy::Enter => state.active,
                StateTransitionPolicy::Leave => !state.active,
                StateTransitionPolicy::InOut => true,
            };
        let target_changed = record.target != target;
        let was_active = record.motion.is_active() || record.motion.state() == MotionState::Paused;
        record.target = target;
        record.previous_active = Some(state.active);
        record.specification = specification;
        if animate_edge {
            start_transition(record, target);
        } else if edge {
            record.motion.set(target);
        } else if was_active && target_changed {
            start_transition(record, target);
        } else if !was_active && record.motion.value() != target {
            record.motion.set(target);
        }
        return record.motion.value();
    }
    record.previous_active = None;
    let changed = record.specification != specification;
    let target_changed = record.target != target;
    record.specification = specification.clone();
    record.target = target;
    if let Some(spring) = specification.spring {
        if changed
            || target_changed
            || matches!(
                record.motion.state(),
                MotionState::Idle | MotionState::Canceled
            )
        {
            record
                .motion
                .spring_to(specification.to.unwrap_or(target), spring)
                .expect("validated spring configuration");
        }
    } else if specification.from.is_some()
        || specification.to.is_some()
        || !specification.keyframes.is_empty()
    {
        if changed
            || matches!(
                record.motion.state(),
                MotionState::Idle | MotionState::Canceled
            )
        {
            start_explicit(record, target);
        }
    } else if target_changed {
        record.motion.animate_to(
            target,
            Tween::new(specification.duration).easing(specification.easing),
        );
    }
    record.motion.value()
}

/// Starts a state-edge tween or spring from the retained presented value.
///
/// * `record` — persistent typed motion at the state site.
/// * `target` — effective property value after state selection.
fn start_transition<T>(record: &mut Record<T>, target: T)
where
    T: Clone + Interpolate + MotionValue + PartialEq,
{
    if let Some(config) = record.specification.spring {
        record
            .motion
            .spring_to(target, config)
            .expect("validated spring configuration");
    } else {
        record.motion.animate_to(
            target,
            Tween::new(record.specification.duration).easing(record.specification.easing.clone()),
        );
    }
}

/// Starts or retargets explicit keyframes from the current presentation value.
///
/// * `record` — persistent typed motion.
/// * `target` — current declarative target when `to` is absent.
///
/// # Panics
///
/// Cannot panic after [`PropertyAnimation::new`] validated the duration.
fn start_explicit<T>(record: &mut Record<T>, target: T)
where
    T: Clone + Interpolate + MotionValue + PartialEq,
{
    if let Some(config) = record.specification.spring {
        record
            .motion
            .spring_to(record.specification.to.unwrap_or(target), config)
            .expect("validated spring configuration");
        return;
    }
    let from = record.motion.value();
    if !record.specification.keyframes.is_empty() {
        let frames = record
            .specification
            .keyframes
            .iter()
            .enumerate()
            .map(|(index, (offset, value))| {
                let value = if index == 0 { from } else { *value };
                Keyframe::new(*offset, value).easing(record.specification.easing.clone())
            })
            .collect::<Vec<_>>();
        let timeline = Timeline::new(
            Keyframes::new(frames).expect("validated keyframe offsets"),
            Timing::new(record.specification.duration).iterations(record.specification.iterations),
        )
        .expect("validated animation timing");
        record.motion.play(timeline);
        return;
    }
    let to = record.specification.to.unwrap_or(target);
    let timeline = Timeline::new(
        Keyframes::new([
            Keyframe::new(0.0, from).easing(record.specification.easing.clone()),
            Keyframe::new(1.0, to),
        ])
        .expect("two valid endpoints"),
        Timing::new(record.specification.duration).iterations(record.specification.iterations),
    )
    .expect("validated animation timing");
    record.motion.play(timeline);
}

/// Validates finite numeric endpoints before starting interpolation.
///
/// * `target` — declarative value.
/// * `specification` — optional endpoints.
///
/// # Errors
///
/// Returns for NaN or infinity.
pub(super) fn validate_number(
    target: f64,
    specification: &PropertyAnimation<f64>,
) -> Result<(), String> {
    if [
        Some(target),
        specification.from,
        specification.to,
        specification
            .state_transition
            .as_ref()
            .map(|state| state.base),
    ]
    .into_iter()
    .flatten()
    .chain(specification.keyframes.iter().map(|(_, value)| *value))
    .all(f64::is_finite)
    {
        Ok(())
    } else {
        Err("numeric animation values must be finite".into())
    }
}

/// Validates a dimensional animation and its unit compatibility.
///
/// * `target` — declarative length or percentage.
/// * `specification` — optional endpoints.
///
/// # Errors
///
/// Returns for auto/intrinsic dimensions, non-finite values, or mixed units.
pub(super) fn validate_dimension(
    target: Dimension,
    specification: &PropertyAnimation<Dimension>,
) -> Result<(), String> {
    let mut unit = None;
    for value in [
        Some(target),
        specification.from,
        specification.to,
        specification
            .state_transition
            .as_ref()
            .map(|state| state.base),
    ]
    .into_iter()
    .flatten()
    .chain(specification.keyframes.iter().map(|(_, value)| *value))
    {
        let (kind, scalar) = match value.expand() {
            ExpandedDimension::Length(value) => ("length", value),
            ExpandedDimension::Percent(value) => ("percent", value),
            _ => return Err("auto and intrinsic dimensions cannot be animated".into()),
        };
        if !scalar.is_finite() {
            return Err("animation dimensions must be finite".into());
        }
        if unit.is_some_and(|unit| unit != kind) {
            return Err("animation dimensions must use the same unit".into());
        }
        unit = Some(kind);
    }
    Ok(())
}
