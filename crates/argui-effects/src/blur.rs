use argui_paint::Filter;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blur(pub f32);

impl Blur {
    #[must_use]
    pub const fn filter(self) -> Filter {
        Filter::Blur(self.0)
    }
}
