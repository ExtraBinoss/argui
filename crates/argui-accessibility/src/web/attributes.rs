use crate::{LiveRegion, Role, SemanticNode, SemanticValue};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, HtmlInputElement, HtmlTextAreaElement};

pub(super) fn apply_attributes(
    element: &Element,
    node: &SemanticNode,
    id_prefix: &str,
) -> Result<(), JsValue> {
    let relations = &node.semantics.relations;
    for (name, ids) in [
        ("aria-labelledby", &relations.labelled_by),
        ("aria-describedby", &relations.described_by),
        ("aria-controls", &relations.controls),
    ] {
        let value = ids
            .iter()
            .map(|id| format!("{id_prefix}-{}", id.get()))
            .collect::<Vec<_>>()
            .join(" ");
        set_optional(element, name, (!value.is_empty()).then_some(value.as_str()))?;
    }
    let active = relations
        .active_descendant
        .map(|id| format!("{id_prefix}-{}", id.get()));
    set_optional(element, "aria-activedescendant", active.as_deref())?;
    let grid = node.semantics.grid;
    for (name, value) in [
        ("aria-rowcount", grid.row_count),
        ("aria-colcount", grid.column_count),
        ("aria-rowindex", grid.row_index),
        ("aria-colindex", grid.column_index),
    ] {
        set_optional_number(element, name, value.map(f64::from))?;
    }
    set_optional(
        element,
        "aria-sort",
        node.semantics.sort.map(|value| match value {
            crate::SortDirection::Ascending => "ascending",
            crate::SortDirection::Descending => "descending",
        }),
    )?;
    set_optional(
        element,
        "aria-haspopup",
        node.semantics.popup.map(|value| match value {
            crate::PopupKind::Menu => "menu",
            crate::PopupKind::ListBox => "listbox",
            crate::PopupKind::Tree => "tree",
            crate::PopupKind::Grid => "grid",
            crate::PopupKind::Dialog => "dialog",
        }),
    )?;
    element.set_attribute("data-argui-node", &node.id.get().to_string())?;
    element.set_attribute("role", aria_role(node.semantics.role))?;
    element.set_attribute(
        "tabindex",
        if node.semantics.focus_policy.is_tab_stop() && !node.semantics.state.disabled {
            "0"
        } else {
            "-1"
        },
    )?;
    match node.semantics.role {
        Role::TextInput if node.semantics.state.protected => {
            element.set_attribute("type", "password")?
        }
        Role::Button => element.set_attribute("type", "button")?,
        Role::SearchInput => element.set_attribute("type", "search")?,
        _ => element.remove_attribute("type")?,
    }
    set_optional(element, "aria-label", node.semantics.label.as_deref())?;
    set_optional(
        element,
        "aria-description",
        node.semantics.description.as_deref(),
    )?;
    element.set_attribute(
        "aria-disabled",
        if node.semantics.state.disabled {
            "true"
        } else {
            "false"
        },
    )?;
    set_bool(element, "aria-selected", node.semantics.state.selected)?;
    set_optional(
        element,
        "aria-current",
        node.semantics.state.current.map(|current| match current {
            crate::Current::True => "true",
            crate::Current::Page => "page",
            crate::Current::Step => "step",
            crate::Current::Location => "location",
            crate::Current::Date => "date",
            crate::Current::Time => "time",
        }),
    )?;
    set_optional_bool(
        element,
        "aria-multiselectable",
        matches!(
            node.semantics.role,
            Role::ListBox | Role::Grid | Role::Tree | Role::TabList
        )
        .then_some(node.semantics.state.multiselectable),
    )?;
    set_bool(element, "aria-required", node.semantics.state.required)?;
    set_bool(element, "aria-readonly", node.semantics.state.read_only)?;
    set_bool(element, "aria-invalid", node.semantics.state.invalid)?;
    set_bool(element, "aria-busy", node.semantics.state.busy)?;
    set_optional(
        element,
        "aria-modal",
        node.semantics.state.modal.then_some("true"),
    )?;
    set_optional(
        element,
        "aria-checked",
        node.semantics.state.checked.map(|value| match value {
            crate::CheckedState::Unchecked => "false",
            crate::CheckedState::Checked => "true",
            crate::CheckedState::Mixed => "mixed",
        }),
    )?;
    set_optional_bool(element, "aria-pressed", node.semantics.state.pressed)?;
    set_optional_bool(element, "aria-expanded", node.semantics.state.expanded)?;
    for attribute in [
        "aria-valuetext",
        "aria-valuenow",
        "aria-valuemin",
        "aria-valuemax",
    ] {
        element.remove_attribute(attribute)?;
    }
    match &node.semantics.value {
        Some(SemanticValue::Text(value)) => {
            if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                if input.value() != *value {
                    input.set_value(value);
                }
            } else if let Some(textarea) = element.dyn_ref::<HtmlTextAreaElement>() {
                if textarea.value() != *value {
                    textarea.set_value(value);
                }
            } else {
                element.set_attribute("aria-valuetext", value)?;
            }
        }
        Some(SemanticValue::Number {
            value,
            minimum,
            maximum,
            step: _,
        }) => {
            element.set_attribute("aria-valuenow", &value.to_string())?;
            set_optional_number(element, "aria-valuemin", *minimum)?;
            set_optional_number(element, "aria-valuemax", *maximum)?;
        }
        None => {}
    }
    set_optional(
        element,
        "aria-live",
        match node.semantics.live {
            LiveRegion::Off => None,
            LiveRegion::Polite => Some("polite"),
            LiveRegion::Assertive => Some("assertive"),
        },
    )?;
    set_optional(
        element,
        "aria-orientation",
        node.semantics
            .orientation
            .map(|orientation| match orientation {
                crate::Orientation::Horizontal => "horizontal",
                crate::Orientation::Vertical => "vertical",
            }),
    )?;
    set_optional_number(element, "aria-level", node.semantics.level.map(f64::from))?;
    set_optional_number(
        element,
        "aria-posinset",
        node.semantics.position_in_set.map(f64::from),
    )?;
    set_optional_number(
        element,
        "aria-setsize",
        node.semantics.set_size.map(f64::from),
    )?;
    Ok(())
}

