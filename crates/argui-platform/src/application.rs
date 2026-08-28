use std::collections::HashSet;

use crate::{ApplicationIdentity, TrayConfig, WindowKey, WindowSpec};

#[derive(Clone, Debug, PartialEq)]
pub struct ApplicationConfig {
    pub identity: ApplicationIdentity,
    pub windows: Vec<WindowSpec>,
    pub tray: Option<TrayConfig>,
}

impl ApplicationConfig {
    #[must_use]
    pub fn new(identity: ApplicationIdentity, main_window: crate::WindowConfig) -> Self {
        Self {
            identity,
            windows: vec![WindowSpec::new(WindowKey::main(), main_window)],
            tray: None,
        }
    }

    #[must_use]
    pub fn with_window(mut self, window: WindowSpec) -> Self {
        self.windows.push(window);
        self
    }

    #[must_use]
    pub fn with_tray(mut self, tray: TrayConfig) -> Self {
        self.tray = Some(tray);
        self
    }

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
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationConfigError {
    EmptyWindowKey,
    DuplicateWindowKey(WindowKey),
    InvalidTray(String),
}

impl std::fmt::Display for ApplicationConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyWindowKey => formatter.write_str("window keys cannot be empty"),
            Self::DuplicateWindowKey(key) => {
                write!(formatter, "duplicate window key: {}", key.as_str())
            }
            Self::InvalidTray(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for ApplicationConfigError {}
