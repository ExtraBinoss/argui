use std::{fmt, sync::Arc};

type CustomFunction = dyn Fn(f32) -> f32 + Send + Sync + 'static;

#[derive(Clone, Default)]
pub enum Easing {
    #[default]
    Linear,
    CubicBezier(CubicBezier),
    Steps(Steps),
    PiecewiseLinear(Box<[LinearStop]>),
    Custom(Arc<CustomFunction>),
}

impl Easing {
    #[must_use]
    pub fn custom(function: impl Fn(f32) -> f32 + Send + Sync + 'static) -> Self {
        Self::Custom(Arc::new(function))
    }

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl CubicBezier {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Result<Self, EasingError> {
        if [x1, y1, x2, y2].iter().any(|value| !value.is_finite())
            || !(0.0..=1.0).contains(&x1)
            || !(0.0..=1.0).contains(&x2)
        {
            return Err(EasingError::InvalidBezier);
        }
        Ok(Self { x1, y1, x2, y2 })
    }

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

fn cubic(parameter: f32, first: f32, second: f32) -> f32 {
    let inverse = 1.0 - parameter;
    3.0 * inverse * inverse * parameter * first
        + 3.0 * inverse * parameter * parameter * second
        + parameter * parameter * parameter
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepPosition {
    JumpStart,
    JumpEnd,
    JumpNone,
    JumpBoth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Steps {
    count: u32,
    position: StepPosition,
}

impl Steps {
    pub fn new(count: u32, position: StepPosition) -> Result<Self, EasingError> {
        if count == 0 || (position == StepPosition::JumpNone && count == 1) {
            return Err(EasingError::InvalidSteps);
        }
        Ok(Self { count, position })
    }

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearStop {
    pub input: f32,
    pub output: f32,
}

impl LinearStop {
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
pub enum EasingError {
    InvalidBezier,
    InvalidSteps,
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
