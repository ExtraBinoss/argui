//! Retains the phase of an equivalent timeline across rebuilt UI descriptions.

use super::{Driver, Motion, MotionState};

impl<T> Motion<T> {
    /// Reuses this motion's timeline position when `replacement` describes the same timeline.
    /// Applies the replacement's paused or running state and returns whether reuse succeeded.
    /// Finished timelines stay finished until their definition changes or their node remounts.
    /// A non-timeline driver or changed keyframes or timing returns false without changing this motion.
    pub fn retain_timeline(&self, replacement: &Self) -> bool
    where
        T: PartialEq,
    {
        if self == replacement {
            return true;
        }
        let (matches, desired_state, finished) = {
            let old = self.lock();
            let new = replacement.lock();
            let matches = matches!(
                old.state,
                MotionState::Running | MotionState::Paused | MotionState::Finished
            ) && match (&old.driver, &new.driver) {
                (Driver::Timeline(old), Driver::Timeline(new)) => old.same_definition(new),
                _ => false,
            };
            (matches, new.state, old.state == MotionState::Finished)
        };
        if !matches {
            return false;
        }
        if finished {
            return matches!(
                desired_state,
                MotionState::Running | MotionState::Paused | MotionState::Finished
            );
        }
        match desired_state {
            MotionState::Paused => self.pause(),
            MotionState::Running => self.resume(),
            _ => return false,
        }
        true
    }
}
