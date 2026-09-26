//! Native scroll viewport with an overflow-aware scrollbar.

use argui_animation::{Duration, Tween};
use argui_paint::{CornerRadii, QuadStyle};
use argui_ui::{
    Axes, Color, Element, EventType, InertialScroll, Overflow, ScrollAxes, ScrollConfig,
    ScrollPhysics, ScrollbarPartStyle, ScrollbarSide, ScrollbarStyle, ScrollbarVisibility, Sides,
    StylePatch, StyleTransition, Transition, VisualState, property,
};

use super::{
    CHILDREN, CONTENT_HEIGHT, CONTENT_WIDTH, CommonProperty, ENABLED, FLICK_VIEWPORT_HEIGHT,
    FLICK_VIEWPORT_WIDTH, GROW, OFFSET_X, OFFSET_Y, SCROLL, SCROLL_MOMENTUM, SCROLL_X, SCROLL_Y,
    SCROLLBAR_HOVER_COLOR, SCROLLBAR_HOVER_WIDTH, SCROLLBAR_PRESSED_COLOR, SCROLLBAR_SIDE,
    SCROLLBAR_THUMB_COLOR, SCROLLBAR_TRACK_COLOR, SCROLLBAR_VISIBLE, SCROLLBAR_WIDTH, apply_common,
    common_property, optional_bool,
};
use crate::{
    EventSchema, NativeElementInput, NativeSchema, ObservationKind, PropertyId, PropertySchema,
    SchemaError, SchemaRegistry, SchemaValue, SlotArity, SlotSchema, ValueType,
};

