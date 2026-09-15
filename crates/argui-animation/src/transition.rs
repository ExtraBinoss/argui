use crate::{Interpolate, Motion, MotionValue, PhysicsError, SpringConfig, Tween};

#[derive(Clone, Debug, PartialEq)]
enum TransitionDriver {
    Tween(Tween),
    Spring(SpringConfig),
}

/// A validated driver used when a presented style retargets.
#[derive(Clone, Debug, PartialEq)]
/// Validated driver used to retarget a motion.
pub struct Transition(TransitionDriver);

impl Transition {
    /// Creates a transition driven by the supplied tween.
    #[must_use]
    pub const fn tween(tween: Tween) -> Self {
        Self(TransitionDriver::Tween(tween))
    }

    /// Creates a spring transition using the default spring configuration.
    #[must_use]
    pub fn spring() -> Self {
        Self(TransitionDriver::Spring(SpringConfig::default()))
    }

    /// Creates a spring transition after validating its configuration.
    ///
    /// # Errors
    /// Returns a physics error if `config` contains invalid spring parameters.
    pub fn try_spring(config: SpringConfig) -> Result<Self, PhysicsError> {
        Ok(Self(TransitionDriver::Spring(config.validate()?)))
    }

    /// Applies this driver to `motion`, retargeting it to `target`.
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
