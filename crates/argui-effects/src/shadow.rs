use argui_paint::{Color, Shadow};

#[derive(Clone, Copy, Debug, PartialEq)]
/// Drop-shadow preset wrapping its paint shadow parameters.
pub struct DropShadow(pub Shadow);

impl DropShadow {
    /// Creates a drop shadow with a 2D offset, blur radius, and color.
    #[must_use]
    pub const fn new(offset: [f32; 2], blur: f32, color: Color) -> Self {
        Self(Shadow::drop(offset, blur, color))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Glow preset wrapping its paint shadow parameters.
pub struct Glow(pub Shadow);

impl Glow {
    /// Creates a glow with a blur radius and color.
    #[must_use]
    pub const fn new(blur: f32, color: Color) -> Self {
        Self(Shadow::glow(blur, color))
    }
}
