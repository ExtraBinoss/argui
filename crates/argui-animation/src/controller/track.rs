//! Type-erased motion operations used by the tree animation registry.

use super::{Driver, Motion, MotionState, advance_spring, advance_timeline};
use crate::{FrameSchedule, Interpolate, MotionValue, Time};

/// Type-erased operations used by animation scheduling.
pub trait MotionTrack {
    /// Returns the stable identity of this track.
    fn identity(&self) -> usize;
    /// Returns whether this track is active.
    fn is_active(&self) -> bool;
    /// Returns the track's next presentation requirement after its last sample.
    fn frame_schedule(&self) -> FrameSchedule;
    /// Advances the track at `now`, returning whether its value changed.
    fn advance(&self, now: Time) -> bool;
    /// Moves the track to its target and marks it finished.
    fn finish(&self);
    /// Cancels the track without moving it to its target.
    fn cancel(&self);
    /// Freezes the track at its last sampled presentation value.
    fn pause(&self);
    /// Resumes a paused track without counting time spent paused.
    fn resume(&self);
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

    fn frame_schedule(&self) -> FrameSchedule {
        if !self.is_active() {
            return FrameSchedule::None;
        }
        let inner = self.lock();
        if inner.resume_pending {
            return FrameSchedule::Continuous;
        }
        match (&inner.driver, inner.last_frame) {
            (Driver::Timeline(timeline), Some(now)) => timeline.frame_schedule(now),
            _ => FrameSchedule::Continuous,
        }
    }

    fn advance(&self, now: Time) -> bool {
        if !self.is_active() {
            return false;
        }
        let mut inner = self.lock();
        let changed = if matches!(&inner.driver, Driver::Spring(_)) {
            advance_spring(&mut inner, now)
        } else {
            advance_timeline(&mut inner, now).unwrap_or_else(|_| {
                inner.driver = Driver::None;
                inner.state = MotionState::Canceled;
                inner.last_frame = None;
                false
            })
        };
        let active = inner.state == MotionState::Running;
        self.set_active(active);
        changed
    }

    fn finish(&self) {
        self.finish();
    }

    fn cancel(&self) {
        self.cancel();
    }

    fn pause(&self) {
        self.pause();
    }

    fn resume(&self) {
        self.resume();
    }
}
