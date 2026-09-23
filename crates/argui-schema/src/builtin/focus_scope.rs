//! Unpainted focus and keyboard boundary for declarative controls.

use argui_ui::{
    CheckedState, Current, Element, EventType, FocusContainment, FocusPolicy, FocusScope,
    FocusTarget, InitialFocus, Interaction, KeyboardActivation, LiveRegion, Role, SemanticAction,
    SemanticState, SemanticValue, Semantics, UserSelect,
};

use super::{
    BLUR, BUSY, CAPTURE_KEY_INPUT, CHECKED, CHILDREN, CLICK, CommonProperty, ENABLED, EXPANDED,
    FOCUS, FOCUS_CONTAINMENT, FOCUS_ON_CLICK, FOCUS_ON_TAB, FOCUS_VISIBLE, HAS_FOCUS,
    INITIAL_FOCUS, KEY_INPUT, KEYBOARD_ACTIVATION, RESTORE_FOCUS, SEMANTIC_ACTIVE_DESCENDANT,
    SEMANTIC_CONTROLS, SEMANTIC_CURRENT, SEMANTIC_DESCRIBED_BY, SEMANTIC_DESCRIPTION,
    SEMANTIC_EXPANDABLE, SEMANTIC_INVALID, SEMANTIC_LABEL, SEMANTIC_LABELLED_BY, SEMANTIC_LIVE,
    SEMANTIC_ROLE, SEMANTIC_SELECTED, SEMANTIC_VALUE, apply_common, common_event, common_property,
    optional_bool,
};
use crate::{
    NativeElementInput, NativeSchema, ObservationKind, PropertySchema, SchemaError, SchemaRegistry,
    SchemaValue, SlotArity, SlotSchema, ValueType,
};

