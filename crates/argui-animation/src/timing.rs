use crate::Duration;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
/// Number of times an animation's active interval is played.
pub enum Iterations {
    /// A positive finite iteration count; fractional counts play a partial final iteration.
    Finite(f64),
    /// Repeat without a finite end.
    Infinite,
}

impl Default for Iterations {
    fn default() -> Self {
        Self::Finite(1.0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Playback direction applied to successive iterations.
pub enum Direction {
    /// Play each iteration from start to end.
    #[default]
    Normal,
    /// Play each iteration from end to start.
    Reverse,
    /// Alternate direction after each iteration, starting forward.
    Alternate,
    /// Alternate direction after each iteration, starting backward.
    AlternateReverse,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Whether a timeline contributes values outside its active interval.
pub enum FillMode {
    /// Do not contribute before or after the active interval.
    #[default]
    None,
    /// Preserve the final value after the active interval.
    Forwards,
    /// Contribute the initial value during the delay.
    Backwards,
    /// Apply both backwards and forwards filling.
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
/// Timing parameters controlling a timeline's playback.
pub struct Timing {
    /// Duration of one iteration.
    pub duration: Duration,
    /// Delay before the active interval begins.
    pub delay: Duration,
    /// Delay after the active interval ends.
    pub end_delay: Duration,
    /// Number of iterations.
    pub iterations: Iterations,
    /// Direction of each iteration.
    pub direction: Direction,
    /// Fill behavior outside the active interval.
    pub fill: FillMode,
    /// Playback speed multiplier; negative values play backwards.
    pub playback_rate: f64,
}

impl Timing {
    /// Creates timing for one forward iteration with no delay or fill.
    /// * `duration` — active interval length.
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

    /// Sets the delay before the active interval.
    #[must_use]
    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Sets the delay after the active interval.
    /// * `end_delay` — time appended after the active interval.
    #[must_use]
    pub const fn end_delay(mut self, end_delay: Duration) -> Self {
        self.end_delay = end_delay;
        self
    }

    /// Sets the number of iterations.
    #[must_use]
    pub const fn iterations(mut self, iterations: Iterations) -> Self {
        self.iterations = iterations;
        self
    }

    /// Sets how iteration directions are selected.
    /// * `direction` — direction policy applied to each iteration.
    #[must_use]
    pub const fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the fill behavior outside the active interval.
    #[must_use]
    pub const fn fill(mut self, fill: FillMode) -> Self {
        self.fill = fill;
        self
    }

    /// Sets the playback speed multiplier.
    /// * `playback_rate` — nonzero speed multiplier; the sign selects direction.
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
/// Error returned for invalid timeline timing or keyframes.
pub enum TimingError {
    /// The per-iteration duration is zero.
    ZeroDuration,
    /// The finite iteration count is non-positive or not finite.
    InvalidIterations,
    /// The playback rate is zero or not finite.
    InvalidPlaybackRate,
    /// Keyframes are not finite, sorted, or covering offsets zero through one.
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
