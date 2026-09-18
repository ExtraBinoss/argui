use argui_core::{Color, ColorScheme, Insets};

/// Read-only platform and application state for the window currently being rendered.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowEnvironment {
    pub color_scheme: ColorScheme,
    pub primary: Color,
    pub reduced_motion: bool,
    pub high_contrast: bool,
    /// Native desktop blur is available and permitted for this window.
    pub desktop_backdrop_available: bool,
    /// Safe region supplied by the platform in logical pixels.
    pub safe_area_insets: Insets,
    /// Application-wide accessibility zoom factor, where `1.0` is 100%.
    pub ui_zoom: f32,
    /// Optional theme tokens; no map is allocated in ordinary environments.
    pub theme_overrides: Option<std::sync::Arc<argui_theme::ThemeOverrides>>,
}

impl Default for WindowEnvironment {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Light,
            primary: Color::srgb(0.10, 0.45, 0.91),
            reduced_motion: false,
            high_contrast: false,
            desktop_backdrop_available: false,
            safe_area_insets: Insets::ZERO,
            ui_zoom: 1.0,
            theme_overrides: None,
        }
    }
}

impl argui_theme::ThemeSource for WindowEnvironment {
    fn primary_color(&self) -> Color {
        self.primary
    }
    fn theme_overrides(&self) -> Option<&argui_theme::ThemeOverrides> {
        self.theme_overrides.as_deref()
    }
}

/// Application-selected visual preferences for a window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ThemeRequest {
    pub color_scheme: Option<ColorScheme>,
    pub primary: Option<Color>,
}
