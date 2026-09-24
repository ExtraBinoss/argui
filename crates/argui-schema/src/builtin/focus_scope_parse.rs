//! Parsing for the shared declarative accessibility and focus vocabulary.

use argui_ui::{Current, FocusContainment, KeyboardActivation, LiveRegion, Role};

use crate::SchemaError;

/// Parses a focus containment policy from its declarative spelling.
///
/// * `name` — policy name supplied by the author.
///
/// # Errors
///
/// Returns an adapter error for unknown policies.
pub(super) fn parse_containment(name: &str) -> Result<FocusContainment, SchemaError> {
    match name {
        "none" => Ok(FocusContainment::None),
        "trap" => Ok(FocusContainment::Trap),
        "modal" => Ok(FocusContainment::Modal),
        _ => Err(SchemaError::Adapter(format!(
            "FocusScope does not support containment `{name}`"
        ))),
    }
}

/// Parses generic keyboard activation without choosing a control style.
///
/// * `name` — activation policy name supplied by the author.
///
/// # Errors
///
/// Returns an adapter error for unknown policies.
pub(super) fn parse_activation(name: &str) -> Result<KeyboardActivation, SchemaError> {
    match name {
        "none" => Ok(KeyboardActivation::None),
        "enter" => Ok(KeyboardActivation::Enter),
        "enter_or_space" => Ok(KeyboardActivation::EnterOrSpace),
        _ => Err(SchemaError::Adapter(format!(
            "unsupported keyboard_activation `{name}`"
        ))),
    }
}

/// Parses the current-item category announced by accessibility adapters.
///
/// * `name` — canonical current-item spelling.
///
/// # Errors
///
/// Returns an adapter error for an unsupported category.
pub(super) fn parse_current(name: &str) -> Result<Current, SchemaError> {
    match name {
        "true" => Ok(Current::True),
        "page" => Ok(Current::Page),
        "step" => Ok(Current::Step),
        "location" => Ok(Current::Location),
        "date" => Ok(Current::Date),
        "time" => Ok(Current::Time),
        _ => Err(SchemaError::Adapter(format!(
            "unsupported current `{name}`"
        ))),
    }
}

/// Parses a live-region announcement policy.
///
/// * `name` — canonical live-region spelling.
///
/// # Errors
///
/// Returns an adapter error for an unsupported policy.
pub(super) fn parse_live(name: &str) -> Result<LiveRegion, SchemaError> {
    match name {
        "off" => Ok(LiveRegion::Off),
        "polite" => Ok(LiveRegion::Polite),
        "assertive" => Ok(LiveRegion::Assertive),
        _ => Err(SchemaError::Adapter(format!("unsupported live `{name}`"))),
    }
}

/// Parses an accessible role supported by the engine.
///
/// * `name` — role spelling supplied by the author.
///
/// # Errors
///
/// Returns an adapter error for unknown roles.
pub(super) fn parse_role(name: &str) -> Result<Role, SchemaError> {
    let role = match name {
        "generic" => Role::Generic,
        "window" => Role::Window,
        "group" => Role::Group,
        "navigation" => Role::Navigation,
        "text" => Role::Text,
        "heading" => Role::Heading,
        "image" => Role::Image,
        "link" => Role::Link,
        "button" => Role::Button,
        "check_box" => Role::CheckBox,
        "radio_button" => Role::RadioButton,
        "switch" => Role::Switch,
        "text_input" => Role::TextInput,
        "text_area" => Role::TextArea,
        "search_input" => Role::SearchInput,
        "table" => Role::Table,
        "grid" => Role::Grid,
        "row" => Role::Row,
        "column_header" => Role::ColumnHeader,
        "cell" => Role::Cell,
        "list" => Role::List,
        "list_item" => Role::ListItem,
        "tree" => Role::Tree,
        "tree_item" => Role::TreeItem,
        "list_box" => Role::ListBox,
        "option" => Role::Option,
        "menu" => Role::Menu,
        "menu_item" => Role::MenuItem,
        "menu_bar" => Role::MenuBar,
        "menu_item_check_box" => Role::MenuItemCheckBox,
        "menu_item_radio" => Role::MenuItemRadio,
        "combo_box" => Role::ComboBox,
        "tooltip" => Role::Tooltip,
        "status" => Role::Status,
        "alert_dialog" => Role::AlertDialog,
        "slider" => Role::Slider,
        "progress" => Role::Progress,
        "tab" => Role::Tab,
        "tab_list" => Role::TabList,
        "tab_panel" => Role::TabPanel,
        "dialog" => Role::Dialog,
        "alert" => Role::Alert,
        "separator" => Role::Separator,
        _ => {
            return Err(SchemaError::Adapter(format!(
                "unsupported accessible role `{name}`"
            )));
        }
    };
    Ok(role)
}
