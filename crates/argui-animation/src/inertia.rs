use crate::{Decay, DecayConfig, Duration, PhysicsError, Spring, SpringConfig};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InertiaConfig {
    pub decay: DecayConfig,
    pub bounce: SpringConfig,
    pub bounds: Option<(f32, f32)>,
}

impl Default for InertiaConfig {
    fn default() -> Self {
        Self {
            decay: DecayConfig::default(),
            bounce: SpringConfig {
                stiffness: 240.0,
                damping: 28.0,
                ..SpringConfig::default()
            },
            bounds: None,
        }
    }
}

impl InertiaConfig {
    pub fn validate(self) -> Result<Self, PhysicsError> {
        self.decay.validate()?;
        self.bounce.validate()?;
        if self.bounds.is_some_and(|(minimum, maximum)| {
            !minimum.is_finite() || !maximum.is_finite() || minimum > maximum
        }) {
            Err(PhysicsError::InvalidBounds)
        } else {
            Ok(self)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InertiaState {
    #[default]
    Settled,
    Decaying,
    Bouncing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inertia {
    decay: Decay<f32>,
    spring: Option<Spring<f32>>,
    config: InertiaConfig,
    state: InertiaState,
}

impl Inertia {
    pub fn new(value: f32, velocity: f32, config: InertiaConfig) -> Result<Self, PhysicsError> {
        let config = config.validate()?;
        let decay = Decay::new(value, velocity, config.decay)?;
        let bound = bound_target(config.bounds, value);
        let spring = bound.map(|target| {
            Spring::new(value, target, velocity, config.bounce)
                .expect("validated inertia contains a valid bounce spring")
        });
        let state = if spring.is_some() {
            InertiaState::Bouncing
        } else if decay.is_active() {
            InertiaState::Decaying
        } else {
            InertiaState::Settled
        };
        Ok(Self {
            decay,
            spring,
            config,
            state,
        })
    }

    #[must_use]
    pub fn value(&self) -> f32 {
        self.spring
            .map_or_else(|| self.decay.value(), |spring| spring.value())
    }

    #[must_use]
    pub fn velocity(&self) -> f32 {
        self.spring
            .map_or_else(|| self.decay.velocity(), |spring| spring.velocity())
    }

    #[must_use]
    pub const fn state(&self) -> InertiaState {
        self.state
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state != InertiaState::Settled
    }

    pub fn advance(&mut self, elapsed: Duration) -> bool {
        match &mut self.spring {
            Some(spring) => {
                let changed = spring.advance(elapsed);
                if !spring.is_active() {
                    self.state = InertiaState::Settled;
                }
                changed
            }
            None => {
                let changed = self.decay.advance(elapsed);
                if let Some(target) = self.bound_target() {
                    self.spring = Some(
                        Spring::new(
                            self.decay.value(),
                            target,
                            self.decay.velocity(),
                            self.config.bounce,
                        )
                        .expect("validated inertia contains a valid bounce spring"),
                    );
                    self.state = InertiaState::Bouncing;
                } else if !self.decay.is_active() {
                    self.state = InertiaState::Settled;
                }
                changed
            }
        }
    }

    pub fn launch(&mut self, value: f32, velocity: f32) {
        *self = Self::new(value, velocity, self.config)
            .expect("an existing inertia always retains valid configuration");
    }

    fn bound_target(&self) -> Option<f32> {
        bound_target(self.config.bounds, self.decay.value())
    }
}

fn bound_target(bounds: Option<(f32, f32)>, value: f32) -> Option<f32> {
    let (minimum, maximum) = bounds?;
    if value < minimum {
        Some(minimum)
    } else if value > maximum {
        Some(maximum)
    } else {
        None
    }
}
