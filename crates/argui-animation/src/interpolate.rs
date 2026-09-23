use argui_core::{Color, ColorInterpolation, Point, Rect, Size, Transform2D, TransformOrigin};

/// Produces a value between two typed endpoints.
pub trait Interpolate: Sized {
    #[must_use]
    /// Interpolates from `self` toward `target` by `progress`.
    fn interpolate(self, target: Self, progress: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        self + (target - self) * progress
    }
}

impl Interpolate for f64 {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        self + (target - self) * f64::from(progress)
    }
}

impl<const N: usize> Interpolate for [f32; N] {
    fn interpolate(mut self, target: Self, progress: f32) -> Self {
        for (value, target) in self.iter_mut().zip(target) {
            *value = value.interpolate(target, progress);
        }
        self
    }
}

impl Interpolate for Color {
    /// Mixes toward `target` in Oklab at `progress`, preserving equal colors and
    /// exact zero/one endpoints to avoid rounding drift when playback resumes.
    fn interpolate(self, target: Self, progress: f32) -> Self {
        if progress == 0.0 || self == target {
            self
        } else if progress == 1.0 {
            target
        } else {
            self.mix(target, progress, ColorInterpolation::Oklab)
        }
    }
}

impl Interpolate for Point {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        Self::new(
            self.x.interpolate(target.x, progress),
            self.y.interpolate(target.y, progress),
        )
    }
}

impl Interpolate for Size {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        Self::new(
            self.width.interpolate(target.width, progress),
            self.height.interpolate(target.height, progress),
        )
    }
}

impl Interpolate for Rect {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        Self::new(
            self.origin.interpolate(target.origin, progress),
            self.size.interpolate(target.size, progress),
        )
    }
}

impl Interpolate for Transform2D {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        Self {
            translation: self.translation.interpolate(target.translation, progress),
            scale: self.scale.interpolate(target.scale, progress),
            rotation: self.rotation.interpolate(target.rotation, progress),
            skew: self.skew.interpolate(target.skew, progress),
        }
    }
}

impl Interpolate for TransformOrigin {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        Self::new(
            self.x.interpolate(target.x, progress),
            self.y.interpolate(target.y, progress),
        )
    }
}
