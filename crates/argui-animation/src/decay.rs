use crate::{Duration, MotionValue, PhysicsError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecayConfig {
    pub rate: f64,
    pub rest_speed: f64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            rate: 5.0,
            rest_speed: 0.01,
        }
    }
}

impl DecayConfig {
    pub fn validate(self) -> Result<Self, PhysicsError> {
        if !self.rate.is_finite() || self.rate <= 0.0 {
            Err(PhysicsError::InvalidDecay)
        } else if !self.rest_speed.is_finite() || self.rest_speed < 0.0 {
            Err(PhysicsError::InvalidRestThreshold)
        } else {
            Ok(self)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decay<T> {
    value: T,
    velocity: T,
    config: DecayConfig,
    active: bool,
}

impl<T: MotionValue> Decay<T> {
    pub fn new(value: T, velocity: T, config: DecayConfig) -> Result<Self, PhysicsError> {
        let config = config.validate()?;
        let active = velocity.magnitude() > config.rest_speed;
        Ok(Self {
            value,
            velocity,
            config,
            active,
        })
    }

    #[must_use]
    pub const fn value(&self) -> T {
        self.value
    }

    #[must_use]
    pub const fn velocity(&self) -> T {
        self.velocity
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    pub fn kick(&mut self, velocity: T) {
        self.velocity = velocity;
        self.active = velocity.magnitude() > self.config.rest_speed;
    }

    pub fn advance(&mut self, elapsed: Duration) -> bool {
        if !self.active || elapsed == Duration::ZERO {
            return false;
        }
        let decay = (-self.config.rate * elapsed.as_secs_f64()).exp();
        let distance = self.velocity.scale((1.0 - decay) / self.config.rate);
        self.value = self.value.add(distance);
        self.velocity = self.velocity.scale(decay);
        if self.velocity.magnitude() <= self.config.rest_speed {
            self.velocity = T::zero();
            self.active = false;
        }
        true
    }
}
