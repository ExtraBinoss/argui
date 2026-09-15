use crate::{
    Composition, Duration, Easing, FillMode, Interpolate, Keyframe, Keyframes, MotionValue,
    PlaybackState, Spring, SpringConfig, Time, Timeline, Timing, TimingError,
};
use std::{
    fmt,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

#[derive(Clone, Debug, PartialEq)]
/// Parameters for a finite, eased transition between two values.
pub struct Tween {
    /// Active interpolation duration.
    pub duration: Duration,
    /// Delay before interpolation begins.
    pub delay: Duration,
    /// Easing curve applied during interpolation.
    pub easing: Easing,
}

impl Tween {
    /// Creates a tween of `duration` with no delay and linear easing.
    /// Sets the delay before interpolation begins.
    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self {
            duration,
            delay: Duration::ZERO,
            easing: Easing::Linear,
        }
    }

    /// Sets the delay before interpolation begins.
    #[must_use]
    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Sets the interpolation easing curve.
    #[must_use]
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Lifecycle state of a [`Motion`].
pub enum MotionState {
    /// No animation has been started or a value was set directly.
    #[default]
    Idle,
    /// The motion is advancing.
    Running,
    /// The motion is paused.
    Paused,
    /// The motion reached its target.
    Finished,
    /// The motion was canceled.
    Canceled,
}

/// Cloneable, shared control handle for an animated value.
#[derive(Clone)]
pub struct Motion<T>(Arc<Mutex<MotionInner<T>>>);

#[derive(Clone, Debug, PartialEq)]
/// Associates a motion with its composition rule and priority.
pub struct MotionBinding<T> {
    /// Motion supplying the animated value.
    pub motion: Motion<T>,
    /// How this value combines with other contributions.
    pub composition: Composition,
    /// Composition priority; lower values are applied first.
    pub priority: i32,
}

impl<T> MotionBinding<T> {
    /// Creates a binding using replacement composition and priority zero.
    /// * `motion` — motion whose value and playback this binding exposes.
    #[must_use]
    pub fn new(motion: Motion<T>) -> Self {
        Self {
            motion,
            composition: Composition::Replace,
            priority: 0,
        }
    }

    /// Sets the contribution composition rule.
    #[must_use]
    pub const fn composition(mut self, composition: Composition) -> Self {
        self.composition = composition;
        self
    }

    /// Sets the contribution priority.
    #[must_use]
    pub const fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl<T> From<Motion<T>> for MotionBinding<T> {
    fn from(motion: Motion<T>) -> Self {
        Self::new(motion)
    }
}

struct MotionInner<T> {
    value: T,
    target: T,
    driver: Driver<T>,
    state: MotionState,
    last_frame: Option<Time>,
    resume_pending: bool,
    completed_iterations: u64,
}

enum Driver<T> {
    None,
    PendingTween { from: T, tween: Tween },
    Timeline(Timeline<T>),
    Spring(Spring<T>),
}

impl<T: fmt::Debug> fmt::Debug for Motion<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.lock();
        formatter
            .debug_struct("Motion")
            .field("value", &inner.value)
            .field("target", &inner.target)
            .field("state", &inner.state)
            .finish_non_exhaustive()
    }
}

impl<T> PartialEq for Motion<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Motion<T> {
    fn lock(&self) -> MutexGuard<'_, MotionInner<T>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Creates an idle motion initialized to `value`.
    #[must_use]
    pub fn new(value: T) -> Self
    where
        T: Clone,
    {
        Self(Arc::new(Mutex::new(MotionInner {
            value: value.clone(),
            target: value,
            driver: Driver::None,
            state: MotionState::Idle,
            last_frame: None,
            resume_pending: false,
            completed_iterations: 0,
        })))
    }

    /// Returns the current animated value.
    #[must_use]
    pub fn value(&self) -> T
    where
        T: Clone,
    {
        self.lock().value.clone()
    }

    /// Returns the motion's current target.
    #[must_use]
    pub fn target(&self) -> T
    where
        T: Clone,
    {
        self.lock().target.clone()
    }

    /// Returns the current lifecycle state.
    #[must_use]
    pub fn state(&self) -> MotionState {
        self.lock().state
    }

