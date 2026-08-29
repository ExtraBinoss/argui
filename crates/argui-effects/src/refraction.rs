use argui_paint::{Filter, Refraction as PaintRefraction};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Refraction(pub PaintRefraction);

impl Refraction {
    #[must_use]
    pub const fn new(strength: f32) -> Self {
        Self(PaintRefraction::new(strength))
    }
    #[must_use]
    pub const fn chromatic_aberration(mut self, amount: f32) -> Self {
        self.0 = self.0.chromatic_aberration(amount);
        self
    }
    #[must_use]
    pub const fn filter(self) -> Filter {
        Filter::Refraction(self.0)
    }
}