pub(super) fn apply_bounds(
    element: &HtmlElement,
    previous: &mut Option<[f64; 4]>,
    bounds: [f64; 4],
) -> Result<(), JsValue> {
    if *previous == Some(bounds) {
        return Ok(());
    }
    let style = element.style();
    for (index, name) in ["left", "top", "width", "height"].iter().enumerate() {
        if previous.is_none_or(|old| old[index] != bounds[index]) {
            style.set_property(name, &format!("{}px", bounds[index]))?;
        }
    }
    *previous = Some(bounds);
    Ok(())
}

fn set_optional(element: &Element, name: &str, value: Option<&str>) -> Result<(), JsValue> {
    if let Some(value) = value {
        element.set_attribute(name, value)
    } else {
        element.remove_attribute(name)
    }
}

fn set_bool(element: &Element, name: &str, value: bool) -> Result<(), JsValue> {
    element.set_attribute(name, if value { "true" } else { "false" })
}

fn set_optional_bool(element: &Element, name: &str, value: Option<bool>) -> Result<(), JsValue> {
    set_optional(
        element,
        name,
        value.map(|value| if value { "true" } else { "false" }),
    )
}

fn set_optional_number(element: &Element, name: &str, value: Option<f64>) -> Result<(), JsValue> {
    if let Some(value) = value {
        element.set_attribute(name, &value.to_string())
    } else {
        element.remove_attribute(name)
    }
}

pub(super) const fn html_tag(role: Role) -> &'static str {
    match role {
        Role::Button => "button",
        Role::TextInput | Role::SearchInput => "input",
        Role::TextArea => "textarea",
        Role::Link => "a",
        Role::List => "ul",
        Role::ListItem => "li",
        Role::Heading => "h2",
        Role::Text => "span",
        _ => "div",
    }
}

const fn aria_role(role: Role) -> &'static str {
    match role {
        Role::Generic => "presentation",
        Role::Window => "application",
        Role::Group => "group",
        Role::Navigation => "navigation",
        Role::Text => "text",
        Role::Heading => "heading",
        Role::Image => "img",
        Role::Link => "link",
        Role::Button => "button",
        Role::CheckBox => "checkbox",
        Role::RadioButton => "radio",
        Role::Switch => "switch",
        Role::TextInput => "textbox",
        Role::TextArea => "textbox",
        Role::SearchInput => "searchbox",
        Role::Table => "table",
        Role::Grid => "grid",
        Role::Row => "row",
        Role::ColumnHeader => "columnheader",
        Role::Cell => "cell",
        Role::List => "list",
        Role::ListItem => "listitem",
        Role::ListBox => "listbox",
        Role::Option => "option",
        Role::Menu => "menu",
        Role::MenuBar => "menubar",
        Role::MenuItemCheckBox => "menuitemcheckbox",
        Role::MenuItemRadio => "menuitemradio",
        Role::ComboBox => "combobox",
        Role::Tooltip => "tooltip",
        Role::Status => "status",
        Role::AlertDialog => "alertdialog",
        Role::MenuItem => "menuitem",
        Role::Slider => "slider",
        Role::Progress => "progressbar",
        Role::Tab => "tab",
        Role::TabList => "tablist",
        Role::TabPanel => "tabpanel",
        Role::Dialog => "dialog",
        Role::Alert => "alert",
        Role::Separator => "separator",
        Role::Tree => "tree",
        Role::TreeItem => "treeitem",
    }
}
