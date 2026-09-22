//! Shared schema declarations and style application for built-in primitives.

use argui_ui::{
    Dimension, Display, Element, EventType, ExpandedDimension, LengthPercentageAuto, Sides,
    TextSelectionHighlight,
};

use super::{
    BACKDROP_FILTER, BACKGROUND, BLUR, CLICK, DISMISS, FOCUS, GAP, HEIGHT, INPUT_CHANGED, KEY,
    MIN_HEIGHT, MIN_WIDTH, OPACITY, PADDING, ROTATION, SCROLL, SELECTION_COLOR, SELECTION_FILL,
    SELECTION_RADIUS, SUBMIT, TEXT_EDIT, TOOLTIP, VISIBLE, WIDTH, X, Y,
};
use crate::{
    EventId, EventSchema, NativeElementInput, PropertyId, PropertySchema, SchemaError, SchemaValue,
    ValueType,
};

/// Common native properties shared across visual primitives.
#[derive(Clone, Copy)]
pub(super) enum CommonProperty {
    Key,
    Tooltip,
    Width,
    Height,
    X,
    Y,
    Rotation,
    Opacity,
    BackdropFilter,
    Visible,
    MinWidth,
    MinHeight,
    Background,
    SelectionFill,
    SelectionColor,
    SelectionRadius,
    Gap,
    Padding,
}

/// Declares one common native property with its canonical ID and type.
///
/// * `property` — shared property to declare.
pub(super) fn common_property(property: CommonProperty) -> PropertySchema {
    match property {
        CommonProperty::Key => {
            PropertySchema::new(KEY, "key", ValueType::String, "Public reconciliation key.")
        }
        CommonProperty::Tooltip => PropertySchema::new(
            TOOLTIP,
            "tooltip",
            ValueType::String,
            "Tooltip description.",
        ),
        CommonProperty::Width => {
            PropertySchema::new(WIDTH, "width", ValueType::Dimension, "Preferred width.")
        }
        CommonProperty::Height => {
            PropertySchema::new(HEIGHT, "height", ValueType::Dimension, "Preferred height.")
        }
        CommonProperty::X => PropertySchema::new(
            X,
            "x",
            ValueType::Dimension,
            "Horizontal position within the parent.",
        ),
        CommonProperty::Y => PropertySchema::new(
            Y,
            "y",
            ValueType::Dimension,
            "Vertical position within the parent.",
        ),
        CommonProperty::Rotation => PropertySchema::new(
            ROTATION,
            "rotation",
            ValueType::Float,
            "Clockwise rotation in degrees.",
        )
        .default_value(SchemaValue::Float(0.0)),
        CommonProperty::Opacity => PropertySchema::new(
            OPACITY,
            "opacity",
            ValueType::Float,
            "Group opacity for the element and its descendants.",
        )
        .default_value(SchemaValue::Float(1.0)),
        CommonProperty::BackdropFilter => PropertySchema::new(
            BACKDROP_FILTER,
            "backdrop_filter",
            ValueType::String,
            "CSS-like ordered filters applied to pixels already painted behind this element.",
        )
        .default_value(SchemaValue::String("none".into())),
        CommonProperty::Visible => PropertySchema::new(
            VISIBLE,
            "visible",
            ValueType::Bool,
            "Whether the element participates in layout and painting.",
        )
        .default_value(SchemaValue::Bool(true)),
        CommonProperty::MinWidth => PropertySchema::new(
            MIN_WIDTH,
            "min_width",
            ValueType::Float,
            "Minimum width in logical pixels.",
        ),
        CommonProperty::MinHeight => PropertySchema::new(
            MIN_HEIGHT,
            "min_height",
            ValueType::Float,
            "Minimum height in logical pixels.",
        ),
        CommonProperty::Background => PropertySchema::new(
            BACKGROUND,
            "background",
            ValueType::Color,
            "Background color.",
        ),
        CommonProperty::SelectionFill => PropertySchema::new(
            SELECTION_FILL,
            "selection_fill",
            ValueType::Brush,
            "Inherited GPU fill for selected text fragments.",
        ),
        CommonProperty::SelectionColor => PropertySchema::new(
            SELECTION_COLOR,
            "selection_color",
            ValueType::Color,
            "Solid color for selected text when no selection_fill is set.",
        ),
        CommonProperty::SelectionRadius => PropertySchema::new(
            SELECTION_RADIUS,
            "selection_radius",
            ValueType::Float,
            "Corner radius of selected text fragments.",
        ),
        CommonProperty::Gap => {
            PropertySchema::new(GAP, "gap", ValueType::Float, "Uniform child gap.")
                .default_value(SchemaValue::Float(0.0))
        }
        CommonProperty::Padding => {
            PropertySchema::new(PADDING, "padding", ValueType::Float, "Uniform padding.")
                .default_value(SchemaValue::Float(0.0))
        }
    }
}

