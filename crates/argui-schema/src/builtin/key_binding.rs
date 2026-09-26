//! Layout-neutral typed keyboard shortcut for declarative composition.

use argui_core::Key;
use argui_ui::{Element, EventType, Position, Shortcut, length};

use super::{
    ACTIVATED, CommonProperty, ENABLED, KEY_BINDING, SHORTCUT, apply_common, common_event,
    common_property, optional_bool, required_string,
};
use crate::{
    NativeElementInput, NativeSchema, PropertySchema, SchemaError, SchemaRegistry, SchemaValue,
    ValueType,
};

/// Registers a keyboard binding that occupies no layout space and paints nothing.
///
/// * `registry` — native registry receiving the binding metadata and adapter.
///
/// # Errors
///
/// Returns a schema error when built-in identifiers or metadata conflict.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        KEY_BINDING,
        "KeyBinding",
        "Typed shortcut active across the current tree unless focused input handles the key.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::Visible))
    .property(
        PropertySchema::new(
            SHORTCUT,
            "shortcut",
            ValueType::String,
            "Key chord such as Primary+Shift+K or Escape.",
        )
        .required(),
    )
    .property(
        PropertySchema::new(
            ENABLED,
            "enabled",
            ValueType::Bool,
            "Enable shortcut delivery.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .event(common_event(ACTIVATED, "activated", EventType::Key));
    registry.register(schema, |input: &NativeElementInput| {
        let shortcut = parse_shortcut(required_string(input, SHORTCUT, "shortcut")?)?;
        let mut element = apply_common(Element::container([]), input)?
            .position(Position::Absolute)
            .width(length(0.0))
            .height(length(0.0));
        if optional_bool(input, ENABLED).unwrap_or(true)
            && optional_bool(input, super::VISIBLE).unwrap_or(true)
            && let Some(handler) = input.event_handler(ACTIVATED)
        {
            element = element.on(handler
                .listener(EventType::Key)
                .shortcut(shortcut)
                .global_key());
        }
        Ok(element)
    })
}

/// Parses a case-insensitive chord with optional Primary, Shift, and Alt modifiers.
///
/// * `name` — declarative chord ending in exactly one normalized key name.
///
/// # Errors
///
/// Returns an adapter error for an empty, duplicate, or unknown chord component.
fn parse_shortcut(name: &str) -> Result<Shortcut, SchemaError> {
    let mut primary = false;
    let mut shift = false;
    let mut alt = false;
    let mut key = None;
    for part in name.split('+').map(str::trim) {
        let lower = part.to_lowercase();
        match lower.as_str() {
            "primary" | "ctrl" | "control" | "command" | "cmd" | "meta"
                if !primary && key.is_none() =>
            {
                primary = true
            }
            "shift" if !shift && key.is_none() => shift = true,
            "alt" | "option" if !alt && key.is_none() => alt = true,
            "primary" | "ctrl" | "control" | "command" | "cmd" | "meta" | "shift" | "alt"
            | "option" => return Err(invalid_shortcut(name)),
            _ if key.is_none() => {
                key = Some(parse_key(&lower).ok_or_else(|| invalid_shortcut(name))?)
            }
            _ => return Err(invalid_shortcut(name)),
        }
    }
    let Some(key) = key else {
        return Err(invalid_shortcut(name));
    };
    Ok(Shortcut {
        key,
        primary,
        shift,
        alt,
    })
}

/// Maps one normalized declarative key spelling to an engine key.
///
/// * `name` — lowercased key spelling without modifiers.
fn parse_key(name: &str) -> Option<Key> {
    let key = match name {
        "escape" | "esc" => Key::Escape,
        "enter" | "return" => Key::Enter,
        "tab" => Key::Tab,
        "space" => Key::Character(" ".into()),
        "backspace" => Key::Backspace,
        "delete" => Key::Delete,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" | "pageUp" => Key::PageUp,
        "pagedown" | "pageDown" => Key::PageDown,
        "left" | "arrowleft" => Key::ArrowLeft,
        "right" | "arrowright" => Key::ArrowRight,
        "up" | "arrowup" => Key::ArrowUp,
        "down" | "arrowdown" => Key::ArrowDown,
        "contextmenu" => Key::ContextMenu,
        _ if name.starts_with('f') && name.len() > 1 => match name[1..].parse::<u8>() {
            Ok(number @ 1..=24) => Key::Function(number),
            _ => return None,
        },
        _ if name.chars().count() == 1 => Key::Character(name.into()),
        _ => return None,
    };
    Some(key)
}

/// Reports an invalid authored chord with its original spelling.
///
/// * `name` — unparsed declarative shortcut value.
fn invalid_shortcut(name: &str) -> SchemaError {
    SchemaError::Adapter(format!("KeyBinding does not support shortcut `{name}`"))
}
