use crate::{Duration, Time};

/// Stable identity for one scheduled animation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AnimationId(u64);

/// Timing shared by every animation sampled during one presentation frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Frame {
    /// Timestamp at the current presentation frame.
    pub now: Time,
    /// Time elapsed since the previous frame returned by this scheduler.
    pub elapsed: Duration,
}

/// Compact storage containing active animations only.
#[derive(Debug, Default)]
pub struct Scheduler {
    active: Vec<AnimationId>,
    next_id: u64,
    last_frame: Option<Time>,
}

impl Scheduler {
    /// Registers a new active animation and returns its identifier.
    #[must_use]
    pub fn start(&mut self) -> AnimationId {
        let id = AnimationId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        self.active.push(id);
        id
    }

    /// Stops an active animation, returning whether it was present.
    /// * `id` — identifier returned when the animation was started.
    pub fn stop(&mut self, id: AnimationId) -> bool {
        let Some(index) = self.active.iter().position(|active| *active == id) else {
            return false;
        };
        self.active.swap_remove(index);
        if self.active.is_empty() {
            self.last_frame = None;
        }
        true
    }

    /// Returns whether `id` is currently active.
    #[must_use]
    pub fn contains(&self, id: AnimationId) -> bool {
        self.active.contains(&id)
    }

    /// Returns the active animation identifiers.
    #[must_use]
    pub fn active(&self) -> &[AnimationId] {
        &self.active
    }

    /// Returns whether any animation is active and needs frames.
    #[must_use]
    pub fn needs_frame(&self) -> bool {
        !self.active.is_empty()
    }

    /// Produces a frame for `now` if at least one animation is active.
    ///
    /// The first frame after the scheduler becomes active has zero elapsed time.
    /// Produces a frame for `now` if at least one animation is active.
    ///
    /// The first frame after the scheduler becomes active has zero elapsed time.
    pub fn frame(&mut self, now: Time) -> Option<Frame> {
        self.needs_frame().then(|| {
            let elapsed = self
                .last_frame
                .map_or(Duration::ZERO, |last| now.duration_since(last));
            self.last_frame = Some(now);
            Frame { now, elapsed }
        })
    }
}