    /// Returns whether the motion is currently running.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state() == MotionState::Running
    }

    /// Returns the number of completed timeline iterations.
    #[must_use]
    pub fn completed_iterations(&self) -> u64 {
        self.lock().completed_iterations
    }

    /// Returns a stable identity for this shared motion handle.
    #[must_use]
    pub fn identity(&self) -> usize {
        Arc::as_ptr(&self.0).cast::<()>() as usize
    }

    /// Replaces the current value and target, stopping any active animation.
    pub fn set(&self, value: T)
    where
        T: Clone,
    {
        let mut inner = self.lock();
        inner.value = value.clone();
        inner.target = value;
        inner.driver = Driver::None;
        inner.state = MotionState::Idle;
        inner.last_frame = None;
        inner.completed_iterations = 0;
    }

    /// Cancels the current animation without changing the current value.
    pub fn cancel(&self) {
        let mut inner = self.lock();
        inner.driver = Driver::None;
        inner.state = MotionState::Canceled;
        inner.last_frame = None;
    }

    /// Moves directly to the target and marks the motion finished.
    pub fn finish(&self)
    where
        T: Clone,
    {
        let mut inner = self.lock();
        inner.value = inner.target.clone();
        inner.driver = Driver::None;
        inner.state = MotionState::Finished;
        inner.last_frame = None;
    }

    /// Pauses a running motion.
    pub fn pause(&self) {
        let mut inner = self.lock();
        if inner.state != MotionState::Running {
            return;
        }
        let last_frame = inner.last_frame;
        if let (Driver::Timeline(timeline), Some(now)) = (&mut inner.driver, last_frame) {
            timeline.pause(now);
        }
        inner.state = MotionState::Paused;
    }

    /// Resumes a paused motion; time spent paused is excluded from playback.
    pub fn resume(&self) {
        let mut inner = self.lock();
        if inner.state == MotionState::Paused {
            inner.state = MotionState::Running;
            inner.resume_pending = true;
            inner.last_frame = None;
        }
    }

    /// Starts the supplied timeline on this motion.
    pub fn play(&self, timeline: Timeline<T>)
    where
        T: Clone + Interpolate,
    {
        let mut inner = self.lock();
        inner.target = timeline.terminal_value();
        inner.driver = Driver::Timeline(timeline);
        inner.state = MotionState::Running;
        inner.last_frame = None;
        inner.resume_pending = true;
        inner.completed_iterations = 0;
    }
}

impl<T: Clone + Interpolate + PartialEq> Motion<T> {
    /// Animates the current value toward `target` using `tween`.
    pub fn animate_to(&self, target: T, tween: Tween) {
        let mut inner = self.lock();
        if tween.duration == Duration::ZERO {
            inner.value = target.clone();
            inner.target = target;
            inner.driver = Driver::None;
            inner.state = MotionState::Finished;
            inner.last_frame = None;
            inner.completed_iterations = 0;
            return;
        }
        let from = inner.value.clone();
        inner.target = target;
        inner.driver = Driver::PendingTween { from, tween };
        inner.state = MotionState::Running;
        inner.last_frame = None;
        inner.resume_pending = false;
        inner.completed_iterations = 0;
    }

    /// Sets `from` immediately and begins a new tween toward `target`.
    pub fn restart(&self, from: T, target: T, tween: Tween) {
        self.set(from);
        self.animate_to(target, tween);
    }

    /// Samples the motion at `now`, returning whether its value changed.
    ///
    /// # Errors
    /// Returns a timing error if the tween cannot be represented as a valid timeline.
    pub fn advance(&self, now: Time) -> Result<bool, TimingError> {
        let mut inner = self.lock();
        if inner.state != MotionState::Running {
            return Ok(false);
        }
        if let Driver::PendingTween { from, tween } = &inner.driver {
            let frames = Keyframes::new(vec![
                Keyframe::new(0.0, from.clone()).easing(tween.easing.clone()),
                Keyframe::new(1.0, inner.target.clone()),
            ])?;
            let mut timeline = Timeline::new(
                frames,
                Timing::new(tween.duration)
                    .delay(tween.delay)
                    .fill(FillMode::Both),
            )?;
            timeline.play(now);
            inner.driver = Driver::Timeline(timeline);
        }
        if inner.resume_pending {
            if let Driver::Timeline(timeline) = &mut inner.driver {
                match timeline.state() {
                    PlaybackState::Idle | PlaybackState::Finished | PlaybackState::Canceled => {
                        timeline.play(now);
                    }
                    PlaybackState::Paused => timeline.resume(now),
                    PlaybackState::Running => {}
                }
            }
            inner.resume_pending = false;
        }
        let sample = match &mut inner.driver {
            Driver::Timeline(timeline) => Some(timeline.sample(now)),
            _ => None,
        };
        let mut changed = false;
        if let Some(sample) = sample {
            inner.completed_iterations = inner
                .completed_iterations
                .saturating_add(sample.events.iterations);
            if let Some(value) = sample.value {
                changed = value != inner.value;
                inner.value = value;
            }
            if sample.state == PlaybackState::Finished {
                inner.value = inner.target.clone();
                inner.driver = Driver::None;
                inner.state = MotionState::Finished;
                inner.last_frame = None;
                return Ok(true);
            }
        }
        inner.last_frame = Some(now);
        Ok(changed)
    }
}

