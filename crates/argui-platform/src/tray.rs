use std::collections::HashSet;

use crate::{IconSet, WindowKey};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TrayItemId(String);

impl TrayItemId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrayAction {
    Custom(String),
    ShowWindow(WindowKey),
    HideWindow(WindowKey),
    ToggleWindow(WindowKey),
    FocusWindow(WindowKey),
    CloseWindow(WindowKey),
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TrayMenuItem {
    Action {
        id: TrayItemId,
        label: String,
        enabled: bool,
        action: TrayAction,
    },
    Check {
        id: TrayItemId,
        label: String,
        enabled: bool,
        checked: bool,
        action: TrayAction,
    },
    Separator,
    Submenu {
        id: TrayItemId,
        label: String,
        enabled: bool,
        items: Vec<TrayMenuItem>,
    },
}

impl TrayMenuItem {
    #[must_use]
    pub fn id(&self) -> Option<&TrayItemId> {
        match self {
            Self::Action { id, .. } | Self::Check { id, .. } | Self::Submenu { id, .. } => Some(id),
            Self::Separator => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrayConfig {
    pub tooltip: Option<String>,
    pub title: Option<String>,
    pub icon: Option<IconSet>,
    pub menu: Vec<TrayMenuItem>,
    pub visible: bool,
    pub icon_is_template: bool,
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
pub enum TrayConfigError {
    EmptyItemId,
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
pub enum TrayPointerButton {
    Primary,
    Secondary,
    Middle,
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TrayEvent {
    Action {
        id: TrayItemId,
        action: TrayAction,
    },
    Click {
        button: TrayPointerButton,
        double: bool,
    },
}
