use std::ops::{Add, AddAssign, Sub};

/// A non-negative span represented exactly in nanoseconds.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Duration(u64);

impl Duration {
    /// Zero-length duration.
    pub const ZERO: Self = Self(0);

    /// Creates a duration from nanoseconds.
    /// * `nanos` — number of nanoseconds.
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Creates a duration from microseconds, saturating on overflow.
    /// * `micros` — number of microseconds.
    #[must_use]
    pub const fn from_micros(micros: u64) -> Self {
        Self(micros.saturating_mul(1_000))
    }

    /// Creates a duration from milliseconds, saturating on overflow.
    /// * `millis` — number of milliseconds.
    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis.saturating_mul(1_000_000))
    }

    /// Creates a duration from seconds, saturating on overflow.
    #[must_use]
    pub const fn from_secs(seconds: u64) -> Self {
        Self(seconds.saturating_mul(1_000_000_000))
    }

    /// Returns this duration in nanoseconds.
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// Converts this duration to fractional seconds.
    #[must_use]
    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1_000_000_000.0
    }
}

impl Add for Duration {
    type Output = Self;

    /// Adds durations, saturating at the representable maximum.
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl AddAssign for Duration {
    /// Adds `rhs` to this duration, saturating at the representable maximum.
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Duration {
    type Output = Self;

    /// Subtracts durations, saturating at zero.
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl From<std::time::Duration> for Duration {
    /// Converts a standard duration, saturating if nanoseconds exceed `u64`.
    fn from(value: std::time::Duration) -> Self {
        Self(u64::try_from(value.as_nanos()).unwrap_or(u64::MAX))
    }
}

impl From<Duration> for std::time::Duration {
    /// Converts this duration to the standard library representation.
    fn from(value: Duration) -> Self {
        Self::from_nanos(value.0)
    }
}

/// A monotonic timestamp relative to a clock-defined origin.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Time(u64);

impl Time {
    /// Timestamp at the clock origin.
    pub const ZERO: Self = Self(0);

    /// Creates a timestamp from nanoseconds since its clock origin.
    /// * `nanos` — nanoseconds since the clock origin.
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Returns nanoseconds since the clock origin.
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// Returns the non-negative elapsed duration since `earlier`.
    #[must_use]
    pub const fn duration_since(self, earlier: Self) -> Duration {
        Duration::from_nanos(self.0.saturating_sub(earlier.0))
    }
}

impl Add<Duration> for Time {
    type Output = Self;

    /// Advances the timestamp by `rhs`, saturating on overflow.
    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0.saturating_add(rhs.as_nanos()))
    }
}

impl Sub<Time> for Time {
    type Output = Duration;

    /// Returns elapsed time from `rhs` to this timestamp, saturating at zero.
    fn sub(self, rhs: Time) -> Self::Output {
        self.duration_since(rhs)
    }
}
