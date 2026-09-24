//! Unpainted portal and focus boundary for declarative popup content.

use argui_ui::{
    DismissPolicy, Element, EventType, FloatingPlacement, FocusContainment, FocusScope,
    FocusTarget, InitialFocus, Interaction, Placement, ViewportAlign, ViewportPlacement,
    WindowLayer,
};

use super::{
    ANCHOR, CHILDREN, CommonProperty, DISMISS, DISMISS_POLICY, FOCUS_CONTAINMENT, INITIAL_FOCUS,
    PLACEMENT, POPUP_WINDOW, RESTORE_FOCUS, VISIBLE, WINDOW_LAYER, apply_common, common_event,
    common_property, optional_bool,
};
use crate::{
    NativeElementInput, NativeSchema, PropertySchema, SchemaError, SchemaRegistry, SchemaValue,
    SlotArity, SlotSchema, ValueType,
};

/// Registers a style-free portal with edge placement, dismissal, and focus policy.
///
/// * `registry` — native registry receiving the popup metadata and adapter.
///
/// # Errors
///
/// Returns a schema error when built-in identifiers or metadata conflict.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        POPUP_WINDOW,
        "PopupWindow",
        "Portal and focus boundary whose visual content is supplied by children.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::BackdropFilter))
        .property(common_property(CommonProperty::DesktopBackdropTint))
        .property(common_property(CommonProperty::DesktopBackdropFallback))
    .property(common_property(CommonProperty::Visible))
    .property(PropertySchema::new(
        ANCHOR,
        "anchor",
        ValueType::String,
        "Key of an optional anchor; omitted for viewport placement.",
    ))
    .property(PropertySchema::new(
        PLACEMENT,
        "placement",
        ValueType::String,
        "Anchor side or viewport alignment; collisions fit the viewport.",
    ))
    .property(
        PropertySchema::new(
            DISMISS_POLICY,
            "dismiss_policy",
            ValueType::String,
            "Dismissal trigger: manual, outside_pointer, outside_hover, escape, or combined policies.",
        )
        .default_value(SchemaValue::String("outside_pointer_or_escape".into())),
    )
    .property(
        PropertySchema::new(
            WINDOW_LAYER,
            "window_layer",
            ValueType::String,
            "Portal stacking layer, such as popover or modal.",
        )
        .default_value(SchemaValue::String("popover".into())),
    )
    .property(PropertySchema::new(
        FOCUS_CONTAINMENT,
        "containment",
        ValueType::String,
        "Focus containment: none, trap, or modal.",
    ))
    .property(
        PropertySchema::new(
            RESTORE_FOCUS,
            "restore_focus",
            ValueType::Bool,
            "Restore previous focus after the popup closes.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(PropertySchema::new(
        INITIAL_FOCUS,
        "initial_focus",
        ValueType::String,
        "Initial descendant key, first focusable descendant, or empty for none.",
    ))
    .event(common_event(DISMISS, "dismiss", EventType::Dismiss))
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Popup visuals and interactions supplied by the author.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let layer = parse_layer(input.get(WINDOW_LAYER))?;
        let dismiss = parse_dismiss(input.get(DISMISS_POLICY))?;
        let containment = parse_containment(input.get(FOCUS_CONTAINMENT))?;
        let initial = parse_initial(input.get(INITIAL_FOCUS));
        let anchor = match input.get(ANCHOR) {
            Some(SchemaValue::String(value)) if !value.is_empty() => Some(value.as_str()),
            _ => None,
        };
        let placement = match input.get(PLACEMENT) {
            Some(SchemaValue::String(value)) => Some(value.as_str()),
            _ => None,
        };
        let mut element =
            apply_common(Element::container(input.children(CHILDREN).to_vec()), input)?;
        if !optional_bool(input, VISIBLE).unwrap_or(true) {
            return Ok(element);
        }
        element = match anchor {
            Some(key) => element.anchored_portal(
                layer,
                key,
                FloatingPlacement::new(parse_anchor_placement(
                    placement.unwrap_or("bottom_start"),
                )?),
            ),
            None => element.viewport_portal(
                layer,
                parse_viewport_placement(placement.unwrap_or("center"))?,
            ),
        };
        element = element.portal_dismiss(dismiss).focus_scope(FocusScope {
            containment,
            initial,
            restore: optional_bool(input, RESTORE_FOCUS).unwrap_or(true),
        });
        if layer == WindowLayer::Modal || dismiss == DismissPolicy::OutsideHoverOrEscape {
            element = element.interaction(Interaction::blocker());
        }
        if let Some(handler) = input.event_handler(DISMISS) {
            element = element
                .on(handler.direct_listener(EventType::Dismiss))
                .on(handler.direct_listener(EventType::PointerOutside));
        }
        Ok(element)
    })
}

