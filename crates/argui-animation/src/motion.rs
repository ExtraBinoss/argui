use argui_core::{Color, Point, Rect, Size, Transform2D};
use std::fmt;

pub trait MotionValue: Copy + PartialEq {
    fn zero() -> Self;

    fn add(self, other: Self) -> Self;

    fn subtract(self, other: Self) -> Self;

    fn scale(self, factor: f64) -> Self;

    fn magnitude(self) -> f64;
}

macro_rules! scalar_motion {
    ($type:ty) => {
        impl MotionValue for $type {
            fn zero() -> Self {
                0.0
            }

            fn add(self, other: Self) -> Self {
                self + other
            }

            fn subtract(self, other: Self) -> Self {
                self - other
            }

            fn scale(self, factor: f64) -> Self {
                self * factor as Self
            }

            fn magnitude(self) -> f64 {
                self.abs() as f64
            }
        }
    };
}

scalar_motion!(f32);
scalar_motion!(f64);

impl MotionValue for Point {
    fn zero() -> Self {
        Self::default()
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    fn subtract(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor as f32, self.y * factor as f32)
    }

    fn magnitude(self) -> f64 {
        f64::from(self.x).hypot(f64::from(self.y))
    }
}

impl MotionValue for Size {
    fn zero() -> Self {
        Self::default()
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.width + other.width, self.height + other.height)
    }

    fn subtract(self, other: Self) -> Self {
        Self::new(self.width - other.width, self.height - other.height)
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.width * factor as f32, self.height * factor as f32)
    }

    fn magnitude(self) -> f64 {
        f64::from(self.width).hypot(f64::from(self.height))
    }
}

impl MotionValue for Rect {
    fn zero() -> Self {
        Self::default()
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.origin.add(other.origin), self.size.add(other.size))
    }

    fn subtract(self, other: Self) -> Self {
        Self::new(
            self.origin.subtract(other.origin),
            self.size.subtract(other.size),
        )
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.origin.scale(factor), self.size.scale(factor))
    }

    fn magnitude(self) -> f64 {
        self.origin.magnitude().hypot(self.size.magnitude())
    }
}

impl MotionValue for Color {
    fn zero() -> Self {
        Self::TRANSPARENT
    }

    fn add(self, other: Self) -> Self {
        map_color(self, other, |left, right| left + right)
    }

    fn subtract(self, other: Self) -> Self {
        map_color(self, other, |left, right| left - right)
    }

    fn scale(self, factor: f64) -> Self {
        let [red, green, blue, alpha] = self.as_array();
        let factor = factor as f32;
        Self::rgba(red * factor, green * factor, blue * factor, alpha * factor)
    }

    fn magnitude(self) -> f64 {
        self.as_array()
            .into_iter()
            .map(|channel| f64::from(channel).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

impl MotionValue for Transform2D {
    fn zero() -> Self {
        Self {
            translation: Point::default(),
            scale: Point::default(),
            rotation: 0.0,
            skew: Point::default(),
        }
    }

    fn add(self, other: Self) -> Self {
        Self {
            translation: self.translation.add(other.translation),
            scale: self.scale.add(other.scale),
            rotation: self.rotation + other.rotation,
            skew: self.skew.add(other.skew),
        }
    }

    fn subtract(self, other: Self) -> Self {
        Self {
            translation: self.translation.subtract(other.translation),
            scale: self.scale.subtract(other.scale),
            rotation: self.rotation - other.rotation,
            skew: self.skew.subtract(other.skew),
        }
    }

    fn scale(self, factor: f64) -> Self {
        Self {
            translation: self.translation.scale(factor),
            scale: self.scale.scale(factor),
            rotation: self.rotation * factor as f32,
            skew: self.skew.scale(factor),
        }
    }

    fn magnitude(self) -> f64 {
        self.translation
            .magnitude()
            .hypot(self.scale.magnitude())
            .hypot(f64::from(self.rotation))
            .hypot(self.skew.magnitude())
    }
}

fn map_color(left: Color, right: Color, operation: impl Fn(f32, f32) -> f32) -> Color {
    let left = left.as_array();
    let right = right.as_array();
    Color::rgba(
        operation(left[0], right[0]),
        operation(left[1], right[1]),
        operation(left[2], right[2]),
        operation(left[3], right[3]),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicsError {
    InvalidMass,
    InvalidStiffness,
    InvalidDamping,
    InvalidRestThreshold,
    InvalidDecay,
    InvalidBounds,
}

impl fmt::Display for PhysicsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMass => formatter.write_str("spring mass must be finite and positive"),
            Self::InvalidStiffness => {
                formatter.write_str("spring stiffness must be finite and positive")
            }
            Self::InvalidDamping => {
                formatter.write_str("spring damping must be finite and non-negative")
            }
            Self::InvalidRestThreshold => {
                formatter.write_str("rest thresholds must be finite and non-negative")
            }
            Self::InvalidDecay => formatter.write_str("decay rate must be finite and positive"),
            Self::InvalidBounds => formatter.write_str("inertia bounds must be finite and ordered"),
        }
    }
}

impl std::error::Error for PhysicsError {}