/// Registers a scroll viewport with a default overflow-aware scrollbar.
///
/// * `registry` — native schema registry receiving the viewport adapter.
///
/// # Errors
///
/// Returns a schema error if the type or its members conflict with another built-in.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        super::SCROLL_VIEW,
        "ScrollView",
        "Scroll viewport with input physics and independently authored visuals.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Tooltip))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::Position))
    .property(common_property(CommonProperty::Inset))
    .property(common_property(CommonProperty::MinWidth))
    .property(common_property(CommonProperty::MinHeight))
    .property(common_property(CommonProperty::MaxWidth))
    .property(common_property(CommonProperty::MaxHeight))
    .property(common_property(CommonProperty::Shrink))
    .property(common_property(CommonProperty::AlignSelf))
    .property(common_property(CommonProperty::Margin))
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
            "scrollX",
            ValueType::Bool,
            "Allow horizontal scrolling.",
        )
        .default_value(SchemaValue::Bool(false)),
    )
    .property(
        PropertySchema::new(
            SCROLL_Y,
            "scrollY",
            ValueType::Bool,
            "Allow vertical scrolling.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(PropertySchema::new(
        SCROLLBAR_SIDE,
        "scrollbarSide",
        ValueType::String,
        "Physical edge of the vertical scrollbar: left or right.",
    ))
    .property(PropertySchema::new(
        SCROLLBAR_WIDTH,
        "scrollbarWidth",
        ValueType::Float,
        "Resting width of the native scrollbar in logical pixels.",
    ))
    .property(
        PropertySchema::new(
            SCROLLBAR_VISIBLE,
            "scrollbarVisible",
            ValueType::Bool,
            "Show the native scrollbar when content overflows; set false to hide it.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(PropertySchema::new(
        SCROLLBAR_THUMB_COLOR,
        "scrollbarThumbColor",
        ValueType::Color,
        "Native thumb color before hover.",
    ))
    .property(PropertySchema::new(
        SCROLLBAR_TRACK_COLOR,
        "scrollbarTrackColor",
        ValueType::Color,
        "Native scrollbar track color.",
    ))
    .property(PropertySchema::new(
        SCROLLBAR_HOVER_COLOR,
        "scrollbarHoverColor",
        ValueType::Color,
        "Native thumb color while hovered.",
    ))
    .property(PropertySchema::new(
        SCROLLBAR_HOVER_WIDTH,
        "scrollbarHoverWidth",
        ValueType::Float,
        "Thumb and hit-target width on hover, eased by the native renderer.",
    ))
    .property(PropertySchema::new(
        SCROLLBAR_PRESSED_COLOR,
        "scrollbarPressedColor",
        ValueType::Color,
        "Native thumb color while dragging.",
    ))
    .property(PropertySchema::new(
        SCROLL_MOMENTUM,
        "scrollMomentum",
        ValueType::Float,
        "Scroll glide strength from zero (direct) to one (longest glide).",
    ))
    .property(PropertySchema::new(
        GROW,
        "grow",
        ValueType::Float,
        "Flex growth inside the surrounding layout.",
    ))
    .property(
        PropertySchema::new(
            OFFSET_X,
            "offsetX",
            ValueType::Dimension,
            "Current horizontal scroll offset.",
        )
        .observed(ObservationKind::ScrollX),
    )
    .property(
        PropertySchema::new(
            OFFSET_Y,
            "offsetY",
            ValueType::Dimension,
            "Current vertical scroll offset.",
        )
        .observed(ObservationKind::ScrollY),
    )
    .property(
        PropertySchema::new(
            FLICK_VIEWPORT_WIDTH,
            "viewportWidth",
            ValueType::Dimension,
            "Laid-out viewport width.",
        )
        .observed(ObservationKind::ViewportWidth),
    )
    .property(
        PropertySchema::new(
            FLICK_VIEWPORT_HEIGHT,
            "viewportHeight",
            ValueType::Dimension,
            "Laid-out viewport height.",
        )
        .observed(ObservationKind::ViewportHeight),
    )
    .property(
        PropertySchema::new(
            CONTENT_WIDTH,
            "contentWidth",
            ValueType::Dimension,
            "Scrollable content width.",
        )
        .observed(ObservationKind::ContentWidth),
    )
    .property(
        PropertySchema::new(
            CONTENT_HEIGHT,
            "contentHeight",
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
        let mut config = ScrollConfig::default()
            .enabled(optional_bool(input, ENABLED).unwrap_or(true) && (x || y))
            .axes(axes);
        if let Some(physics) = scroll_momentum(input, SCROLL_MOMENTUM)? {
            config = config.physics(physics);
        }
        if optional_bool(input, SCROLLBAR_VISIBLE).unwrap_or(true) {
            let side = match input.get(SCROLLBAR_SIDE) {
                Some(SchemaValue::String(value)) if value == "left" => ScrollbarSide::Left,
                _ => ScrollbarSide::Right,
            };
            let width = match input.get(SCROLLBAR_WIDTH) {
                Some(SchemaValue::Float(value)) if value.is_finite() && *value > 0.0 => *value,
                Some(_) => {
                    return Err(SchemaError::Adapter(
                        "scrollbarWidth must be a positive finite number".into(),
                    ));
                }
                None => 4.0,
            };
            let thumb = match input.get(SCROLLBAR_THUMB_COLOR) {
                Some(SchemaValue::Color(color)) => *color,
                _ => Color::srgba(0.5, 0.5, 0.5, 0.45),
            };
            let hover = match input.get(SCROLLBAR_HOVER_COLOR) {
                Some(SchemaValue::Color(color)) => *color,
                _ => Color::srgba(0.5, 0.5, 0.5, 0.9),
            };
            let track = match input.get(SCROLLBAR_TRACK_COLOR) {
                Some(SchemaValue::Color(color)) => QuadStyle::solid(*color),
                _ => QuadStyle::default(),
            };
            let pressed = match input.get(SCROLLBAR_PRESSED_COLOR) {
                Some(SchemaValue::Color(color)) => *color,
                _ => hover,
            };
            let hover_width = match input.get(SCROLLBAR_HOVER_WIDTH) {
                Some(SchemaValue::Float(value)) if value.is_finite() && *value >= width => *value,
                Some(_) => {
                    return Err(SchemaError::Adapter(
                        "scrollbarHoverWidth must be finite and at least scrollbarWidth".into(),
                    ));
                }
                None => width + 4.0,
            };
            let scrollbar = ScrollbarStyle::new(
                ScrollbarPartStyle::new(track.radius(CornerRadii::all(hover_width / 2.0))),
                ScrollbarPartStyle::new(
                    QuadStyle::solid(thumb).radius(CornerRadii::all(hover_width / 2.0)),
                )
                .when(
                    VisualState::Hovered,
                    StylePatch::new().set(property::BackgroundColor, hover),
                )
                .when(
                    VisualState::Pressed,
                    StylePatch::new().set(property::BackgroundColor, pressed),
                )
                .transition(StyleTransition::new(Transition::tween(Tween::new(
                    Duration::from_millis(120),
                )))),
            )
            .side(side)
            .width(width)
            .insets(Sides {
                left: 0.0,
                right: 0.0,
                top: 4.0,
                bottom: 4.0,
            })
            .visibility(ScrollbarVisibility::Always);
            config = config.scrollbar(scrollbar.hover_width(hover_width));
        }
        let overflow = Axes {
            x: if x { Overflow::Auto } else { Overflow::Hidden },
            y: if y { Overflow::Auto } else { Overflow::Hidden },
        };
        let mut element = apply_common(Element::column(input.children(CHILDREN).to_vec()), input)?
            .overflow(overflow)
            .scroll_config(config);
        for event in &input.events {
            if event.id == SCROLL {
                element = element.on(event.handler.direct_listener(EventType::Scroll));
            }
        }
        Ok(element)
    })
}

/// Parses a public glide strength into native scroll physics.
///
/// * `input` — authored primitive properties.
/// * `property` — scroll-momentum property identifier.
///
/// Returns `None` when the caller should keep its default physics.
///
/// # Errors
///
/// Returns an adapter error for a nonfinite or out-of-range strength.
pub(super) fn scroll_momentum(
    input: &NativeElementInput,
    property: PropertyId,
) -> Result<Option<ScrollPhysics>, SchemaError> {
    let Some(value) = input.get(property) else {
        return Ok(None);
    };
    let SchemaValue::Float(strength) = value else {
        return Err(SchemaError::Adapter(
            "scrollMomentum must be a number in [0, 1]".into(),
        ));
    };
    if !strength.is_finite() || !(0.0..=1.0).contains(strength) {
        return Err(SchemaError::Adapter(
            "scrollMomentum must be a number in [0, 1]".into(),
        ));
    }
    if *strength == 0.0 {
        return Ok(Some(ScrollPhysics::Direct));
    }
    Ok(Some(ScrollPhysics::Inertial(InertialScroll {
        decay: 16.0 - 11.0 * strength,
        ..InertialScroll::default()
    })))
}