/// Registers a generic focus scope with keyboard activation and observable focus.
///
/// * `registry` — native registry receiving the scope metadata and adapter.
///
/// # Errors
///
/// Returns a schema error when built-in identifiers or metadata conflict.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        super::FOCUS_SCOPE,
        "FocusScope",
        "Unpainted focus boundary with optional keyboard activation and accessibility semantics.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Tooltip))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::X))
    .property(common_property(CommonProperty::Y))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::BackdropFilter))
    .property(common_property(CommonProperty::Visible))
    .property(
        PropertySchema::new(
            ENABLED,
            "enabled",
            ValueType::Bool,
            "Whether this scope can focus.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(
        PropertySchema::new(
            FOCUS_ON_CLICK,
            "focus_on_click",
            ValueType::Bool,
            "Focus this scope when a nonfocusable descendant is pressed.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(
        PropertySchema::new(
            FOCUS_ON_TAB,
            "focus_on_tab_navigation",
            ValueType::Bool,
            "Include this scope in sequential keyboard focus.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(
        PropertySchema::new(
            HAS_FOCUS,
            "has_focus",
            ValueType::Bool,
            "Scope owns keyboard focus.",
        )
        .observed(ObservationKind::Focused),
    )
    .property(
        PropertySchema::new(
            FOCUS_VISIBLE,
            "focus_visible",
            ValueType::Bool,
            "Keyboard focus indication is active.",
        )
        .observed(ObservationKind::FocusVisible),
    )
    .property(PropertySchema::new(
        FOCUS_CONTAINMENT,
        "containment",
        ValueType::String,
        "Focus containment: none, trap, or modal.",
    ))
    .property(PropertySchema::new(
        RESTORE_FOCUS,
        "restore_focus",
        ValueType::Bool,
        "Restore prior focus when the scope disappears.",
    ))
    .property(PropertySchema::new(
        INITIAL_FOCUS,
        "initial_focus",
        ValueType::String,
        "Initial child key, or first for the first focusable child.",
    ))
    .property(PropertySchema::new(
        KEYBOARD_ACTIVATION,
        "keyboard_activation",
        ValueType::String,
        "Keys generating click: none, enter, or enter_or_space.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_ROLE,
        "role",
        ValueType::String,
        "Accessible role for a composed control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_LABEL,
        "accessible_name",
        ValueType::String,
        "Accessible name for a composed control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_DESCRIPTION,
        "accessible_description",
        ValueType::String,
        "Accessible description for a composed control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_VALUE,
        "accessible_value",
        ValueType::String,
        "Current text value announced by assistive technology.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_SELECTED,
        "selected",
        ValueType::Bool,
        "Whether this item is selected in its set.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CURRENT,
        "current",
        ValueType::String,
        "Current item kind: true, page, step, location, date, or time.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_CONTROLS,
        "controls",
        ValueType::String,
        "Key of the element controlled by this control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_ACTIVE_DESCENDANT,
        "active_descendant",
        ValueType::String,
        "Key of the active child in a composite control.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_LABELLED_BY,
        "labelled_by",
        ValueType::String,
        "Key of the element providing this control's accessible name.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_DESCRIBED_BY,
        "described_by",
        ValueType::String,
        "Key of the element providing this control's description.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_INVALID,
        "invalid",
        ValueType::Bool,
        "Whether the composed control has an invalid value.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_LIVE,
        "live",
        ValueType::String,
        "Announcement policy: off, polite, or assertive.",
    ))
    .property(PropertySchema::new(
        BUSY,
        "busy",
        ValueType::Bool,
        "Whether the composed control is busy.",
    ))
    .property(PropertySchema::new(
        CHECKED,
        "checked",
        ValueType::Bool,
        "Checked state for accessible toggle roles.",
    ))
    .property(PropertySchema::new(
        SEMANTIC_EXPANDABLE,
        "expandable",
        ValueType::Bool,
        "Whether expanded state applies to this control.",
    ))
    .property(PropertySchema::new(
        EXPANDED,
        "expanded",
        ValueType::Bool,
        "Current expanded state when expandable is true.",
    ))
    .event(common_event(CLICK, "click", EventType::Click))
    .event(common_event(FOCUS, "focus", EventType::Focus))
    .event(common_event(BLUR, "blur", EventType::Blur))
    .event(common_event(KEY_INPUT, "key", EventType::Key))
    .event(common_event(
        CAPTURE_KEY_INPUT,
        "capture_key",
        EventType::Key,
    ))
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Visual and interaction descendants in this focus boundary.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let enabled = optional_bool(input, ENABLED).unwrap_or(true);
        let focus_policy = if !enabled {
            FocusPolicy::None
        } else if optional_bool(input, FOCUS_ON_TAB).unwrap_or(true) {
            FocusPolicy::TabStop
        } else {
            FocusPolicy::Programmatic
        };
        let activation = match input.get(KEYBOARD_ACTIVATION) {
            Some(SchemaValue::String(value)) => parse_activation(value)?,
            _ => KeyboardActivation::None,
        };
        let containment = match input.get(FOCUS_CONTAINMENT) {
            Some(SchemaValue::String(value)) => parse_containment(value)?,
            _ => FocusContainment::None,
        };
        let initial = match input.get(INITIAL_FOCUS) {
            Some(SchemaValue::String(value)) if value == "first" => Some(InitialFocus::First),
            Some(SchemaValue::String(value)) if value.is_empty() => None,
            Some(SchemaValue::String(value)) => {
                Some(InitialFocus::Target(FocusTarget::Key(value.clone())))
            }
            _ => None,
        };
        let focus_scope = FocusScope {
            containment,
            initial,
            restore: optional_bool(input, RESTORE_FOCUS).unwrap_or(true),
        };
        let interaction = Interaction::default()
            .enabled(enabled)
            .focus_policy(focus_policy)
            .focus_on_descendant_press(optional_bool(input, FOCUS_ON_CLICK).unwrap_or(true))
            .keyboard_activation(activation);
        let mut element =
            apply_common(Element::container(input.children(CHILDREN).to_vec()), input)?
                .focus_scope(focus_scope)
                .interaction(interaction);
        if activation != KeyboardActivation::None
            || input.events.iter().any(|event| event.id == CLICK)
        {
            element = element.user_select(UserSelect::None);
        }
        let has_semantics = [
            SEMANTIC_ROLE,
            SEMANTIC_LABEL,
            SEMANTIC_DESCRIPTION,
            SEMANTIC_VALUE,
            SEMANTIC_SELECTED,
            SEMANTIC_CURRENT,
            SEMANTIC_CONTROLS,
            SEMANTIC_ACTIVE_DESCENDANT,
            SEMANTIC_LABELLED_BY,
            SEMANTIC_DESCRIBED_BY,
            SEMANTIC_INVALID,
            SEMANTIC_LIVE,
        ]
        .iter()
        .any(|id| input.get(*id).is_some());
        if has_semantics {
            let role = match input.get(SEMANTIC_ROLE) {
                Some(SchemaValue::String(name)) => parse_role(name)?,
                _ => Role::Generic,
            };
            let mut semantics = Semantics::new(role)
                .state(SemanticState {
                    disabled: !enabled,
                    busy: optional_bool(input, BUSY).unwrap_or(false),
                    selected: optional_bool(input, SEMANTIC_SELECTED).unwrap_or(false),
                    invalid: optional_bool(input, SEMANTIC_INVALID).unwrap_or(false),
                    current: match input.get(SEMANTIC_CURRENT) {
                        Some(SchemaValue::String(value)) => Some(parse_current(value)?),
                        _ => None,
                    },
                    checked: optional_bool(input, CHECKED).map(|value| {
                        if value {
                            CheckedState::Checked
                        } else {
                            CheckedState::Unchecked
                        }
                    }),
                    expanded: optional_bool(input, SEMANTIC_EXPANDABLE)
                        .filter(|value| *value)
                        .map(|_| optional_bool(input, EXPANDED).unwrap_or(false)),
                    ..SemanticState::default()
                })
                .action(SemanticAction::Focus);
            if let Some(SchemaValue::String(name)) = input.get(SEMANTIC_LABEL) {
                semantics = semantics.label(name.clone());
            }
            if let Some(SchemaValue::String(description)) = input.get(SEMANTIC_DESCRIPTION) {
                semantics = semantics.description(description.clone());
            }
            if let Some(SchemaValue::String(value)) = input.get(SEMANTIC_VALUE) {
                semantics = semantics.value(SemanticValue::Text(value.clone()));
            }
            if let Some(SchemaValue::String(value)) = input.get(SEMANTIC_LIVE) {
                semantics = semantics.live(parse_live(value)?);
            }
            if activation != KeyboardActivation::None {
                semantics = semantics.action(SemanticAction::Click);
            }
            element = element.semantics(semantics);
            if let Some(SchemaValue::String(key)) = input.get(SEMANTIC_CONTROLS) {
                element = element.controls([key.as_str()]);
            }
            if let Some(SchemaValue::String(key)) = input.get(SEMANTIC_ACTIVE_DESCENDANT) {
                element = element.active_descendant(key.as_str());
            }
            if let Some(SchemaValue::String(key)) = input.get(SEMANTIC_LABELLED_BY) {
                element = element.labelled_by([key.as_str()]);
            }
            if let Some(SchemaValue::String(key)) = input.get(SEMANTIC_DESCRIBED_BY) {
                element = element.described_by([key.as_str()]);
            }
        }
        for event in &input.events {
            let listener = match event.id {
                CLICK => event.handler.listener(EventType::Click),
                FOCUS => event.handler.direct_listener(EventType::Focus),
                BLUR => event.handler.direct_listener(EventType::Blur),
                KEY_INPUT => event.handler.listener(EventType::Key),
                CAPTURE_KEY_INPUT => event.handler.listener(EventType::Key).capture(true),
                _ => continue,
            };
            element = element.on(listener);
        }
        Ok(element)
    })
}

