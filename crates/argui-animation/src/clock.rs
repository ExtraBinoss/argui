use crate::{Duration, Time};
use std::cell::Cell;

/// Supplies monotonic time to animation sampling.
pub trait Clock {
    /// Returns the current monotonic timestamp.
    fn now(&self) -> Time;
}

/// A deterministic clock intended for tests, previews, and explicit playback.
#[derive(Debug, Default)]
pub struct ManualClock {
    now: Cell<Time>,
}

impl ManualClock {
    /// Creates a manual clock starting at `now`.
    #[must_use]
    pub const fn new(now: Time) -> Self {
        Self {
            now: Cell::new(now),
        }
    }

    /// Sets the clock to an absolute timestamp.
    /// * `now` — timestamp to store as the current clock value.
    pub fn set(&self, now: Time) {
        self.now.set(now);
    }

    /// Advances the clock by a non-negative duration.
    pub fn advance(&self, duration: Duration) {
        self.now.set(self.now.get() + duration);
    }
}

impl Clock for ManualClock {
    /// Returns the timestamp currently stored in this clock.
    fn now(&self) -> Time {
        self.now.get()
    }
}
