use argui_paint::{Filter, Refraction as PaintRefraction};

#[derive(Clone, Copy, Debug, PartialEq)]
/// Refraction filter preset.
pub struct Refraction(pub PaintRefraction);

impl Refraction {
    /// Creates refraction with the given displacement strength.
    #[must_use]
    pub const fn new(strength: f32) -> Self {
        Self(PaintRefraction::new(strength))
    }
    /// Sets chromatic dispersion amount.
    #[must_use]
    pub const fn chromatic_aberration(mut self, amount: f32) -> Self {
        self.0 = self.0.chromatic_aberration(amount);
        self
    }
    /// Converts this preset into a paint filter.
    #[must_use]
    pub const fn filter(self) -> Filter {
        Filter::Refraction(self.0)
    }
}
