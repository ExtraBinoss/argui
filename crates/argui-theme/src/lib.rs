//! Typed light, dark, and system-resolved themes.

use argui_core::ColorScheme;

mod tokens;
pub use tokens::{ThemeOverrides, ThemeSource, ThemeValue};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme<T> {
    mode: ThemeMode,
    light: T,
    dark: T,
}

impl<T> Theme<T> {
    #[must_use]
    pub const fn new(light: T, dark: T) -> Self {
        Self {
            mode: ThemeMode::System,
            light,
            dark,
        }
    }

    #[must_use]
    pub const fn mode(&self) -> ThemeMode {
        self.mode
    }

    pub const fn set_mode(&mut self, mode: ThemeMode) {
        self.mode = mode;
    }

    #[must_use]
    pub const fn with_mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub const fn resolve(&self, system: ColorScheme) -> &T {
        match self.mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
            ThemeMode::System => match system {
                ColorScheme::Light => &self.light,
                ColorScheme::Dark => &self.dark,
            },
        }
    }
}
