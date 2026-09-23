//! Typed, retained property motion for native elements.

use std::collections::{HashMap, HashSet};

use argui_animation::{CubicBezier, Duration, Easing, Iterations, SpringConfig, Time};
use argui_core::Color;
use argui_ui::{Dimension, RetainedIdentity};

mod engine;

use engine::{
    AnimatedDimension, Entry, new_record, sample_record, validate_dimension, validate_number,
};

/// Stable identity of one property animation on one retained component site.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PropertyMotionKey {
    pub element: RetainedIdentity,
    pub property: u64,
    pub slot: u64,
}

/// Which state edge starts a property's declarative transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateTransitionPolicy {
    Enter,
    Leave,
    InOut,
}

/// State activity and canonical value used to detect a transition edge.
#[derive(Clone, Debug, PartialEq)]
pub struct StateTransition<T> {
    pub policy: StateTransitionPolicy,
    pub active: bool,
    pub base: T,
}

impl PropertyMotionKey {
    /// Builds a key from element, property, and animation-slot IDs.
    ///
    /// * `element` — retained instance/site/repeater identity.
    /// * `property` — stable native or component property ID.
    /// * `slot` — stable animation declaration ID.
    #[must_use]
    pub const fn new(element: RetainedIdentity, property: u64, slot: u64) -> Self {
        Self {
            element,
            property,
            slot,
        }
    }
}

/// Typed from/to values and timing for a keyframe or implicit transition.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAnimation<T> {
    pub from: Option<T>,
    pub to: Option<T>,
    /// Whether playback advances; false retains its current presented value.
    pub playing: bool,
    pub duration: Duration,
    pub iterations: Iterations,
    pub spring: Option<SpringConfig>,
    pub easing: Easing,
    pub keyframes: Vec<(f32, T)>,
    pub state_transition: Option<StateTransition<T>>,
}

impl<T> PropertyAnimation<T> {
    /// Validates shared timing and repetition policy for an animable property.
    ///
    /// * `from` — optional explicit starting value; absent for implicit transitions.
    /// * `to` — optional explicit target; absent for implicit transitions.
    /// * `duration_ms` — positive duration per iteration, in milliseconds.
    /// * `iterations` — `"once"` or `"infinite"`.
    ///
    /// # Errors
    ///
    /// Returns a diagnostic for invalid timing or an infinite animation without endpoints.
    pub fn new(
        from: Option<T>,
        to: Option<T>,
        duration_ms: f64,
        iterations: &str,
    ) -> Result<Self, String> {
        if !duration_ms.is_finite()
            || duration_ms <= 0.0
            || duration_ms > u64::MAX as f64 / 1_000_000.0
        {
            return Err(
                "animation duration must be a positive finite number of milliseconds".into(),
            );
        }
        let iterations = match iterations {
            "once" => Iterations::Finite(1.0),
            "infinite" if from.is_some() && to.is_some() => Iterations::Infinite,
            "infinite" => return Err("infinite animation requires both `from` and `to`".into()),
            _ => {
                return Err(format!(
                    "unknown animation iterations `{iterations}`; expected `once` or `infinite`"
                ));
            }
        };
        Ok(Self {
            from,
            to,
            playing: true,
            duration: Duration::from_nanos((duration_ms * 1_000_000.0).round() as u64),
            iterations,
            spring: None,
            easing: Easing::Linear,
            keyframes: Vec::new(),
            state_transition: None,
        })
    }

    /// Creates a spring driver using Argui's physical motion engine.
    ///
    /// * `from` — optional initial value for a newly mounted slot.
    /// * `to` — optional explicit target, otherwise the declarative property target.
    /// * `stiffness` — positive spring stiffness.
    /// * `damping` — nonnegative damping coefficient.
    ///
    /// # Errors
    ///
    /// Returns for invalid spring coefficients.
    pub fn spring(
        from: Option<T>,
        to: Option<T>,
        stiffness: f64,
        damping: f64,
    ) -> Result<Self, String> {
        let config = SpringConfig {
            stiffness,
            damping,
            ..SpringConfig::default()
        }
        .validate()
        .map_err(|error| error.to_string())?;
        Ok(Self {
            from,
            to,
            playing: true,
            duration: Duration::ZERO,
            iterations: Iterations::Finite(1.0),
            spring: Some(config),
            easing: Easing::Linear,
            keyframes: Vec::new(),
            state_transition: None,
        })
    }