/// Parses a focus containment policy from its declarative spelling.
///
/// * `name` — policy name supplied by the author.
///
/// # Errors
///
/// Returns an adapter error for unknown policies.
fn parse_containment(name: &str) -> Result<FocusContainment, SchemaError> {
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
fn parse_activation(name: &str) -> Result<KeyboardActivation, SchemaError> {
    match name {
        "none" => Ok(KeyboardActivation::None),
        "enter" => Ok(KeyboardActivation::Enter),
        "enter_or_space" => Ok(KeyboardActivation::EnterOrSpace),
        _ => Err(SchemaError::Adapter(format!(
            "FocusScope does not support keyboard_activation `{name}`"
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
fn parse_current(name: &str) -> Result<Current, SchemaError> {
    match name {
        "true" => Ok(Current::True),
        "page" => Ok(Current::Page),
        "step" => Ok(Current::Step),
        "location" => Ok(Current::Location),
        "date" => Ok(Current::Date),
        "time" => Ok(Current::Time),
        _ => Err(SchemaError::Adapter(format!(
            "FocusScope does not support current `{name}`"
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
fn parse_live(name: &str) -> Result<LiveRegion, SchemaError> {
    match name {
        "off" => Ok(LiveRegion::Off),
        "polite" => Ok(LiveRegion::Polite),
        "assertive" => Ok(LiveRegion::Assertive),
        _ => Err(SchemaError::Adapter(format!(
            "FocusScope does not support live `{name}`"
        ))),
    }
}

/// Parses an accessible role supported by the engine.
///
/// * `name` — role spelling supplied by the author.
///
/// # Errors
///
/// Returns an adapter error for unknown roles.
fn parse_role(name: &str) -> Result<Role, SchemaError> {
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
                "FocusScope does not support role `{name}`"
            )));
        }
    };
    Ok(role)
}
