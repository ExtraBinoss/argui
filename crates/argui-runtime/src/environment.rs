use argui_core::{Color, ColorScheme};

/// Read-only platform and application state for the window currently being rendered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowEnvironment {
    pub color_scheme: ColorScheme,
    pub primary: Color,
    pub reduced_motion: bool,
    pub high_contrast: bool,
}

impl Default for WindowEnvironment {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Light,
            primary: Color::srgb(0.10, 0.45, 0.91),
            reduced_motion: false,
            high_contrast: false,
        }
    }
}

/// Application-selected visual preferences for a window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ThemeRequest {
    pub color_scheme: Option<ColorScheme>,
    pub primary: Option<Color>,
}
