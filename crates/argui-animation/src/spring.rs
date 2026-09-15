use crate::{Duration, MotionValue, PhysicsError};

#[derive(Clone, Copy, Debug, PartialEq)]
/// Physical parameters and rest thresholds for a spring.
pub struct SpringConfig {
    /// Positive mass.
    pub mass: f64,
    /// Positive spring stiffness.
    pub stiffness: f64,
    /// Non-negative damping coefficient.
    pub damping: f64,
    /// Velocity magnitude at or below which the spring can rest.
    pub rest_speed: f64,
    /// Displacement magnitude at or below which the spring can rest.
    pub rest_delta: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            mass: 1.0,
            stiffness: 170.0,
            damping: 26.0,
            rest_speed: 0.001,
            rest_delta: 0.001,
        }
    }
}

impl SpringConfig {
    /// Checks that the spring coefficients and rest thresholds are valid.
    ///
    /// # Errors
    /// Returns the corresponding [`PhysicsError`] for an invalid mass, stiffness,
    /// damping coefficient, or rest threshold.
    pub fn validate(self) -> Result<Self, PhysicsError> {
        if !self.mass.is_finite() || self.mass <= 0.0 {
            Err(PhysicsError::InvalidMass)
        } else if !self.stiffness.is_finite() || self.stiffness <= 0.0 {
            Err(PhysicsError::InvalidStiffness)
        } else if !self.damping.is_finite() || self.damping < 0.0 {
            Err(PhysicsError::InvalidDamping)
        } else if !self.rest_speed.is_finite()
            || self.rest_speed < 0.0
            || !self.rest_delta.is_finite()
            || self.rest_delta < 0.0
        {
            Err(PhysicsError::InvalidRestThreshold)
        } else {
            Ok(self)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Analytically integrated spring motion for a supported [`MotionValue`].
pub struct Spring<T> {
    value: T,
    target: T,
    velocity: T,
    config: SpringConfig,
    active: bool,
}

impl<T: MotionValue> Spring<T> {
    /// Creates a spring from current value, target, initial velocity, and config.
    ///
    /// # Errors
    /// Returns a physics error if `config` contains invalid coefficients or thresholds.
    pub fn new(
        value: T,
        target: T,
        velocity: T,
        config: SpringConfig,
    ) -> Result<Self, PhysicsError> {
        let config = config.validate()?;
        let active = value.subtract(target).magnitude() > config.rest_delta
            || velocity.magnitude() > config.rest_speed;
        Ok(Self {
            value,
            target,
            velocity,
            config,
            active,
        })
    }

    /// Returns the current spring value.
    #[must_use]
    pub const fn value(&self) -> T {
        self.value
    }

    /// Returns the spring target.
    #[must_use]
    pub const fn target(&self) -> T {
        self.target
    }

    /// Returns the current spring velocity.
    #[must_use]
    pub const fn velocity(&self) -> T {
        self.velocity
    }

    /// Returns whether the spring has not yet reached its rest thresholds.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Changes the target while preserving the current value and velocity.
    pub fn retarget(&mut self, target: T) {
        self.target = target;
        self.active = self.value.subtract(target).magnitude() > self.config.rest_delta
            || self.velocity.magnitude() > self.config.rest_speed;
    }

    /// Replaces the current velocity and updates the active state.
    pub fn set_velocity(&mut self, velocity: T) {
        self.velocity = velocity;
        self.active = self.value.subtract(self.target).magnitude() > self.config.rest_delta
            || velocity.magnitude() > self.config.rest_speed;
    }

    /// Advances the spring by `elapsed`; returns whether its value changed.
    pub fn advance(&mut self, elapsed: Duration) -> bool {
        if !self.active || elapsed == Duration::ZERO {
            return false;
        }
        let seconds = elapsed.as_secs_f64();
        let displacement = self.value.subtract(self.target);
        let frequency = (self.config.stiffness / self.config.mass).sqrt();
        let damping_ratio =
            self.config.damping / (2.0 * (self.config.stiffness * self.config.mass).sqrt());
        let (next_displacement, next_velocity) = if damping_ratio < 1.0 - f64::EPSILON {
            underdamped(
                displacement,
                self.velocity,
                frequency,
                damping_ratio,
                seconds,
            )
        } else if damping_ratio > 1.0 + f64::EPSILON {
            overdamped(
                displacement,
                self.velocity,
                frequency,
                damping_ratio,
                seconds,
            )
        } else {
            critically_damped(displacement, self.velocity, frequency, seconds)
        };
        self.value = self.target.add(next_displacement);
        self.velocity = next_velocity;
        if next_displacement.magnitude() <= self.config.rest_delta
            && next_velocity.magnitude() <= self.config.rest_speed
        {
            self.value = self.target;
            self.velocity = T::zero();
            self.active = false;
        }
        true
    }
}

fn underdamped<T: MotionValue>(
    displacement: T,
    velocity: T,
    frequency: f64,
    ratio: f64,
    seconds: f64,
) -> (T, T) {
    let damped = frequency * (1.0 - ratio * ratio).sqrt();
    let decay = (-ratio * frequency * seconds).exp();
    let cosine = (damped * seconds).cos();
    let sine = (damped * seconds).sin();
    let secondary = velocity
        .add(displacement.scale(ratio * frequency))
        .scale(1.0 / damped);
    let position_wave = displacement.scale(cosine).add(secondary.scale(sine));
    let velocity_wave = displacement
        .scale(-damped * sine)
        .add(secondary.scale(damped * cosine))
        .add(position_wave.scale(-ratio * frequency));
    (position_wave.scale(decay), velocity_wave.scale(decay))
}

fn critically_damped<T: MotionValue>(
    displacement: T,
    velocity: T,
    frequency: f64,
    seconds: f64,
) -> (T, T) {
    let decay = (-frequency * seconds).exp();
    let coefficient = velocity.add(displacement.scale(frequency));
    let position_wave = displacement.add(coefficient.scale(seconds));
    let velocity_wave = coefficient.subtract(position_wave.scale(frequency));
    (position_wave.scale(decay), velocity_wave.scale(decay))
}

fn overdamped<T: MotionValue>(
    displacement: T,
    velocity: T,
    frequency: f64,
    ratio: f64,
    seconds: f64,
) -> (T, T) {
    let root = (ratio * ratio - 1.0).sqrt();
    let first_rate = -frequency * (ratio - root);
    let second_rate = -frequency * (ratio + root);
    let first = velocity
        .subtract(displacement.scale(second_rate))
        .scale(1.0 / (first_rate - second_rate));
    let second = displacement.subtract(first);
    let first_decay = (first_rate * seconds).exp();
    let second_decay = (second_rate * seconds).exp();
    let position = first.scale(first_decay).add(second.scale(second_decay));
    let velocity = first
        .scale(first_rate * first_decay)
        .add(second.scale(second_rate * second_decay));
    (position, velocity)
}
