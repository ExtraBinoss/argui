use argui_core::{Affine2D, Color, ColorInterpolation, Point, Rect};

use crate::{
    BilinearGradient, ClipChain, ConicGradient, GradientError, GradientStop, GradientStops,
    ImageFit, ImageId, ImageSampling, LinearGradient, RadialGradient,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Fill {
    Solid(Color),
    Linear(LinearGradient),
    Radial(RadialGradient),
    Conic(ConicGradient),
    Bilinear(BilinearGradient),
}

impl Fill {
    /// Creates a linear GPU fill with any number of ordered stops.
    ///
    /// * `colors` — colors at each stop.
    /// * `offsets` — corresponding normalized stop positions.
    /// * `angle_degrees` — clockwise direction of the color transition.
    /// * `space` — `oklab`, `linear-srgb`, or `srgb` interpolation.
    ///
    /// # Errors
    ///
    /// Returns a gradient error for mismatched, invalid, or unordered stops or an unknown color space.
    pub fn linear_gradient(
        colors: &[Color],
        offsets: &[f32],
        angle_degrees: f32,
        space: &str,
    ) -> Result<Self, GradientError> {
        let radians = angle_degrees.to_radians();
        let (sine, cosine) = radians.sin_cos();
        let start = Point::new(0.5 - cosine * 0.5, 0.5 - sine * 0.5);
        let end = Point::new(0.5 + cosine * 0.5, 0.5 + sine * 0.5);
        Ok(Self::Linear(LinearGradient::with_stops(
            start,
            end,
            interpolation(space)?,
            gradient_stops(colors, offsets)?,
        )))
    }

    /// Creates a circular or elliptical radial GPU fill with ordered stops.
    ///
    /// * `colors` — colors at each stop.
    /// * `offsets` — corresponding normalized stop positions.
    /// * `center` — relative center of the gradient.
    /// * `radius` — relative horizontal and vertical radii.
    /// * `space` — interpolation color space.
    ///
    /// # Errors
    ///
    /// Returns a gradient error for invalid stops or color space.
    pub fn radial_gradient(
        colors: &[Color],
        offsets: &[f32],
        center: Point,
        radius: Point,
        space: &str,
    ) -> Result<Self, GradientError> {
        Ok(Self::Radial(RadialGradient::with_stops(
            center,
            radius,
            interpolation(space)?,
            gradient_stops(colors, offsets)?,
        )))
    }

    /// Creates an angular GPU fill with ordered stops around one revolution.
    ///
    /// * `colors` — colors at each stop.
    /// * `offsets` — corresponding normalized stop positions.
    /// * `center` — relative center of the gradient.
    /// * `start_angle` — clockwise starting angle in degrees.
    /// * `space` — interpolation color space.
    ///
    /// # Errors
    ///
    /// Returns a gradient error for invalid stops or color space.
    pub fn conic_gradient(
        colors: &[Color],
        offsets: &[f32],
        center: Point,
        start_angle: f32,
        space: &str,
    ) -> Result<Self, GradientError> {
        Ok(Self::Conic(ConicGradient::with_stops(
            center,
            start_angle,
            interpolation(space)?,
            gradient_stops(colors, offsets)?,
        )))
    }
}

/// Validates matching arbitrary-length arrays before creating reusable GPU stops.
fn gradient_stops(colors: &[Color], offsets: &[f32]) -> Result<GradientStops, GradientError> {
    if colors.len() != offsets.len() {
        return Err(GradientError::MismatchedStops);
    }
    GradientStops::from_vec(
        colors
            .iter()
            .copied()
            .zip(offsets.iter().copied())
            .map(|(color, offset)| GradientStop::new(offset, color))
            .collect(),
    )
}

/// Resolves an authored color-space name to the GPU interpolation mode.
fn interpolation(name: &str) -> Result<ColorInterpolation, GradientError> {
    match name {
        "oklab" => Ok(ColorInterpolation::Oklab),
        "linear-srgb" => Ok(ColorInterpolation::LinearSrgb),
        "srgb" => Ok(ColorInterpolation::Srgb),
        _ => Err(GradientError::InvalidInterpolation),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    /// Creates equal radii for all four corners.
    /// * `radius` — radius assigned to every corner.
    #[must_use]
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Returns radii in top-left, top-right, bottom-right, bottom-left order.
    #[must_use]
    pub const fn as_array(self) -> [f32; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }

    /// Scales every corner radius by `factor`.
    #[must_use]
    pub const fn scaled(self, factor: f32) -> Self {
        Self {
            top_left: self.top_left * factor,
            top_right: self.top_right * factor,
            bottom_right: self.bottom_right * factor,
            bottom_left: self.bottom_left * factor,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderWidths {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl BorderWidths {
    /// Creates equal border widths on all edges.
    /// * `width` — width assigned to each edge.
    #[must_use]
    pub const fn all(width: f32) -> Self {
        Self {
            left: width,
            right: width,
            top: width,
            bottom: width,
        }
    }

    /// Returns widths in left, right, top, bottom order.
    #[must_use]
    pub const fn as_array(self) -> [f32; 4] {
        [self.left, self.right, self.top, self.bottom]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    pub widths: BorderWidths,
    pub color: Color,
}

impl Border {
    /// Creates a border with equal width on every edge and the supplied color.
    #[must_use]
    pub const fn all(width: f32, color: Color) -> Self {
        Self {
            widths: BorderWidths::all(width),
            color,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuadStyle {
    pub background: Option<Fill>,
    pub border: Option<Border>,
    pub radii: CornerRadii,
    pub opacity: f32,
}

impl Default for QuadStyle {
    fn default() -> Self {
        Self {
            background: None,
            border: None,
            radii: CornerRadii::all(0.0),
            opacity: 1.0,
        }
    }
}

impl QuadStyle {
    /// Creates a style with a solid background and no border.
    /// * `color` — solid background color.
    #[must_use]
    pub const fn solid(color: Color) -> Self {
        Self {
            background: Some(Fill::Solid(color)),
            border: None,
            radii: CornerRadii::all(0.0),
            opacity: 1.0,
        }
    }

    /// Sets the border.
    #[must_use]
    pub const fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Sets the corner radii.
    #[must_use]
    pub const fn radius(mut self, radii: CornerRadii) -> Self {
        self.radii = radii;
        self
    }

    /// Sets the quad opacity.
    #[must_use]
    pub const fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Returns whether a background or border is present.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.background.is_some() || self.border.is_some()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintStyle {
    pub quad: QuadStyle,
}

impl PaintStyle {
    /// Creates a paint style from a quad style.
    #[must_use]
    pub const fn new(quad: QuadStyle) -> Self {
        Self { quad }
    }

    /// Returns whether the contained quad style is visible.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.quad.is_visible()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Quad {
    pub bounds: Rect,
    pub background: Option<Fill>,
    pub border: Border,
    pub radii: CornerRadii,
    pub opacity: f32,
    pub transform: Affine2D,
    pub clips: ClipChain,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImagePrimitive {
    pub bounds: Rect,
    pub image: ImageId,
    pub fit: ImageFit,
    pub sampling: ImageSampling,
    pub opacity: f32,
    pub radii: CornerRadii,
    pub transform: Affine2D,
    pub clips: ClipChain,
}
