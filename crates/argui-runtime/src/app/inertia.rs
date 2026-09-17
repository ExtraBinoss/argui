use std::time::Duration;

use argui_core::{Point, ScrollDelta};
use argui_ui::{InertialScroll, NodeId, ScrollPhysics};
use winit::event::TouchPhase;

/// Tracks filtered pointer velocity and advances platform-independent scroll momentum.
#[derive(Debug)]
pub(super) struct ScrollInertia {
    velocity: Point,
    last_input: Option<Duration>,
    last_frame: Option<Duration>,
    released: bool,
    target: Option<NodeId>,
    config: InertialScroll,
    point: Option<Point>,
    dispatch_wheel: bool,
}

impl Default for ScrollInertia {
    fn default() -> Self {
        Self {
            velocity: Point::default(),
            last_input: None,
            last_frame: None,
            released: false,
            target: None,
            config: InertialScroll::default(),
            point: None,
            dispatch_wheel: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ScrollSample {
    /// Pixel or line displacement reported by the current input event.
    pub(super) delta: ScrollDelta,
    /// Input phase used to begin, continue, release, or cancel momentum.
    pub(super) phase: TouchPhase,
    /// Monotonic time since the owning application started.
    pub(super) now: Duration,
    /// Scroll target under the input point, if any.
    pub(super) target: Option<NodeId>,
    /// Scroll physics selected for the target region.
    pub(super) physics: ScrollPhysics,
    /// Current viewport-space pointer position.
    pub(super) point: Point,
    /// Whether synthesized wheel events should accompany momentum.
    pub(super) dispatch_wheel: bool,
    /// Whether the system requests that animated motion be minimized.
    pub(super) reduced_motion: bool,
}

/// Maps finger movement to Argui's scroll delta convention.
///
/// * `finger_delta` — viewport-space pointer displacement since the previous touch event.
/// * `natural` — whether rendered content should follow rather than oppose the finger.
///
/// Returns the displacement in Argui's scroll convention for the requested direction.
#[must_use]
pub(super) const fn touch_scroll_delta(finger_delta: Point, natural: bool) -> Point {
    if natural {
        finger_delta
    } else {
        Point::new(-finger_delta.x, -finger_delta.y)
    }
}

impl ScrollInertia {
    /// Records a scroll input and updates the filtered release velocity.
    pub(super) fn observe(&mut self, sample: ScrollSample) {
        let ScrollSample {
            delta,
            phase,
            now,
            target,
            physics,
            point,
            dispatch_wheel,
            reduced_motion,
        } = sample;
        if reduced_motion {
            self.cancel();
            return;
        }
        let config = match physics {
            ScrollPhysics::Hybrid => InertialScroll::default(),
            ScrollPhysics::Inertial(config) => config,
            ScrollPhysics::Direct | ScrollPhysics::Native => {
                self.cancel();
                return;
            }
        };
        let config = valid_inertia(config);
        let ScrollDelta::Pixels(delta) = delta else {
            self.cancel();
            return;
        };
        if phase == TouchPhase::Cancelled {
            self.cancel();
            return;
        }
        let target_changed = self.target != target;
        self.target = target;
        self.config = config;
        self.point = Some(point);
        self.dispatch_wheel = dispatch_wheel;
        if phase == TouchPhase::Ended {
            self.released = true;
            self.last_input = Some(now);
            self.last_frame = Some(now);
            return;
        }
        if phase == TouchPhase::Started || target_changed {
            self.velocity = Point::default();
        }
        let elapsed = self
            .last_input
            .map_or(1.0 / 60.0, |last| now.saturating_sub(last).as_secs_f32())
            .clamp(1.0 / 240.0, 1.0 / 20.0);
        let sample = Point::new(delta.x / elapsed, delta.y / elapsed);
        let sample_weight = config.sample_weight.clamp(0.0, 1.0);
        let retained = 1.0 - sample_weight;
        self.velocity.x = (self.velocity.x * retained + sample.x * sample_weight)
            .clamp(-config.velocity_limit, config.velocity_limit);
        self.velocity.y = (self.velocity.y * retained + sample.y * sample_weight)
            .clamp(-config.velocity_limit, config.velocity_limit);
        self.last_input = Some(now);
        self.last_frame = Some(now);
        self.released = false;
    }

    /// Advances exponential momentum and returns the viewport scroll delta for this frame.
    ///
    /// * `now` — monotonic time since the owning application started.
    ///
    /// Returns the decayed pixel delta, prior pointer position, and wheel-dispatch policy while
    /// momentum is active; returns `None` when no frame should be applied yet or motion has ended.
    pub(super) fn advance(&mut self, now: Duration) -> Option<(Point, Point, bool)> {
        let last_input = self.last_input?;
        if !self.released && now.saturating_sub(last_input) < self.config.continuation_grace {
            return None;
        }
        let elapsed = now
            .saturating_sub(self.last_frame.unwrap_or(last_input))
            .as_secs_f32()
            .clamp(1.0 / 240.0, 1.0 / 30.0);
        self.last_frame = Some(now);
        let decay = (-self.config.decay * elapsed).exp();
        let distance = if self.config.decay <= f32::EPSILON {
            elapsed
        } else {
            (1.0 - decay) / self.config.decay
        };
        let delta = Point::new(self.velocity.x * distance, self.velocity.y * distance);
        self.velocity.x *= decay;
        self.velocity.y *= decay;
        if self.velocity.x.hypot(self.velocity.y) < self.config.stop_velocity {
            self.cancel();
        }
        self.point.map(|point| (delta, point, self.dispatch_wheel))
    }

    /// Cancels active momentum and forgets its input target.
    pub(super) fn cancel(&mut self) {
        *self = Self::default();
    }

    /// Reports whether another animation frame is needed for this input.
    #[must_use]
    pub(super) fn needs_frame(&self) -> bool {
        self.last_input.is_some()
    }

    /// Returns the node receiving the current momentum scroll, if one was selected.
    #[must_use]
    pub(super) const fn target(&self) -> Option<NodeId> {
        self.target
    }
}

/// Replaces invalid inertia fields with defaults and clamps the sample weight.
fn valid_inertia(mut config: InertialScroll) -> InertialScroll {
    let defaults = InertialScroll::default();
    config.velocity_limit = non_negative(config.velocity_limit, defaults.velocity_limit);
    config.stop_velocity = non_negative(config.stop_velocity, defaults.stop_velocity);
    config.decay = non_negative(config.decay, defaults.decay);
    config.sample_weight = if config.sample_weight.is_finite() {
        config.sample_weight.clamp(0.0, 1.0)
    } else {
        defaults.sample_weight
    };
    config
}

/// Returns a finite non-negative value or the supplied fallback.
fn non_negative(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        fallback
    }
}
