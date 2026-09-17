use std::{fmt, sync::Arc};

type CustomFunction = dyn Fn(f32) -> f32 + Send + Sync + 'static;

/// Maps normalized time to animation progress.
///
/// Argui clamps the input supplied through [`Easing::curve`] to `0.0..=1.0`.
/// Implementations may return values outside that interval to create anticipation
/// or overshoot. Curves should return finite values and preserve the endpoints
/// when they are intended to finish exactly on their animated target.
pub trait Curve: Send + Sync + 'static {
    /// Samples the curve at normalized `progress` and returns transformed progress.
    #[must_use]
    fn sample(&self, progress: f32) -> f32;
}

impl<F> Curve for F
where
    F: Fn(f32) -> f32 + Send + Sync + 'static,
{
    fn sample(&self, progress: f32) -> f32 {
        self(progress)
    }
}

#[derive(Clone, Default)]
/// Maps normalized animation progress to eased progress.
pub enum Easing {
    /// Linear interpolation with no easing.
    #[default]
    Linear,
    /// Cubic Bézier easing curve.
    CubicBezier(CubicBezier),
    /// Discrete step easing.
    Steps(Steps),
    /// Piecewise-linear easing defined by stops.
    PiecewiseLinear(Box<[LinearStop]>),
    /// User-provided easing function.
    Custom(Arc<CustomFunction>),
}

impl Easing {
    /// Wraps a thread-safe custom function as an easing curve.
    #[must_use]
    pub fn custom(function: impl Fn(f32) -> f32 + Send + Sync + 'static) -> Self {
        Self::Custom(Arc::new(function))
    }

    /// Wraps a user-defined [`Curve`] as an easing value.
    #[must_use]
    pub fn curve(curve: impl Curve) -> Self {
        Self::Custom(Arc::new(move |progress| curve.sample(progress)))
    }

    /// Creates piecewise-linear easing from sorted stops spanning input zero to one.
    ///
    /// # Errors
    /// Returns [`EasingError::InvalidStops`] if stops are missing, unsorted, outside
    /// the normalized input interval, or contain non-finite coordinates.
    pub fn piecewise_linear(stops: impl Into<Vec<LinearStop>>) -> Result<Self, EasingError> {
        let stops = stops.into();
        if stops.len() < 2
            || stops.first().is_none_or(|stop| stop.input != 0.0)
            || stops.last().is_none_or(|stop| stop.input != 1.0)
            || stops.iter().any(|stop| {
                !stop.input.is_finite()
                    || !stop.output.is_finite()
                    || !(0.0..=1.0).contains(&stop.input)
            })
            || stops.windows(2).any(|pair| pair[0].input > pair[1].input)
        {
            return Err(EasingError::InvalidStops);
        }
        Ok(Self::PiecewiseLinear(stops.into_boxed_slice()))
    }

    /// Evaluates the easing curve at normalized `progress`, clamped to zero through one.
    #[must_use]
    pub fn sample(&self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Linear => progress,
            Self::CubicBezier(curve) => curve.sample(progress),
            Self::Steps(steps) => steps.sample(progress),
            Self::PiecewiseLinear(stops) => sample_stops(stops, progress),
            Self::Custom(function) => function(progress),
        }
    }
}

impl Curve for Easing {
    fn sample(&self, progress: f32) -> f32 {
        Easing::sample(self, progress)
    }
}

impl fmt::Debug for Easing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Linear => formatter.write_str("Linear"),
            Self::CubicBezier(value) => formatter.debug_tuple("CubicBezier").field(value).finish(),
            Self::Steps(value) => formatter.debug_tuple("Steps").field(value).finish(),
            Self::PiecewiseLinear(value) => formatter
                .debug_tuple("PiecewiseLinear")
                .field(value)
                .finish(),
            Self::Custom(_) => formatter.write_str("Custom(..)"),
        }
    }
}

