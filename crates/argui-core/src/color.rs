#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color([f32; 4]);

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    #[must_use]
    pub const fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self([red, green, blue, 1.0])
    }

    #[must_use]
    pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self([red, green, blue, alpha])
    }

    #[must_use]
    pub const fn as_array(self) -> [f32; 4] {
        self.0
    }
}
