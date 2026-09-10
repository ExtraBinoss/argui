use super::WidgetGallery;
use argui::{
    core::{Color, ColorScheme},
    runtime::ThemeRequest,
    theme::ThemeMode,
};
use std::sync::LazyLock;

pub(crate) static PRIMARIES: LazyLock<[Color; 6]> = LazyLock::new(|| {
    [
        Color::srgb(0.10, 0.45, 0.91),
        Color::srgb(0.49, 0.23, 0.93),
        Color::srgb(0.86, 0.20, 0.45),
        Color::srgb(0.04, 0.62, 0.48),
        Color::srgb(0.92, 0.42, 0.08),
        Color::srgb(0.20, 0.68, 0.94),
    ]
});

impl WidgetGallery {
    pub(super) fn theme_request(&self) -> ThemeRequest {
        ThemeRequest {
            color_scheme: match self.theme_mode {
                ThemeMode::Light => Some(ColorScheme::Light),
                ThemeMode::Dark => Some(ColorScheme::Dark),
                ThemeMode::System => None,
            },
            primary: Some(PRIMARIES[self.primary]),
        }
    }

    pub(super) fn cycle_theme(&mut self) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
            ThemeMode::System => ThemeMode::Light,
        };
    }
}

pub(super) const fn mode_label(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "Light",
        ThemeMode::Dark => "Dark",
        ThemeMode::System => "System",
    }
}
