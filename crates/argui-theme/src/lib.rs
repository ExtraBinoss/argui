//! Typed light, dark, and system-resolved themes.

use argui_core::ColorScheme;

mod runtime;
mod schema;
mod tokens;
mod value;
pub use runtime::{ThemeChange, ThemeRuntime};
pub use schema::{ThemeError, ThemeImpact, ThemeSchema, ThemeTokenDefinition, ThemeTokenId};
pub use tokens::{ThemeOverrides, ThemeSource, ThemeValue};
pub use value::{ThemeDimension, ThemeValueType};

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
    /// Creates a theme with light and dark values, initially following the system scheme.
    #[must_use]
    pub const fn new(light: T, dark: T) -> Self {
        Self {
            mode: ThemeMode::System,
            light,
            dark,
        }
    }

    #[must_use]
    /// Returns whether this theme follows the system scheme or uses an explicit mode.
    pub const fn mode(&self) -> ThemeMode {
        self.mode
    }

    /// Sets whether this theme follows the system scheme or uses an explicit mode.
    pub const fn set_mode(&mut self, mode: ThemeMode) {
        self.mode = mode;
    }

    #[must_use]
    /// Sets the mode and returns the updated theme.
    pub const fn with_mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    /// Resolves the active value using this theme's mode and the system color scheme.
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
