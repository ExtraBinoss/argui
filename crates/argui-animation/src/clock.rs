use crate::{Duration, Time};
use std::cell::Cell;

/// Supplies monotonic time to animation sampling.
pub trait Clock {
    fn now(&self) -> Time;
}

/// A deterministic clock intended for tests, previews, and explicit playback.
#[derive(Debug, Default)]
pub struct ManualClock {
    now: Cell<Time>,
}

impl ManualClock {
    #[must_use]
    pub const fn new(now: Time) -> Self {
        Self {
            now: Cell::new(now),
        }
    }

    pub fn set(&self, now: Time) {
        self.now.set(now);
    }

    pub fn advance(&self, duration: Duration) {
        self.now.set(self.now.get() + duration);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Time {
        self.now.get()
    }
}