    /// Creates a multi-stop timeline, including indefinitely repeated keyframes.
    ///
    /// * `frames` — normalized stops spanning zero through one.
    /// * `duration_ms` — positive duration per iteration in milliseconds.
    /// * `iterations` — `"once"` or `"infinite"`.
    ///
    /// # Errors
    ///
    /// Returns for invalid timing, repetition, or keyframe offsets.
    pub fn keyframes(
        frames: Vec<(f32, T)>,
        duration_ms: f64,
        iterations: &str,
    ) -> Result<Self, String> {
        let mut animation = Self::new(None, None, duration_ms, "once")?.with_keyframes(frames)?;
        animation.iterations = match iterations {
            "once" => Iterations::Finite(1.0),
            "infinite" => Iterations::Infinite,
            _ => {
                return Err(format!(
                    "unknown animation iterations `{iterations}`; expected `once` or `infinite`"
                ));
            }
        };
        Ok(animation)
    }

    /// Selects playback without restarting or discarding the retained timeline.
    ///
    /// `playing` resumes when true and freezes presentation when false. Returns
    /// this specification; paused motions request no frames and exclude paused time.
    #[must_use]
    pub fn with_playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }

    /// Applies a named CSS-compatible easing curve to a timeline.
    ///
    /// * `name` — one of `linear`, `ease`, `ease-in`, `ease-out`, or `ease-in-out`.
    ///
    /// # Errors
    ///
    /// Returns for unknown names or spring drivers, which do not use easing curves.
    pub fn with_easing_name(mut self, name: &str) -> Result<Self, String> {
        if self.spring.is_some() {
            return Err("spring animations do not accept `easing`".into());
        }
        self.easing = match name {
            "linear" => Easing::Linear,
            "ease" => bezier(0.25, 0.1, 0.25, 1.0),
            "ease-in" => bezier(0.42, 0.0, 1.0, 1.0),
            "ease-out" => bezier(0.0, 0.0, 0.58, 1.0),
            "ease-in-out" => bezier(0.42, 0.0, 0.58, 1.0),
            _ => return Err(format!("unknown animation easing `{name}`")),
        };
        Ok(self)
    }

    /// Installs validated multi-stop keyframes on a timeline.
    ///
    /// * `frames` — ordered normalized offsets and typed property values.
    ///
    /// # Errors
    ///
    /// Returns for invalid offsets, too few frames, a spring driver, or mixed from/to syntax.
    pub fn with_keyframes(mut self, frames: Vec<(f32, T)>) -> Result<Self, String> {
        if self.spring.is_some() {
            return Err("spring animations do not accept keyframes".into());
        }
        if self.from.is_some() || self.to.is_some() {
            return Err("keyframes cannot be mixed with `from` or `to`".into());
        }
        if frames.len() < 2
            || frames.first().is_none_or(|frame| frame.0 != 0.0)
            || frames.last().is_none_or(|frame| frame.0 != 1.0)
            || frames
                .iter()
                .any(|frame| !frame.0.is_finite() || !(0.0..=1.0).contains(&frame.0))
            || frames.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err("keyframes must be strictly increasing from 0% through 100%".into());
        }
        self.keyframes = frames;
        Ok(self)
    }

    /// Binds this motion to rising/falling state activity rather than value comparison.
    ///
    /// * `policy` — state edge or edges allowed to animate.
    /// * `active` — whether the named state currently matches.
    /// * `base` — canonical value outside the state.
    ///
    /// # Errors
    ///
    /// Returns when explicit endpoints, keyframes, or repetition conflict with a state transition.
    pub fn with_state_transition(
        mut self,
        policy: StateTransitionPolicy,
        active: bool,
        base: T,
    ) -> Result<Self, String> {
        if self.from.is_some()
            || self.to.is_some()
            || !self.keyframes.is_empty()
            || self.iterations != Iterations::Finite(1.0)
        {
            return Err("state transitions cannot use endpoints, keyframes, or repetition".into());
        }
        self.state_transition = Some(StateTransition {
            policy,
            active,
            base,
        });
        Ok(self)
    }
}

/// Builds one known-valid CSS cubic Bézier curve.
///
/// * `x1`, `y1`, `x2`, `y2` — standard CSS control-point coordinates.
fn bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Easing {
    Easing::CubicBezier(CubicBezier::new(x1, y1, x2, y2).expect("constant CSS Bézier is valid"))
}

/// Per-root typed property motions; idle trees request no animation frames.
#[derive(Default)]
pub struct PropertyMotionStore {
    entries: HashMap<PropertyMotionKey, Entry>,
    seen: HashSet<PropertyMotionKey>,
    last_error: Option<String>,
}

impl PropertyMotionStore {
    /// Creates an empty typed motion store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a render pass and marks prior property slots unseen.
    pub fn begin_render(&mut self) {
        self.seen.clear();
        self.last_error = None;
    }

    /// Records an invalid dynamic animation value without crashing the UI.
    ///
    /// * `message` — diagnostic for the latest render pass.
    pub fn report_error(&mut self, message: impl Into<String>) {
        self.last_error = Some(message.into());
    }

    /// Returns the latest dynamic animation diagnostic, if any.
    #[must_use]
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Drops animations whose property bindings disappeared from the tree.
    pub fn end_render(&mut self) {
        self.entries.retain(|key, _| self.seen.contains(key));
    }