/// Creates a common event declaration whose name is shared across primitives.
///
/// * `id` — stable event ID.
/// * `name` — public event name.
/// * `event_type` — engine event routed to this declaration.
pub(super) fn common_event(id: EventId, name: &'static str, event_type: EventType) -> EventSchema {
    EventSchema::new(id, name, event_type, format!("Native `{name}` event."))
}

/// Attaches schema-validated event handlers to an element.
///
/// * `element` — element receiving handlers.
/// * `input` — validated native event input.
pub(super) fn apply_events(mut element: Element, input: &NativeElementInput) -> Element {
    for event in &input.events {
        let event_type = match event.id {
            CLICK => EventType::Click,
            FOCUS => EventType::Focus,
            BLUR => EventType::Blur,
            INPUT_CHANGED => EventType::Input,
            TEXT_EDIT => EventType::TextEdit,
            SUBMIT => EventType::Submit,
            DISMISS => EventType::Dismiss,
            SCROLL => EventType::Scroll,
            _ => continue,
        };
        element = element.on(event.handler.direct_listener(event_type));
    }
    element
}

/// Applies shared element identity, dimensions, and background.
///
/// * `element` — native element receiving common properties.
/// * `input` — validated native property input.
///
/// # Errors
///
/// Returns an adapter error when `backdrop_filter` has invalid syntax or values.
pub(super) fn apply_common(
    mut element: Element,
    input: &NativeElementInput,
) -> Result<Element, SchemaError> {
    if let Some(SchemaValue::String(value)) = input.get(KEY) {
        element = element.keyed(value.clone());
    }
    if let Some(SchemaValue::String(value)) = input.get(TOOLTIP) {
        element = element.tooltip(value.clone());
    }
    if let Some(SchemaValue::Dimension(value)) = input.get(WIDTH) {
        element = element.width(*value);
    }
    if let Some(SchemaValue::Dimension(value)) = input.get(HEIGHT) {
        element = element.height(*value);
    }
    let x = input.get(X).and_then(|value| match value {
        SchemaValue::Dimension(value) => Some(*value),
        _ => None,
    });
    let y = input.get(Y).and_then(|value| match value {
        SchemaValue::Dimension(value) => Some(*value),
        _ => None,
    });
    if x.is_some() || y.is_some() {
        element = element.absolute(Sides {
            left: inset_coordinate(x),
            right: LengthPercentageAuto::auto(),
            top: inset_coordinate(y),
            bottom: LengthPercentageAuto::auto(),
        });
    }
    if let Some(SchemaValue::Float(value)) = input.get(MIN_WIDTH) {
        element = element.min_width(argui_ui::length(*value));
    }
    if let Some(SchemaValue::Float(value)) = input.get(MIN_HEIGHT) {
        element = element.min_height(argui_ui::length(*value));
    }
    if let Some(SchemaValue::Color(value)) = input.get(BACKGROUND) {
        element = element.background(*value);
    }
    let selection_fill = match input.get(SELECTION_FILL) {
        Some(SchemaValue::Brush(value)) => Some(value.clone()),
        _ => None,
    };
    let selection_color = match input.get(SELECTION_COLOR) {
        Some(SchemaValue::Color(value)) => Some(*value),
        _ => None,
    };
    let selection_radius = match input.get(SELECTION_RADIUS) {
        Some(SchemaValue::Float(value)) => Some(*value),
        _ => None,
    };
    if selection_fill.is_some() || selection_color.is_some() || selection_radius.is_some() {
        let fill = selection_fill.unwrap_or_else(|| {
            argui_paint::Fill::Solid(
                selection_color.unwrap_or(argui_ui::TextSelectionStyle::default().background),
            )
        });
        element = element.selection_highlight(
            TextSelectionHighlight::new(fill).radius(selection_radius.unwrap_or(3.0).max(0.0)),
        );
    }
    if let Some(SchemaValue::Float(value)) = input.get(ROTATION) {
        element = element.transform(argui_core::Transform2D::IDENTITY.rotate(value.to_radians()));
    }
    if let Some(SchemaValue::Float(value)) = input.get(OPACITY) {
        element = element.opacity(*value);
    }
    if let Some(SchemaValue::String(value)) = input.get(BACKDROP_FILTER) {
        for filter in crate::backdrop_filter::parse(value).map_err(|message| {
            SchemaError::Adapter(format!("invalid backdrop_filter: {message}"))
        })? {
            element = element.backdrop_filter(filter);
        }
    }
    if matches!(input.get(VISIBLE), Some(SchemaValue::Bool(false))) {
        element = element.display(Display::None);
    }
    Ok(element)
}

