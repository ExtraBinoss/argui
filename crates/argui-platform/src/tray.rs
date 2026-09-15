use std::collections::HashSet;

use crate::{IconSet, WindowKey};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// Stable identifier for a tray menu item.
pub struct TrayItemId(String);

impl TrayItemId {
    /// Creates a stable identifier for a tray item.
    /// `value` is the identifier used to match later menu events.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the identifier's string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Command dispatched when a user activates a tray item.
pub enum TrayAction {
    /// Deliver an application-defined command string.
    Custom(String),
    /// Show the identified window.
    ShowWindow(WindowKey),
    /// Hide the identified window.
    HideWindow(WindowKey),
    /// Toggle visibility of the identified window.
    ToggleWindow(WindowKey),
    /// Focus the identified window.
    FocusWindow(WindowKey),
    /// Close the identified window.
    CloseWindow(WindowKey),
    /// Quit the application.
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
/// Action, checkable item, separator, or nested menu in the system tray.
pub enum TrayMenuItem {
    /// A menu item that dispatches an action.
    Action {
        /// Stable identifier for this item.
        id: TrayItemId,
        /// User-visible label.
        label: String,
        /// Whether the item can be activated.
        enabled: bool,
        /// Command dispatched on activation.
        action: TrayAction,
    },
    /// A checkable menu item.
    Check {
        /// Stable identifier for this item.
        id: TrayItemId,
        /// User-visible label.
        label: String,
        /// Whether the item can be activated.
        enabled: bool,
        /// Whether the item is currently checked.
        checked: bool,
        /// Command dispatched on activation.
        action: TrayAction,
    },
    /// Visual separator between menu groups.
    Separator,
    /// Nested menu with child entries.
    Submenu {
        /// Stable identifier for this submenu.
        id: TrayItemId,
        /// User-visible label.
        label: String,
        /// Whether the submenu can be opened.
        enabled: bool,
        /// Child entries in display order.
        items: Vec<TrayMenuItem>,
    },
}

impl TrayMenuItem {
    /// Returns the ID of an actionable item or submenu; separators have no ID.
    #[must_use]
    pub fn id(&self) -> Option<&TrayItemId> {
        match self {
            Self::Action { id, .. } | Self::Check { id, .. } | Self::Submenu { id, .. } => Some(id),
            Self::Separator => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Desired visibility, icon, and menu for the system tray.
pub struct TrayConfig {
    /// Optional tooltip shown by the platform.
    pub tooltip: Option<String>,
    /// Optional title shown alongside the tray icon.
    pub title: Option<String>,
    /// Icon variants for the tray, when supplied.
    pub icon: Option<IconSet>,
    /// Menu entries in display order.
    pub menu: Vec<TrayMenuItem>,
    /// Whether the tray icon is visible.
    pub visible: bool,
    /// Whether the icon is a template image on platforms that support it.
    pub icon_is_template: bool,
    /// Whether a primary-button click opens the menu.
    pub show_menu_on_left_click: bool,
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            tooltip: None,
            title: None,
            icon: None,
            menu: Vec::new(),
            visible: true,
            icon_is_template: false,
            show_menu_on_left_click: true,
        }
    }
}

impl TrayConfig {
    /// Checks that all menu item IDs are non-empty and unique, including submenus.
    ///
    /// # Errors
    /// Returns an error if any item ID is empty or duplicated.
    pub fn validate(&self) -> Result<(), TrayConfigError> {
        let mut ids = HashSet::new();
        validate_items(&self.menu, &mut ids)
    }
}

fn validate_items(
    items: &[TrayMenuItem],
    ids: &mut HashSet<TrayItemId>,
) -> Result<(), TrayConfigError> {
    for item in items {
        let Some(id) = item.id() else {
            continue;
        };
        if id.as_str().is_empty() {
            return Err(TrayConfigError::EmptyItemId);
        }
        if !ids.insert(id.clone()) {
            return Err(TrayConfigError::DuplicateItemId(id.clone()));
        }
        if let TrayMenuItem::Submenu { items, .. } = item {
            validate_items(items, ids)?;
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Validation failure in a tray menu.
pub enum TrayConfigError {
    /// A menu item has an empty ID.
    EmptyItemId,
    /// Two menu items share an ID.
    DuplicateItemId(TrayItemId),
}

impl std::fmt::Display for TrayConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyItemId => formatter.write_str("tray item ids cannot be empty"),
            Self::DuplicateItemId(id) => {
                write!(formatter, "duplicate tray item id: {}", id.as_str())
            }
        }
    }
}

impl std::error::Error for TrayConfigError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Pointer button reported by a tray icon.
pub enum TrayPointerButton {
    /// Primary pointer button.
    Primary,
    /// Secondary pointer button.
    Secondary,
    /// Middle pointer button.
    Middle,
    /// Button not classified by the platform.
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
/// User action emitted by a native tray integration.
pub enum TrayEvent {
    /// A menu command was activated.
    Action {
        /// Identifier of the activated item.
        id: TrayItemId,
        /// Command associated with the item.
        action: TrayAction,
    },
    /// The tray icon itself was clicked.
    Click {
        /// Pointer button reported by the platform.
        button: TrayPointerButton,
        /// Whether this was a double click.
        double: bool,
    },
}
