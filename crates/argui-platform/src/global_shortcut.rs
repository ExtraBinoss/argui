use std::collections::HashSet;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// Stable application-local identifier for a global keyboard shortcut.
pub struct GlobalShortcutId(String);

impl GlobalShortcutId {
    /// Creates an identifier for a global shortcut.
    ///
    /// `value` is compared with configured shortcut IDs when native events arrive.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns this shortcut identifier's string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A global keyboard shortcut requested by an application.
pub struct GlobalShortcut {
    /// Stable identifier delivered with activation events.
    pub id: GlobalShortcutId,
    /// Portable accelerator such as `CmdOrCtrl+Space` or `Shift+Alt+KeyD`.
    pub accelerator: String,
}

impl GlobalShortcut {
    /// Creates a shortcut request from an application ID and portable accelerator.
    ///
    /// `id` identifies later events. `accelerator` lists modifiers before one physical key.
    #[must_use]
    pub fn new(id: impl Into<String>, accelerator: impl Into<String>) -> Self {
        Self {
            id: GlobalShortcutId::new(id),
            accelerator: accelerator.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// State transition reported for a registered global shortcut.
pub enum GlobalShortcutState {
    /// The shortcut's main key became pressed.
    Pressed,
    /// The shortcut's main key became released.
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Event emitted when a registered shortcut changes state outside the application.
pub struct GlobalShortcutEvent {
    /// Stable ID from the matching [`GlobalShortcut`].
    pub id: GlobalShortcutId,
    /// Whether the shortcut was pressed or released.
    pub state: GlobalShortcutState,
    /// Compositor-provided token for activating a window in response to this event.
    ///
    /// This is populated by the Wayland global-shortcuts portal for activation
    /// events and is `None` on platforms that do not require such a token.
    pub activation_token: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Validation failure in an application's global shortcuts.
pub enum GlobalShortcutConfigError {
    /// A shortcut has an empty application-defined ID.
    EmptyId,
    /// Two shortcuts use the same application-defined ID.
    DuplicateId(GlobalShortcutId),
    /// A shortcut has an empty accelerator.
    EmptyAccelerator(GlobalShortcutId),
    /// Two shortcuts use the same accelerator spelling.
    DuplicateAccelerator(String),
}

impl std::fmt::Display for GlobalShortcutConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyId => formatter.write_str("global shortcut ids cannot be empty"),
            Self::DuplicateId(id) => {
                write!(formatter, "duplicate global shortcut id: {}", id.as_str())
            }
            Self::EmptyAccelerator(id) => write!(
                formatter,
                "global shortcut '{}' has an empty accelerator",
                id.as_str()
            ),
            Self::DuplicateAccelerator(accelerator) => {
                write!(formatter, "duplicate global shortcut: {accelerator}")
            }
        }
    }
}

impl std::error::Error for GlobalShortcutConfigError {}

/// Validates shortcut IDs and accelerator spellings before native registration.
///
/// `shortcuts` contains the complete application shortcut set.
///
/// # Errors
/// Returns an error for empty or duplicate IDs and accelerators.
pub(crate) fn validate_global_shortcuts(
    shortcuts: &[GlobalShortcut],
) -> Result<(), GlobalShortcutConfigError> {
    let mut ids = HashSet::new();
    let mut accelerators = HashSet::new();
    for shortcut in shortcuts {
        if shortcut.id.as_str().is_empty() {
            return Err(GlobalShortcutConfigError::EmptyId);
        }
        if !ids.insert(shortcut.id.clone()) {
            return Err(GlobalShortcutConfigError::DuplicateId(shortcut.id.clone()));
        }
        let accelerator = shortcut.accelerator.trim();
        if accelerator.is_empty() {
            return Err(GlobalShortcutConfigError::EmptyAccelerator(
                shortcut.id.clone(),
            ));
        }
        let normalized = accelerator.to_ascii_lowercase();
        if !accelerators.insert(normalized) {
            return Err(GlobalShortcutConfigError::DuplicateAccelerator(
                shortcut.accelerator.clone(),
            ));
        }
    }
    Ok(())
}