/// Parses the stacking layer for a popup portal.
///
/// * `value` — optional validated declarative layer.
///
/// # Errors
///
/// Returns an adapter error for an unknown layer.
fn parse_layer(value: Option<&SchemaValue>) -> Result<WindowLayer, SchemaError> {
    let name = match value {
        Some(SchemaValue::String(name)) => name.as_str(),
        _ => "popover",
    };
    match name {
        "background" => Ok(WindowLayer::Background),
        "content" => Ok(WindowLayer::Content),
        "floating" => Ok(WindowLayer::Floating),
        "popover" => Ok(WindowLayer::Popover),
        "modal" => Ok(WindowLayer::Modal),
        "debug" => Ok(WindowLayer::Debug),
        _ => Err(SchemaError::Adapter(format!(
            "PopupWindow does not support window_layer `{name}`"
        ))),
    }
}

/// Parses the triggers that request popup dismissal.
///
/// * `value` — optional validated declarative policy.
///
/// # Errors
///
/// Returns an adapter error for an unknown dismissal policy.
fn parse_dismiss(value: Option<&SchemaValue>) -> Result<DismissPolicy, SchemaError> {
    let name = match value {
        Some(SchemaValue::String(name)) => name.as_str(),
        _ => "outside_pointer_or_escape",
    };
    match name {
        "manual" => Ok(DismissPolicy::Manual),
        "outside_pointer" => Ok(DismissPolicy::OutsidePointer),
        "escape" => Ok(DismissPolicy::Escape),
        "outside_pointer_or_escape" => Ok(DismissPolicy::OutsidePointerOrEscape),
        "outside_hover_or_escape" => Ok(DismissPolicy::OutsideHoverOrEscape),
        _ => Err(SchemaError::Adapter(format!(
            "PopupWindow does not support dismiss_policy `{name}`"
        ))),
    }
}

/// Parses focus containment for a mounted popup.
///
/// * `value` — optional validated declarative containment policy.
///
/// # Errors
///
/// Returns an adapter error for an unknown containment policy.
fn parse_containment(value: Option<&SchemaValue>) -> Result<FocusContainment, SchemaError> {
    let name = match value {
        Some(SchemaValue::String(name)) => name.as_str(),
        _ => "none",
    };
    match name {
        "none" => Ok(FocusContainment::None),
        "trap" => Ok(FocusContainment::Trap),
        "modal" => Ok(FocusContainment::Modal),
        _ => Err(SchemaError::Adapter(format!(
            "PopupWindow does not support containment `{name}`"
        ))),
    }
}

/// Resolves an initial focus target, using the first focusable child by default.
///
/// * `value` — optional validated initial focus spelling.
fn parse_initial(value: Option<&SchemaValue>) -> Option<InitialFocus> {
    match value {
        Some(SchemaValue::String(name)) if name.is_empty() => None,
        Some(SchemaValue::String(name)) if name == "first" => Some(InitialFocus::First),
        Some(SchemaValue::String(name)) => {
            Some(InitialFocus::Target(FocusTarget::Key(name.clone())))
        }
        _ => Some(InitialFocus::First),
    }
}

/// Parses an anchor-relative side and edge alignment.
///
/// * `name` — validated placement spelling.
///
/// # Errors
///
/// Returns an adapter error for unsupported anchor placement.
fn parse_anchor_placement(name: &str) -> Result<Placement, SchemaError> {
    match name {
        "top_start" => Ok(Placement::TopStart),
        "top" => Ok(Placement::Top),
        "top_end" => Ok(Placement::TopEnd),
        "bottom_start" => Ok(Placement::BottomStart),
        "bottom" => Ok(Placement::Bottom),
        "bottom_end" => Ok(Placement::BottomEnd),
        "left_start" => Ok(Placement::LeftStart),
        "left" => Ok(Placement::Left),
        "left_end" => Ok(Placement::LeftEnd),
        "right_start" => Ok(Placement::RightStart),
        "right" => Ok(Placement::Right),
        "right_end" => Ok(Placement::RightEnd),
        _ => Err(SchemaError::Adapter(format!(
            "PopupWindow does not support anchored placement `{name}`"
        ))),
    }
}

/// Parses viewport-relative placement without an anchor.
///
/// * `name` — validated placement spelling.
///
/// # Errors
///
/// Returns an adapter error for unsupported viewport placement.
fn parse_viewport_placement(name: &str) -> Result<ViewportPlacement, SchemaError> {
    let placement = match name {
        "fill" => ViewportPlacement::fill(),
        "center" => ViewportPlacement::centered(),
        "top_start" => {
            ViewportPlacement::centered().align(ViewportAlign::Start, ViewportAlign::Start)
        }
        "top" => ViewportPlacement::centered().align(ViewportAlign::Center, ViewportAlign::Start),
        "top_end" => ViewportPlacement::centered().align(ViewportAlign::End, ViewportAlign::Start),
        "bottom_start" => {
            ViewportPlacement::centered().align(ViewportAlign::Start, ViewportAlign::End)
        }
        "bottom" => ViewportPlacement::centered().align(ViewportAlign::Center, ViewportAlign::End),
        "bottom_end" => ViewportPlacement::centered().align(ViewportAlign::End, ViewportAlign::End),
        _ => {
            return Err(SchemaError::Adapter(format!(
                "PopupWindow does not support viewport placement `{name}`"
            )));
        }
    };
    Ok(placement)
}