impl PartialEq for Easing {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Linear, Self::Linear) => true,
            (Self::CubicBezier(left), Self::CubicBezier(right)) => left == right,
            (Self::Steps(left), Self::Steps(right)) => left == right,
            (Self::PiecewiseLinear(left), Self::PiecewiseLinear(right)) => left == right,
            (Self::Custom(left), Self::Custom(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Cubic Bézier easing curve with endpoints fixed at (0, 0) and (1, 1).
pub struct CubicBezier {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl CubicBezier {
    /// Creates a curve from its two control points.
    /// * `x1`, `y1` — first control point; `x2`, `y2` — second control point.
    ///
    /// # Errors
    /// Returns [`EasingError::InvalidBezier`] if any coordinate is non-finite or
    /// either control-point x coordinate is outside zero through one.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Result<Self, EasingError> {
        if [x1, y1, x2, y2].iter().any(|value| !value.is_finite())
            || !(0.0..=1.0).contains(&x1)
            || !(0.0..=1.0).contains(&x2)
        {
            return Err(EasingError::InvalidBezier);
        }
        Ok(Self { x1, y1, x2, y2 })
    }

    /// Evaluates the curve at normalized `progress`, clamped to zero through one.
    #[must_use]
    pub fn sample(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        let mut low = 0.0;
        let mut high = 1.0;
        let mut parameter = progress;
        for _ in 0..16 {
            let x = cubic(parameter, self.x1, self.x2);
            if (x - progress).abs() <= 1.0e-5 {
                break;
            }
            if x < progress {
                low = parameter;
            } else {
                high = parameter;
            }
            parameter = (low + high) * 0.5;
        }
        cubic(parameter, self.y1, self.y2)
    }
}

impl Curve for CubicBezier {
    fn sample(&self, progress: f32) -> f32 {
        (*self).sample(progress)
    }
}

fn cubic(parameter: f32, first: f32, second: f32) -> f32 {
    let inverse = 1.0 - parameter;
    3.0 * inverse * inverse * parameter * first
        + 3.0 * inverse * parameter * parameter * second
        + parameter * parameter * parameter
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Placement of jumps in a [`Steps`] easing curve.
pub enum StepPosition {
    /// Jump at the start of each interval.
    JumpStart,
    /// Jump at the end of each interval.
    JumpEnd,
    /// Omit jumps at both endpoints.
    JumpNone,
    /// Include jumps at both endpoints.
    JumpBoth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Step-based easing with a jump count and endpoint policy.
pub struct Steps {
    count: u32,
    position: StepPosition,
}

impl Steps {
    /// Creates a step curve with `count` jumps and the selected endpoint policy.
    /// * `position` — policy for jump behavior at interval boundaries.
    ///
    /// # Errors
    /// Returns [`EasingError::InvalidSteps`] when the count is zero, or when
    /// `JumpNone` is used with fewer than two jumps.
    pub fn new(count: u32, position: StepPosition) -> Result<Self, EasingError> {
        if count == 0 || (position == StepPosition::JumpNone && count == 1) {
            return Err(EasingError::InvalidSteps);
        }
        Ok(Self { count, position })
    }

    /// Evaluates the step curve at normalized `progress`, clamped to zero through one.
    #[must_use]
    pub fn sample(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        let count = self.count as f32;
        match self.position {
            StepPosition::JumpStart => ((progress * count).floor() + 1.0).min(count) / count,
            StepPosition::JumpEnd => (progress * count).floor() / count,
            StepPosition::JumpNone => ((progress * count).floor() / (count - 1.0)).clamp(0.0, 1.0),
            StepPosition::JumpBoth => ((progress * count).floor() + 1.0) / (count + 1.0),
        }
    }
}

impl Curve for Steps {
    fn sample(&self, progress: f32) -> f32 {
        (*self).sample(progress)
    }
}

/// Ready-to-use curves for common interface motion.
pub mod curves {
    use super::{CubicBezier, Easing};

    /// Constant-speed progress.
    pub const LINEAR: Easing = Easing::Linear;
    /// Gentle acceleration from rest.
    pub const EASE_IN: Easing = cubic(0.42, 0.0, 1.0, 1.0);
    /// Gentle deceleration into the target.
    pub const EASE_OUT: Easing = cubic(0.0, 0.0, 0.58, 1.0);
    /// Symmetric acceleration and deceleration.
    pub const EASE_IN_OUT: Easing = cubic(0.42, 0.0, 0.58, 1.0);
    /// Balanced material-style movement for most interface changes.
    pub const STANDARD: Easing = cubic(0.2, 0.0, 0.0, 1.0);
    /// Fast exit from the current state followed by a soft arrival.
    pub const EMPHASIZED: Easing = cubic(0.2, 0.8, 0.2, 1.0);
    /// Quickly leaves the initial value.
    pub const ACCELERATE: Easing = cubic(0.3, 0.0, 0.8, 0.15);
    /// Quickly becomes visible and then settles gently.
    pub const DECELERATE: Easing = cubic(0.05, 0.7, 0.1, 1.0);
    /// Overshoots the target before settling on it.
    pub const BACK_OUT: Easing = cubic(0.34, 1.56, 0.64, 1.0);

    /// Creates a constant cubic Bézier easing for the preset catalog.
    const fn cubic(x1: f32, y1: f32, x2: f32, y2: f32) -> Easing {
        Easing::CubicBezier(CubicBezier { x1, y1, x2, y2 })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Input/output pair defining a point on piecewise-linear easing.
pub struct LinearStop {
    /// Input progress coordinate.
    pub input: f32,
    /// Output progress coordinate.
    pub output: f32,
}

impl LinearStop {
    /// Creates a stop from its input and output coordinates.
    #[must_use]
    pub const fn new(input: f32, output: f32) -> Self {
        Self { input, output }
    }
}

fn sample_stops(stops: &[LinearStop], progress: f32) -> f32 {
    let upper = stops.partition_point(|stop| stop.input <= progress);
    if upper == stops.len() {
        return stops[upper - 1].output;
    }
    let from = stops[upper - 1];
    let to = stops[upper];
    if from.input == to.input {
        return to.output;
    }
    let local = (progress - from.input) / (to.input - from.input);
    from.output + (to.output - from.output) * local
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error returned when easing parameters are invalid.
pub enum EasingError {
    /// Bézier control points are invalid.
    InvalidBezier,
    /// Step count or position is invalid.
    InvalidSteps,
    /// Piecewise-linear stops are invalid.
    InvalidStops,
}

impl fmt::Display for EasingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBezier => formatter.write_str(
                "Bezier x coordinates must be within 0..=1 and all coordinates must be finite",
            ),
            Self::InvalidSteps => {
                formatter.write_str("steps require at least one jump, or two for jump-none")
            }
            Self::InvalidStops => {
                formatter.write_str("linear stops must be finite, sorted, and cover 0..=1")
            }
        }
    }
}

impl std::error::Error for EasingError {}
