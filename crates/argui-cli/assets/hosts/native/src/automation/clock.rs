//! Application time excludes explicit automation pauses; test timeout does not.

use std::time::Duration;
use web_time::Instant;

/// Monotonic application clock and wall timeout shared by queued test actions.
#[derive(Clone, Copy)]
pub(super) struct TestClock {
    started: Instant,
    deadline: Instant,
    paused_at: Option<Instant>,
    paused_total: Duration,
}

impl TestClock {
    /// Creates a clock with `started` as application origin and `deadline` as timeout.
    pub(super) fn new(started: Instant, deadline: Instant) -> Self {
        Self {
            started,
            deadline,
            paused_at: None,
            paused_total: Duration::ZERO,
        }
    }

    /// Returns application time in milliseconds, excluding paused intervals.
    pub(super) fn elapsed_ms(self) -> f64 {
        let now = self.paused_at.unwrap_or_else(Instant::now);
        now.duration_since(self.started)
            .saturating_sub(self.paused_total)
            .as_secs_f64()
            * 1000.0
    }

    /// Reports whether the application scheduler is frozen.
    pub(super) fn paused(self) -> bool {
        self.paused_at.is_some()
    }

    /// Returns wall time left before the overall test timeout.
    pub(super) fn remaining(self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }

    /// Marks the start of a pause, preserving the first pause point.
    pub(super) fn pause(&mut self) {
        self.paused_at.get_or_insert_with(Instant::now);
    }

    /// Restarts application time after excluding the full paused interval.
    pub(super) fn resume(&mut self) {
        if let Some(paused_at) = self.paused_at.take() {
            self.paused_total += paused_at.elapsed();
        }
    }
}
