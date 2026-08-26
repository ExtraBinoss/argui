use crate::Duration;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Iterations {
    Finite(f64),
    Infinite,
}

impl Default for Iterations {
    fn default() -> Self {
        Self::Finite(1.0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Direction {
    #[default]
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FillMode {
    #[default]
    None,
    Forwards,
    Backwards,
    Both,
}

impl FillMode {
    pub(crate) const fn fills_before(self) -> bool {
        matches!(self, Self::Backwards | Self::Both)
    }

    pub(crate) const fn fills_after(self) -> bool {
        matches!(self, Self::Forwards | Self::Both)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timing {
    pub duration: Duration,
    pub delay: Duration,
    pub end_delay: Duration,
    pub iterations: Iterations,
    pub direction: Direction,
    pub fill: FillMode,
    pub playback_rate: f64,
}

impl Timing {
    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self {
            duration,
            delay: Duration::ZERO,
            end_delay: Duration::ZERO,
            iterations: Iterations::Finite(1.0),
            direction: Direction::Normal,
            fill: FillMode::None,
            playback_rate: 1.0,
        }
    }

    #[must_use]
    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    #[must_use]
    pub const fn end_delay(mut self, end_delay: Duration) -> Self {
        self.end_delay = end_delay;
        self
    }

    #[must_use]
    pub const fn iterations(mut self, iterations: Iterations) -> Self {
        self.iterations = iterations;
        self
    }

    #[must_use]
    pub const fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn fill(mut self, fill: FillMode) -> Self {
        self.fill = fill;
        self
    }

    #[must_use]
    pub const fn playback_rate(mut self, playback_rate: f64) -> Self {
        self.playback_rate = playback_rate;
        self
    }

    pub(crate) fn validate(self) -> Result<Self, TimingError> {
        let iterations_valid = match self.iterations {
            Iterations::Finite(value) => value.is_finite() && value > 0.0,
            Iterations::Infinite => true,
        };
        if self.duration == Duration::ZERO {
            Err(TimingError::ZeroDuration)
        } else if !iterations_valid {
            Err(TimingError::InvalidIterations)
        } else if !self.playback_rate.is_finite() || self.playback_rate == 0.0 {
            Err(TimingError::InvalidPlaybackRate)
        } else {
            Ok(self)
        }
    }

    pub(crate) fn active_seconds(self) -> f64 {
        match self.iterations {
            Iterations::Finite(iterations) => self.duration.as_secs_f64() * iterations,
            Iterations::Infinite => f64::INFINITY,
        }
    }

    pub(crate) fn total_seconds(self) -> f64 {
        self.delay.as_secs_f64() + self.active_seconds() + self.end_delay.as_secs_f64()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimingError {
    ZeroDuration,
    InvalidIterations,
    InvalidPlaybackRate,
    InvalidKeyframes,
}

impl fmt::Display for TimingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDuration => {
                formatter.write_str("animation duration must be greater than zero")
            }
            Self::InvalidIterations => {
                formatter.write_str("animation iterations must be positive and finite, or infinite")
            }
            Self::InvalidPlaybackRate => {
                formatter.write_str("animation playback rate must be finite and non-zero")
            }
            Self::InvalidKeyframes => {
                formatter.write_str("keyframes must be finite, sorted, and cover offsets 0..=1")
            }
        }
    }
}

impl std::error::Error for TimingError {}