/// Converts an authored position coordinate to a Taffy inset value.
///
/// * `value` — optional dimension bound to `x` or `y`.
fn inset_coordinate(value: Option<Dimension>) -> LengthPercentageAuto {
    match value.map(Dimension::expand) {
        Some(ExpandedDimension::Length(pixels)) => LengthPercentageAuto::length(pixels),
        Some(ExpandedDimension::Percent(fraction)) => LengthPercentageAuto::percent(fraction),
        Some(_) | None => LengthPercentageAuto::auto(),
    }
}

/// Applies child spacing shared by native containers and controls.
///
/// * `element` — native element receiving spacing.
/// * `input` — validated native property input.
pub(super) fn apply_container(mut element: Element, input: &NativeElementInput) -> Element {
    if let Some(SchemaValue::Float(value)) = input.get(GAP) {
        element = element.gap(*value);
    }
    if let Some(SchemaValue::Float(value)) = input.get(PADDING) {
        element = element.padding(Sides::length(*value));
    }
    element
}

/// Returns a required string property after registry validation.
///
/// * `input` — validated native property input.
/// * `id` — required property ID.
/// * `name` — diagnostic name for the property.
///
/// # Errors
///
/// Returns an adapter error if registry validation failed to preserve the property.
pub(super) fn required_string<'a>(
    input: &'a NativeElementInput,
    id: PropertyId,
    name: &str,
) -> Result<&'a String, SchemaError> {
    let value = input.get(id).ok_or_else(|| {
        SchemaError::Adapter(format!(
            "required property `{name}` disappeared after validation"
        ))
    })?;
    let SchemaValue::String(value) = value else {
        return Err(SchemaError::Adapter(format!(
            "validated property `{name}` changed type"
        )));
    };
    Ok(value)
}

/// Returns an optional string property after registry validation.
///
/// * `input` — validated native property input.
/// * `id` — optional string property ID.
pub(super) fn optional_string(input: &NativeElementInput, id: PropertyId) -> Option<&String> {
    match input.get(id) {
        Some(SchemaValue::String(value)) => Some(value),
        _ => None,
    }
}

/// Returns an optional boolean property after registry validation.
///
/// * `input` — validated native property input.
/// * `id` — optional boolean property ID.
pub(super) fn optional_bool(input: &NativeElementInput, id: PropertyId) -> Option<bool> {
    match input.get(id) {
        Some(SchemaValue::Bool(value)) => Some(*value),
        _ => None,
    }
}
