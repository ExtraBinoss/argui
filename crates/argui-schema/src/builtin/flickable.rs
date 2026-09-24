//! Visually neutral scroll viewport for declarative scroll compositions.

use argui_ui::{Axes, Element, EventType, Overflow, ScrollAxes, ScrollConfig};

use super::{
    CHILDREN, CONTENT_HEIGHT, CONTENT_WIDTH, CommonProperty, ENABLED, FLICK_VIEWPORT_HEIGHT,
    FLICK_VIEWPORT_WIDTH, GROW, OFFSET_X, OFFSET_Y, SCROLL, SCROLL_X, SCROLL_Y, apply_common,
    common_property, optional_bool,
};
use crate::{
    EventSchema, NativeElementInput, NativeSchema, ObservationKind, PropertySchema, SchemaError,
    SchemaRegistry, SchemaValue, SlotArity, SlotSchema, ValueType,
};

/// Registers a scroll viewport without a prescribed scrollbar or visual effect.
///
/// * `registry` — native schema registry receiving the viewport adapter.
///
/// # Errors
///
/// Returns a schema error if the type or its members conflict with another built-in.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        super::FLICKABLE,
        "Flickable",
        "Scroll viewport with input physics and independently authored visuals.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Tooltip))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::X))
    .property(common_property(CommonProperty::Y))
    .property(common_property(CommonProperty::MinWidth))
    .property(common_property(CommonProperty::MinHeight))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::BackdropFilter))
    .property(common_property(CommonProperty::DesktopBackdropTint))
    .property(common_property(CommonProperty::DesktopBackdropFallback))
    .property(common_property(CommonProperty::Visible))
    .property(
        PropertySchema::new(ENABLED, "enabled", ValueType::Bool, "Accept scroll input.")
            .default_value(SchemaValue::Bool(true)),
    )
    .property(
        PropertySchema::new(
            SCROLL_X,
            "scroll_x",
            ValueType::Bool,
            "Allow horizontal scrolling.",
        )
        .default_value(SchemaValue::Bool(false)),
    )
    .property(
        PropertySchema::new(
            SCROLL_Y,
            "scroll_y",
            ValueType::Bool,
            "Allow vertical scrolling.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(PropertySchema::new(
        GROW,
        "grow",
        ValueType::Float,
        "Flex growth inside the surrounding layout.",
    ))
    .property(
        PropertySchema::new(
            OFFSET_X,
            "offset_x",
            ValueType::Dimension,
            "Current horizontal scroll offset.",
        )
        .observed(ObservationKind::ScrollX),
    )
    .property(
        PropertySchema::new(
            OFFSET_Y,
            "offset_y",
            ValueType::Dimension,
            "Current vertical scroll offset.",
        )
        .observed(ObservationKind::ScrollY),
    )
    .property(
        PropertySchema::new(
            FLICK_VIEWPORT_WIDTH,
            "viewport_width",
            ValueType::Dimension,
            "Laid-out viewport width.",
        )
        .observed(ObservationKind::ViewportWidth),
    )
    .property(
        PropertySchema::new(
            FLICK_VIEWPORT_HEIGHT,
            "viewport_height",
            ValueType::Dimension,
            "Laid-out viewport height.",
        )
        .observed(ObservationKind::ViewportHeight),
    )
    .property(
        PropertySchema::new(
            CONTENT_WIDTH,
            "content_width",
            ValueType::Dimension,
            "Scrollable content width.",
        )
        .observed(ObservationKind::ContentWidth),
    )
    .property(
        PropertySchema::new(
            CONTENT_HEIGHT,
            "content_height",
            ValueType::Dimension,
            "Scrollable content height.",
        )
        .observed(ObservationKind::ContentHeight),
    )
    .event(
        EventSchema::new(
            SCROLL,
            "scroll",
            EventType::Scroll,
            "Viewport scroll position changed.",
        )
        .payload(ValueType::Float),
    )
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Content within the scroll viewport.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let x = optional_bool(input, SCROLL_X).unwrap_or(false);
        let y = optional_bool(input, SCROLL_Y).unwrap_or(true);
        let axes = match (x, y) {
            (true, true) => ScrollAxes::Both,
            (true, false) => ScrollAxes::Horizontal,
            _ => ScrollAxes::Vertical,
        };
        let config = ScrollConfig::default()
            .enabled(optional_bool(input, ENABLED).unwrap_or(true) && (x || y))
            .axes(axes);
        let overflow = Axes {
            x: if x { Overflow::Auto } else { Overflow::Hidden },
            y: if y { Overflow::Auto } else { Overflow::Hidden },
        };
        let mut element = apply_common(Element::column(input.children(CHILDREN).to_vec()), input)?
            .overflow(overflow)
            .scroll_config(config);
        if let Some(SchemaValue::Float(grow)) = input.get(GROW) {
            element = element.grow(*grow);
        }
        for event in &input.events {
            if event.id == SCROLL {
                element = element.on(event.handler.direct_listener(EventType::Scroll));
            }
        }
        Ok(element)
    })
}
