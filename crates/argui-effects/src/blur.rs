use argui_paint::Filter;

#[derive(Clone, Copy, Debug, PartialEq)]
/// Gaussian blur preset whose value is the blur radius in logical pixels.
pub struct Blur(pub f32);

impl Blur {
    /// Converts this preset into a paint filter.
    #[must_use]
    pub const fn filter(self) -> Filter {
        Filter::Blur(self.0)
    }
}
