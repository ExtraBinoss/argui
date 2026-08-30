use crate::{Interpolate, Motion, MotionValue, PhysicsError, SpringConfig, Tween};

#[derive(Clone, Debug, PartialEq)]
enum TransitionDriver {
    Tween(Tween),
    Spring(SpringConfig),
}

/// A validated driver used when a presented style retargets.
#[derive(Clone, Debug, PartialEq)]
pub struct Transition(TransitionDriver);

impl Transition {
    #[must_use]
    pub const fn tween(tween: Tween) -> Self {
        Self(TransitionDriver::Tween(tween))
    }

    #[must_use]
    pub fn spring() -> Self {
        Self(TransitionDriver::Spring(SpringConfig::default()))
    }

    pub fn try_spring(config: SpringConfig) -> Result<Self, PhysicsError> {
        Ok(Self(TransitionDriver::Spring(config.validate()?)))
    }

    #[doc(hidden)]
    pub fn retarget<T>(&self, motion: &Motion<T>, target: T)
    where
        T: MotionValue + Interpolate,
    {
        match &self.0 {
            TransitionDriver::Tween(tween) => motion.animate_to(target, tween.clone()),
            TransitionDriver::Spring(config) => {
                let result = motion.spring_to(target, *config);
                debug_assert!(
                    result.is_ok(),
                    "validated spring configuration must stay valid"
                );
            }
        }
    }
}
