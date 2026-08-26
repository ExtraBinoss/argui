use argui_animation::{Clock, Scheduler, Time};
use std::time::Instant;

pub(super) struct RuntimeAnimations {
    clock: MonotonicClock,
    scheduler: Scheduler,
}

impl RuntimeAnimations {
    pub(super) fn new() -> Self {
        Self {
            clock: MonotonicClock::new(),
            scheduler: Scheduler::default(),
        }
    }

    /// Samples time only when active work exists and reports whether to redraw.
    pub(super) fn advance_if_active(&mut self) -> bool {
        if !self.scheduler.needs_frame() {
            return false;
        }
        let _ = self.scheduler.frame(self.clock.now());
        self.scheduler.needs_frame()
    }
}

struct MonotonicClock {
    origin: Instant,
}

impl MonotonicClock {
    fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Clock for MonotonicClock {
    fn now(&self) -> Time {
        Time::from_nanos(u64::try_from(self.origin.elapsed().as_nanos()).unwrap_or(u64::MAX))
    }
}