impl<T: MotionValue> Motion<T> {
    /// Returns the current spring velocity, or the zero value for non-spring drivers.
    #[must_use]
    pub fn velocity(&self) -> T {
        match &self.lock().driver {
            Driver::Spring(spring) => spring.velocity(),
            _ => T::zero(),
        }
    }

    /// Springs the current value toward `target` using `config`.
    ///
    /// # Errors
    /// Returns a physics error if the spring configuration is invalid.
    /// * `config` — spring physics and settling thresholds.
    pub fn spring_to(&self, target: T, config: SpringConfig) -> Result<(), crate::PhysicsError> {
        let mut inner = self.lock();
        let velocity = match &inner.driver {
            Driver::Spring(spring) => spring.velocity(),
            _ => T::zero(),
        };
        let spring = Spring::new(inner.value, target, velocity, config)?;
        inner.target = target;
        inner.state = if spring.is_active() {
            MotionState::Running
        } else {
            MotionState::Finished
        };
        inner.driver = if spring.is_active() {
            Driver::Spring(spring)
        } else {
            Driver::None
        };
        inner.last_frame = None;
        inner.completed_iterations = 0;
        Ok(())
    }

    /// Sets `from` immediately and starts a spring toward `target`.
    ///
    /// # Errors
    /// Returns a physics error if the spring configuration is invalid.
    /// * `config` — spring physics and settling thresholds.
    pub fn restart_spring(
        &self,
        from: T,
        target: T,
        config: SpringConfig,
    ) -> Result<(), crate::PhysicsError> {
        self.set(from);
        self.spring_to(target, config)
    }

    /// Advances a spring-driven motion at `now`, returning whether its value changed.
    pub fn advance_spring(&self, now: Time) -> bool {
        let mut inner = self.lock();
        if inner.state != MotionState::Running {
            return false;
        }
        let elapsed = inner
            .last_frame
            .map_or(Duration::ZERO, |last| now.duration_since(last));
        let (changed, value, active) = match &mut inner.driver {
            Driver::Spring(spring) => {
                let changed = spring.advance(elapsed);
                (changed, spring.value(), spring.is_active())
            }
            _ => return false,
        };
        inner.value = value;
        inner.last_frame = Some(now);
        if !active {
            inner.driver = Driver::None;
            inner.state = MotionState::Finished;
            inner.last_frame = None;
        }
        changed
    }
}

/// Type-erased operations used by animation scheduling.
pub trait MotionTrack {
    /// Returns the stable identity of this track.
    fn identity(&self) -> usize;
    /// Returns whether this track is active.
    fn is_active(&self) -> bool;
    /// Advances the track at `now`, returning whether its value changed.
    fn advance(&self, now: Time) -> bool;
    /// Moves the track to its target and marks it finished.
    fn finish(&self);
    /// Cancels the track without moving it to its target.
    fn cancel(&self);
}

impl<T> MotionTrack for Motion<T>
where
    T: MotionValue + Clone + Interpolate + 'static,
{
    fn identity(&self) -> usize {
        self.identity()
    }

    fn is_active(&self) -> bool {
        self.is_active()
    }

    fn advance(&self, now: Time) -> bool {
        let is_spring = matches!(&self.lock().driver, Driver::Spring(_));
        if is_spring {
            self.advance_spring(now)
        } else {
            self.advance(now).unwrap_or_else(|_| {
                self.cancel();
                false
            })
        }
    }

    fn finish(&self) {
        self.finish();
    }

    fn cancel(&self) {
        self.cancel();
    }
}
