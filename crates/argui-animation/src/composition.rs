use argui_core::{Color, Point, Rect, Size, Transform2D};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Composition {
    #[default]
    Replace,
    Add,
    Accumulate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contribution<T> {
    pub value: T,
    pub composition: Composition,
    pub priority: i32,
    pub order: u64,
    pub completed_iterations: u64,
}

impl<T> Contribution<T> {
    #[must_use]
    pub const fn replace(value: T, priority: i32, order: u64) -> Self {
        Self {
            value,
            composition: Composition::Replace,
            priority,
            order,
            completed_iterations: 0,
        }
    }
}

pub trait Compose: Copy {
    fn add(self, contribution: Self) -> Self;

    fn scale(self, factor: f32) -> Self;
}

#[must_use]
pub fn compose<T: Compose>(base: T, contributions: &mut [Contribution<T>]) -> T {
    contributions.sort_by_key(|item| (item.priority, item.order));
    contributions
        .iter()
        .fold(base, |value, contribution| match contribution.composition {
            Composition::Replace => contribution.value,
            Composition::Add => value.add(contribution.value),
            Composition::Accumulate => value.add(
                contribution
                    .value
                    .scale(contribution.completed_iterations as f32 + 1.0),
            ),
        })
}

impl Compose for f32 {
    fn add(self, contribution: Self) -> Self {
        self + contribution
    }

    fn scale(self, factor: f32) -> Self {
        self * factor
    }
}

impl Compose for f64 {
    fn add(self, contribution: Self) -> Self {
        self + contribution
    }

    fn scale(self, factor: f32) -> Self {
        self * f64::from(factor)
    }
}

impl<const N: usize> Compose for [f32; N] {
    fn add(mut self, contribution: Self) -> Self {
        for (value, contribution) in self.iter_mut().zip(contribution) {
            *value += contribution;
        }
        self
    }

    fn scale(mut self, factor: f32) -> Self {
        for value in &mut self {
            *value *= factor;
        }
        self
    }
}

impl Compose for Color {
    fn add(self, contribution: Self) -> Self {
        let left = self.to_linear_rgba();
        let right = contribution.to_linear_rgba();
        Self::linear_rgba(
            left[0] + right[0],
            left[1] + right[1],
            left[2] + right[2],
            left[3] + right[3],
        )
    }

    fn scale(self, factor: f32) -> Self {
        let value = self.to_linear_rgba();
        Self::linear_rgba(
            value[0] * factor,
            value[1] * factor,
            value[2] * factor,
            value[3] * factor,
        )
    }
}

impl Compose for Point {
    fn add(self, contribution: Self) -> Self {
        Self::new(self.x + contribution.x, self.y + contribution.y)
    }

    fn scale(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}

impl Compose for Size {
    fn add(self, contribution: Self) -> Self {
        Self::new(
            self.width + contribution.width,
            self.height + contribution.height,
        )
    }

    fn scale(self, factor: f32) -> Self {
        Self::new(self.width * factor, self.height * factor)
    }
}

impl Compose for Rect {
    fn add(self, contribution: Self) -> Self {
        Self::new(
            self.origin.add(contribution.origin),
            self.size.add(contribution.size),
        )
    }

    fn scale(self, factor: f32) -> Self {
        Self::new(self.origin.scale(factor), self.size.scale(factor))
    }
}

impl Compose for Transform2D {
    fn add(self, contribution: Self) -> Self {
        Self {
            translation: self.translation.add(contribution.translation),
            scale: self.scale.add(contribution.scale),
            rotation: self.rotation + contribution.rotation,
            skew: self.skew.add(contribution.skew),
        }
    }

    fn scale(self, factor: f32) -> Self {
        Self {
            translation: self.translation.scale(factor),
            scale: self.scale.scale(factor),
            rotation: self.rotation * factor,
            skew: self.skew.scale(factor),
        }
    }
}
