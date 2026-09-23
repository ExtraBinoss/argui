//! Retains the phase of an equivalent timeline across rebuilt UI descriptions.

use super::{Driver, Motion, MotionState};

impl<T> Motion<T> {
    /// Reuses this motion's timeline position when `replacement` describes the same timeline.
    /// Applies the replacement's paused or running state and returns whether reuse succeeded.
    /// A non-timeline driver or changed keyframes or timing returns false without changing this motion.
    pub fn retain_timeline(&self, replacement: &Self) -> bool
    where
        T: PartialEq,
    {
        if self == replacement {
            return true;
        }
        let (matches, desired_state) = {
            let old = self.lock();
            let new = replacement.lock();
            let matches = matches!(old.state, MotionState::Running | MotionState::Paused)
                && match (&old.driver, &new.driver) {
                    (Driver::Timeline(old), Driver::Timeline(new)) => old.same_definition(new),
                    _ => false,
                };
            (matches, new.state)
        };
        if !matches {
            return false;
        }
        match desired_state {
            MotionState::Paused => self.pause(),
            MotionState::Running => self.resume(),
            _ => return false,
        }
        true
    }
}
