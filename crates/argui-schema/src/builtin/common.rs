//! Shared schema declarations and style application for built-in primitives.

use argui_ui::{
    AlignSelf, DesktopBackdrop, Display, Element, EventType, ExpandedLengthPercentageAuto,
    LengthPercentageAuto, Position, TextSelectionHighlight,
};

use super::{
    ALIGN_SELF, BACKDROP_FILTER, BACKGROUND, BLUR, CLICK, DESKTOP_BACKDROP_FALLBACK,
    DESKTOP_BACKDROP_TINT, DISMISS, FOCUS, GAP, GROW, HEIGHT, ID, INPUT_CHANGED, INSET, MARGIN,
    MAX_HEIGHT, MAX_WIDTH, MIN_HEIGHT, MIN_WIDTH, OPACITY, PADDING, POSITION, ROTATION, SCROLL,
    SELECTION_COLOR, SELECTION_FILL, SELECTION_RADIUS, SHRINK, SUBMIT, TEXT_EDIT, TOOLTIP,
    VIRTUAL_MEASURE, VIRTUAL_WINDOW_CHANGE, VISIBLE, WIDTH,
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
    Position,
    Inset,
    Rotation,
    Opacity,
    BackdropFilter,
    DesktopBackdropTint,
    DesktopBackdropFallback,
    Visible,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    Grow,
    Shrink,
    AlignSelf,
    Margin,
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
        CommonProperty::Key => PropertySchema::new(
            ID,
            "id",
            ValueType::String,
            "Optional addressable native identity; framework key is separate.",
        ),
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
        CommonProperty::Position => PropertySchema::new(
            POSITION,
            "position",
            ValueType::String,
            "Relative, absolute, or sticky positioning.",
        )
        .allowed_values(&["relative", "absolute", "sticky"]),
        CommonProperty::Inset => PropertySchema::new(
            INSET,
            "inset",
            ValueType::PositionInsets,
            "Positioned offsets; start/end follow writing direction.",
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
            "backdropFilter",
            ValueType::String,
            "CSS-like ordered filters applied to pixels already painted behind this element.",
        )
        .default_value(SchemaValue::String("none".into())),
        CommonProperty::DesktopBackdropTint => PropertySchema::new(
            DESKTOP_BACKDROP_TINT,
            "desktopBackdropTint",
            ValueType::Color,
            "Tint painted over a native desktop blur; pair with desktop_backdrop_fallback.",
        ),
        CommonProperty::DesktopBackdropFallback => PropertySchema::new(
            DESKTOP_BACKDROP_FALLBACK,
            "desktopBackdropFallback",
            ValueType::Color,
            "Color painted when native desktop blur is unavailable; pair with desktop_backdrop_tint.",
        ),
        CommonProperty::Visible => PropertySchema::new(
            VISIBLE,
            "visible",
            ValueType::Bool,
            "Whether the element participates in layout and painting.",
        )
        .default_value(SchemaValue::Bool(true)),
        CommonProperty::MinWidth => PropertySchema::new(
            MIN_WIDTH,
            "minWidth",
            ValueType::Constraint,
            "Minimum width as pixels, a percentage, or auto.",
        ),
        CommonProperty::MinHeight => PropertySchema::new(
            MIN_HEIGHT,
            "minHeight",
            ValueType::Constraint,
            "Minimum height as pixels, a percentage, or auto.",
        ),
        CommonProperty::MaxWidth => PropertySchema::new(
            MAX_WIDTH,
            "maxWidth",
            ValueType::Constraint,
            "Maximum width as pixels, a percentage, or auto.",
        ),
        CommonProperty::MaxHeight => PropertySchema::new(
            MAX_HEIGHT,
            "maxHeight",
            ValueType::Constraint,
            "Maximum height as pixels, a percentage, or auto.",
        ),
        CommonProperty::Grow => {
            PropertySchema::new(GROW, "grow", ValueType::Float, "Flex growth factor.")
        }
        CommonProperty::Shrink => {
            PropertySchema::new(SHRINK, "shrink", ValueType::Float, "Flex shrink factor.")
        }
        CommonProperty::AlignSelf => PropertySchema::new(
            ALIGN_SELF,
            "alignSelf",
            ValueType::String,
            "Alignment within the parent cross axis.",
        )
        .allowed_values(&["start", "center", "end", "stretch"]),
        CommonProperty::Margin => PropertySchema::new(
            MARGIN,
            "margin",
            ValueType::Insets,
            "Outer spacing; start/end follow writing direction.",
        ),
        CommonProperty::Background => PropertySchema::new(
            BACKGROUND,
            "background",
            ValueType::Color,
            "Background color.",
        ),
        CommonProperty::SelectionFill => PropertySchema::new(
            SELECTION_FILL,
            "selectionFill",
            ValueType::Brush,
            "Inherited GPU fill for selected text fragments.",
        ),
        CommonProperty::SelectionColor => PropertySchema::new(
            SELECTION_COLOR,
            "selectionColor",
            ValueType::Color,
            "Solid color for selected text when no selection_fill is set.",
        ),
        CommonProperty::SelectionRadius => PropertySchema::new(
            SELECTION_RADIUS,
            "selectionRadius",
            ValueType::Float,
            "Corner radius of selected text fragments.",
        ),
        CommonProperty::Gap => {
            PropertySchema::new(GAP, "gap", ValueType::Float, "Uniform child gap.")
                .default_value(SchemaValue::Float(0.0))
        }
        CommonProperty::Padding => PropertySchema::new(
            PADDING,
            "padding",
            ValueType::Insets,
            "Inner spacing in logical pixels; start/end follow writing direction.",
        ),
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
            VIRTUAL_MEASURE => EventType::VirtualMeasure,
            VIRTUAL_WINDOW_CHANGE => EventType::VirtualWindow,
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
/// Returns an adapter error when `backdrop_filter` is invalid or desktop backdrop colors are unpaired.
pub(super) fn apply_common(
    mut element: Element,
    input: &NativeElementInput,
) -> Result<Element, SchemaError> {
    if let Some(SchemaValue::String(value)) = input.get(ID) {
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
    if let Some(SchemaValue::String(value)) = input.get(POSITION) {
        element = element.position(match value.as_str() {
            "relative" => Position::Relative,
            "absolute" => Position::Absolute,
            "sticky" => Position::Sticky,
            _ => {
                return Err(SchemaError::Adapter(format!(
                    "unsupported position `{value}`"
                )));
            }
        });
    }
    if let Some(SchemaValue::PositionInsets(value)) = input.get(INSET) {
        if input.get(POSITION).is_none() {
            return Err(SchemaError::Adapter(
                "inset requires an explicit position".into(),
            ));
        }
        element = element.layout_inset(*value);
    }
    if let Some(SchemaValue::Constraint(value)) = input.get(MIN_WIDTH) {
        validate_constraint(*value, "minWidth")?;
        element = element.min_width(*value);
    }
    if let Some(SchemaValue::Constraint(value)) = input.get(MIN_HEIGHT) {
        validate_constraint(*value, "minHeight")?;
        element = element.min_height(*value);
    }
    if let Some(SchemaValue::Constraint(value)) = input.get(MAX_WIDTH) {
        validate_constraint(*value, "maxWidth")?;
        element = element.max_width(*value);
    }
    if let Some(SchemaValue::Constraint(value)) = input.get(MAX_HEIGHT) {
        validate_constraint(*value, "maxHeight")?;
        element = element.max_height(*value);
    }
    if let Some(SchemaValue::Float(value)) = input.get(GROW) {
        if !value.is_finite() || *value < 0.0 {
            return Err(SchemaError::Adapter(
                "grow must be finite and nonnegative".into(),
            ));
        }
        element = element.grow(*value);
    }
    if let Some(SchemaValue::Float(value)) = input.get(SHRINK) {
        if !value.is_finite() || *value < 0.0 {
            return Err(SchemaError::Adapter(
                "shrink must be finite and nonnegative".into(),
            ));
        }
        element = element.shrink(*value);
    }
    if let Some(SchemaValue::String(value)) = input.get(ALIGN_SELF) {
        element = element.align_self(match value.as_str() {
            "start" => AlignSelf::FLEX_START,
            "center" => AlignSelf::CENTER,
            "end" => AlignSelf::FLEX_END,
            "stretch" => AlignSelf::STRETCH,
            _ => {
                return Err(SchemaError::Adapter(format!(
                    "unsupported alignSelf `{value}`"
                )));
            }
        });
    }
    if let Some(SchemaValue::Insets(value)) = input.get(MARGIN) {
        element = element.layout_margin(*value);
    }
    if let Some(SchemaValue::Color(value)) = input.get(BACKGROUND) {
        element = element.background(*value);
    }
    match (
        input.get(DESKTOP_BACKDROP_TINT),
        input.get(DESKTOP_BACKDROP_FALLBACK),
    ) {
        (Some(SchemaValue::Color(tint)), Some(SchemaValue::Color(fallback))) => {
            element = element.desktop_backdrop(DesktopBackdrop::new(*tint, *fallback));
        }
        (None, None) => {}
        _ => {
            return Err(SchemaError::Adapter(
                "desktop_backdrop_tint and desktop_backdrop_fallback must be set together".into(),
            ));
        }
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

/// Applies child spacing shared by native containers and controls.
///
/// * `element` — native element receiving spacing.
/// * `input` — validated native property input.
pub(super) fn apply_container(mut element: Element, input: &NativeElementInput) -> Element {
    if let Some(SchemaValue::Float(value)) = input.get(GAP) {
        element = element.gap(*value);
    }
    if let Some(SchemaValue::Insets(value)) = input.get(PADDING) {
        element = element.layout_padding(*value);
    }
    element
}

/// Rejects negative or nonfinite pixel and percent layout constraints.
///
/// `value` is the typed min/max bound, and `name` appears in diagnostics.
/// Returns an adapter error for an invalid bound.
pub(super) fn validate_constraint(
    value: LengthPercentageAuto,
    name: &str,
) -> Result<(), SchemaError> {
    let valid = match value.expand() {
        ExpandedLengthPercentageAuto::Length(value)
        | ExpandedLengthPercentageAuto::Percent(value) => value.is_finite() && value >= 0.0,
        ExpandedLengthPercentageAuto::Auto => true,
    };
    valid
        .then_some(())
        .ok_or_else(|| SchemaError::Adapter(format!("{name} must be finite and nonnegative")))
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