    /// Returns whether any retained motion needs a presentation frame.
    #[must_use]
    pub fn needs_frame(&self) -> bool {
        self.entries.values().any(Entry::is_active)
    }

    /// Advances active motions and reports whether a sampled value changed.
    ///
    /// * `now` — monotonic timestamp from the host animation frame.
    ///
    /// # Panics
    ///
    /// Cannot panic for validated animation specifications.
    pub fn advance(&self, now: Time) -> bool {
        let mut changed = false;
        for entry in self.entries.values().filter(|entry| entry.is_active()) {
            changed |= entry.advance(now);
        }
        changed
    }

    /// Samples a numeric property for any native or user-defined component.
    ///
    /// * `key` — stable instance/site/property/slot identity.
    /// * `target` — current declarative property value.
    /// * `specification` — validated transition or explicit keyframes.
    /// * `reduced_motion` — accessibility preference disabling motion.
    ///
    /// # Errors
    ///
    /// Returns for non-finite values.
    pub fn sample_number(
        &mut self,
        key: PropertyMotionKey,
        target: f64,
        specification: PropertyAnimation<f64>,
        reduced_motion: bool,
    ) -> Result<f64, String> {
        validate_number(target, &specification)?;
        self.seen.insert(key.clone());
        let entry = self.entries.entry(key).or_insert_with(|| {
            Entry::Number(new_record(target, specification.clone(), reduced_motion))
        });
        if !matches!(entry, Entry::Number(_)) {
            *entry = Entry::Number(new_record(target, specification.clone(), reduced_motion));
        }
        let Entry::Number(record) = entry else {
            unreachable!("entry was replaced with numeric motion")
        };
        Ok(sample_record(record, target, specification, reduced_motion))
    }

    /// Samples a color property for any native or user-defined component.
    ///
    /// * `key` — stable instance/site/property/slot identity.
    /// * `target` — current declarative color.
    /// * `specification` — validated transition or explicit keyframes.
    /// * `reduced_motion` — accessibility preference disabling motion.
    ///
    /// # Errors
    ///
    /// This operation cannot fail for a valid color specification.
    pub fn sample_color(
        &mut self,
        key: PropertyMotionKey,
        target: Color,
        specification: PropertyAnimation<Color>,
        reduced_motion: bool,
    ) -> Result<Color, String> {
        self.seen.insert(key.clone());
        let entry = self.entries.entry(key).or_insert_with(|| {
            Entry::Color(new_record(target, specification.clone(), reduced_motion))
        });
        if !matches!(entry, Entry::Color(_)) {
            *entry = Entry::Color(new_record(target, specification.clone(), reduced_motion));
        }
        let Entry::Color(record) = entry else {
            unreachable!("entry was replaced with color motion")
        };
        Ok(sample_record(record, target, specification, reduced_motion))
    }

    /// Samples a length or percentage dimension without mixing incompatible units.
    ///
    /// * `key` — stable instance/site/property/slot identity.
    /// * `target` — current declarative length or percentage.
    /// * `specification` — validated transition or explicit keyframes.
    /// * `reduced_motion` — accessibility preference disabling motion.
    ///
    /// # Errors
    ///
    /// Returns for auto/intrinsic/mixed dimensions.
    pub fn sample_dimension(
        &mut self,
        key: PropertyMotionKey,
        target: Dimension,
        specification: PropertyAnimation<Dimension>,
        reduced_motion: bool,
    ) -> Result<Dimension, String> {
        validate_dimension(target, &specification)?;
        self.seen.insert(key.clone());
        let specification = PropertyAnimation {
            from: specification.from.map(AnimatedDimension),
            to: specification.to.map(AnimatedDimension),
            playing: specification.playing,
            duration: specification.duration,
            iterations: specification.iterations,
            spring: specification.spring,
            easing: specification.easing,
            keyframes: specification
                .keyframes
                .into_iter()
                .map(|(offset, value)| (offset, AnimatedDimension(value)))
                .collect(),
            state_transition: specification
                .state_transition
                .map(|transition| StateTransition {
                    policy: transition.policy,
                    active: transition.active,
                    base: AnimatedDimension(transition.base),
                }),
        };
        let target = AnimatedDimension(target);
        let entry = self.entries.entry(key).or_insert_with(|| {
            Entry::Dimension(new_record(target, specification.clone(), reduced_motion))
        });
        if !matches!(entry, Entry::Dimension(_)) {
            *entry = Entry::Dimension(new_record(target, specification.clone(), reduced_motion));
        }
        let Entry::Dimension(record) = entry else {
            unreachable!("entry was replaced with dimension motion")
        };
        Ok(sample_record(record, target, specification, reduced_motion).0)
    }

    /// Returns the number of retained property animation slots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether this root retains no property-motion slots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
