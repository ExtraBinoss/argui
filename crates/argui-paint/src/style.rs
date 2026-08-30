use argui_core::{Affine2D, Color, Rect};

use crate::{ClipChain, ImageFit, ImageId, ImageSampling, LinearGradient, RadialGradient};

#[derive(Clone, Debug, PartialEq)]
pub enum Fill {
    Solid(Color),
    Linear(LinearGradient),
    Radial(RadialGradient),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    #[must_use]
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    #[must_use]
    pub const fn as_array(self) -> [f32; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }

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
    #[must_use]
    pub const fn all(width: f32) -> Self {
        Self {
            left: width,
            right: width,
            top: width,
            bottom: width,
        }
    }

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
    #[must_use]
    pub const fn solid(color: Color) -> Self {
        Self {
            background: Some(Fill::Solid(color)),
            border: None,
            radii: CornerRadii::all(0.0),
            opacity: 1.0,
        }
    }

    #[must_use]
    pub const fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    #[must_use]
    pub const fn radius(mut self, radii: CornerRadii) -> Self {
        self.radii = radii;
        self
    }

    #[must_use]
    pub const fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

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
    #[must_use]
    pub const fn new(quad: QuadStyle) -> Self {
        Self { quad }
    }

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
