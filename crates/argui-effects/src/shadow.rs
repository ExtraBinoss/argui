use argui_paint::{Color, Shadow};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropShadow(pub Shadow);

impl DropShadow {
    #[must_use]
    pub const fn new(offset: [f32; 2], blur: f32, color: Color) -> Self {
        Self(Shadow::drop(offset, blur, color))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow(pub Shadow);

impl Glow {
    #[must_use]
    pub const fn new(blur: f32, color: Color) -> Self {
        Self(Shadow::glow(blur, color))
    }
}
