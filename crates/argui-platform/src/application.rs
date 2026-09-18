use std::collections::HashSet;

use crate::{
    ApplicationIdentity, GlobalShortcut, PreferenceOverrides, TrayConfig, WindowKey, WindowSpec,
    global_shortcut::validate_global_shortcuts,
};

#[derive(Clone, Debug, PartialEq)]
/// Validated application-wide platform configuration.
pub struct ApplicationConfig {
    /// Identity metadata for windows and platform integrations.
    pub identity: ApplicationIdentity,
    /// Configured application windows, including the main window.
    pub windows: Vec<WindowSpec>,
    /// Optional system-tray configuration.
    pub tray: Option<TrayConfig>,
    /// Global keyboard shortcuts requested for the application lifetime.
    pub global_shortcuts: Vec<GlobalShortcut>,
    /// Explicit overrides for operating-system preferences.
    pub preferences: PreferenceOverrides,
}

impl ApplicationConfig {
    /// Creates an application configuration with a single main window.
    /// `identity` supplies app metadata; `main_window` configures that window.
    #[must_use]
    pub fn new(identity: ApplicationIdentity, main_window: crate::WindowConfig) -> Self {
        Self {
            identity,
            windows: vec![WindowSpec::new(WindowKey::main(), main_window)],
            tray: None,
            global_shortcuts: Vec::new(),
            preferences: PreferenceOverrides::default(),
        }
    }

    #[must_use]
    /// Adds a secondary window to this application.
    /// `window` is appended to the configured window list.
    pub fn with_window(mut self, window: WindowSpec) -> Self {
        self.windows.push(window);
        self
    }

    #[must_use]
    /// Configures the application's system-tray menu.
    /// `tray` is the desired system-tray configuration.
    pub fn with_tray(mut self, tray: TrayConfig) -> Self {
        self.tray = Some(tray);
        self
    }

    /// Adds a global keyboard shortcut requested for the application lifetime.
    ///
    /// `shortcut` is registered when the native event loop starts.
    #[must_use]
    pub fn with_global_shortcut(mut self, shortcut: GlobalShortcut) -> Self {
        self.global_shortcuts.push(shortcut);
        self
    }

    #[must_use]
    /// Sets overrides for preferences detected from the operating system.
    /// `preferences` contains optional explicit settings.
    pub const fn with_preferences(mut self, preferences: PreferenceOverrides) -> Self {
        self.preferences = preferences;
        self
    }

    /// Checks that window keys and global shortcuts are unique and the tray is valid.
    ///
    /// # Errors
    /// Returns an error for an empty or duplicate window key, global shortcut, or invalid tray.
    pub fn validate(&self) -> Result<(), ApplicationConfigError> {
        let mut keys = HashSet::new();
        for window in &self.windows {
            if window.key.as_str().is_empty() {
                return Err(ApplicationConfigError::EmptyWindowKey);
            }
            if !keys.insert(window.key.clone()) {
                return Err(ApplicationConfigError::DuplicateWindowKey(
                    window.key.clone(),
                ));
            }
        }
        if let Some(tray) = &self.tray {
            tray.validate()
                .map_err(|error| ApplicationConfigError::InvalidTray(error.to_string()))?;
        }
        validate_global_shortcuts(&self.global_shortcuts)
            .map_err(|error| ApplicationConfigError::InvalidGlobalShortcut(error.to_string()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Error returned by [`ApplicationConfig::validate`].
pub enum ApplicationConfigError {
    /// A window has an empty key.
    EmptyWindowKey,
    /// Two windows use the same key.
    DuplicateWindowKey(WindowKey),
    /// The tray configuration failed validation.
    InvalidTray(String),
    /// The global shortcut configuration failed validation.
    InvalidGlobalShortcut(String),
}

impl std::fmt::Display for ApplicationConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyWindowKey => formatter.write_str("window keys cannot be empty"),
            Self::DuplicateWindowKey(key) => {
                write!(formatter, "duplicate window key: {}", key.as_str())
            }
            Self::InvalidTray(error) => formatter.write_str(error),
            Self::InvalidGlobalShortcut(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for ApplicationConfigError {}
