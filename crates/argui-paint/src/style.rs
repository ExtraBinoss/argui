use argui_core::{Color, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fill {
    Solid(Color),
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ClipBehavior {
    #[default]
    None,
    Bounds,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintStyle {
    pub background: Option<Fill>,
    pub border: Option<Border>,
    pub radii: CornerRadii,
    pub opacity: f32,
    pub clip: ClipBehavior,
}

impl Default for PaintStyle {
    fn default() -> Self {
        Self {
            background: None,
            border: None,
            radii: CornerRadii::default(),
            opacity: 1.0,
            clip: ClipBehavior::None,
        }
    }
}

impl PaintStyle {
    #[must_use]
    pub const fn is_visible(self) -> bool {
        self.background.is_some() || self.border.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub bounds: Rect,
    pub background: Color,
    pub border: Border,
    pub radii: CornerRadii,
    pub opacity: f32,
    pub clip: Rect,
}
