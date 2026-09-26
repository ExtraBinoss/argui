use argui_schema::{
    NativeElementInput, NativeEventValue, ObservationKind, SchemaError, SchemaValue, builtin,
};
use argui_ui::{
    Color, EventHandler, EventHandlerId, EventOwnerId, EventType, Overflow, ScrollAxes,
    ScrollbarSide, ScrollbarVisibility, length,
};

/// Flickable leaves the scrollbar and effects to surrounding components.
#[test]
fn flickable_scrolls_both_axes_without_prescribed_paint() {
    let element = builtin::registry()
        .unwrap()
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new()
                .property(builtin::SCROLL_X, SchemaValue::Bool(true))
                .property(builtin::SCROLL_Y, SchemaValue::Bool(true)),
        )
        .unwrap();
    let scroll = element.scroll.as_ref().unwrap();
    assert_eq!(scroll.axes, ScrollAxes::Both);
    assert!(scroll.scrollbar.is_none());
    assert!(scroll.effects.is_empty());
    assert_eq!(element.style.overflow.x, Overflow::Auto);
    assert_eq!(element.style.overflow.y, Overflow::Auto);
}

/// A disabled axis stays clipped and does not receive scroll input.
#[test]
fn flickable_can_disable_scrolling_without_adding_visual_chrome() {
    let element = builtin::registry()
        .unwrap()
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new()
                .property(builtin::SCROLL_X, SchemaValue::Bool(false))
                .property(builtin::SCROLL_Y, SchemaValue::Bool(false)),
        )
        .unwrap();
    assert!(!element.scroll.as_ref().unwrap().enabled);
    assert_eq!(element.style.overflow.x, Overflow::Hidden);
    assert_eq!(element.style.overflow.y, Overflow::Hidden);
}

/// Scroll measurements are engine outputs that declarative visuals may read.
#[test]
fn flickable_exposes_read_only_scroll_metrics() {
    let registry = builtin::registry().unwrap();
    let schema = registry.schema(builtin::SCROLL_VIEW).unwrap();
    for (name, observation) in [
        ("offsetX", ObservationKind::ScrollX),
        ("offsetY", ObservationKind::ScrollY),
        ("viewportWidth", ObservationKind::ViewportWidth),
        ("viewportHeight", ObservationKind::ViewportHeight),
        ("contentWidth", ObservationKind::ContentWidth),
        ("contentHeight", ObservationKind::ContentHeight),
    ] {
        let property = schema
            .properties
            .iter()
            .find(|property| property.name == name)
            .unwrap();
        assert_eq!(property.observation, Some(observation));
        assert!(property.read_only);
    }
    let error = registry
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new()
                .property(builtin::OFFSET_Y, SchemaValue::Dimension(length(20.0))),
        )
        .unwrap_err();
    assert!(matches!(error, SchemaError::ReadOnlyProperty { .. }));
}

#[test]
fn horizontal_flickable_reserves_only_its_enabled_axis_and_delivers_scroll() {
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(43), 0));
    let element = builtin::registry()
        .unwrap()
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new()
                .property(builtin::SCROLL_X, SchemaValue::Bool(true))
                .property(builtin::SCROLL_Y, SchemaValue::Bool(false))
                .property(builtin::GROW, SchemaValue::Float(2.0))
                .event(NativeEventValue::new(builtin::SCROLL, handler)),
        )
        .unwrap();
    assert_eq!(
        element.scroll.as_ref().unwrap().axes,
        ScrollAxes::Horizontal
    );
    assert!(element.scroll.as_ref().unwrap().enabled);
    assert_eq!(element.style.overflow.x, Overflow::Auto);
    assert_eq!(element.style.overflow.y, Overflow::Hidden);
    assert_eq!(element.event_listeners.len(), 1);
    assert_eq!(element.event_listeners[0].event, EventType::Scroll);
}

#[test]
fn scroll_view_opt_in_bar_has_a_typed_left_side_and_native_hover_style() {
    let registry = builtin::registry().unwrap();
    let thumb = Color::srgb(0.3, 0.4, 0.5);
    let element = registry
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new()
                .property(builtin::SCROLLBAR_SIDE, SchemaValue::String("left".into()))
                .property(builtin::SCROLLBAR_WIDTH, SchemaValue::Float(3.0))
                .property(builtin::SCROLLBAR_THUMB_COLOR, SchemaValue::Color(thumb))
                .property(
                    builtin::SCROLLBAR_HOVER_COLOR,
                    SchemaValue::Color(Color::WHITE),
                ),
        )
        .unwrap();
    let bar = element.scroll.as_ref().unwrap().scrollbar.as_ref().unwrap();
    assert_eq!(bar.side, ScrollbarSide::Left);
    assert_eq!(bar.width, 3.0);
    assert_eq!(bar.visibility, ScrollbarVisibility::Always);
    assert_eq!(
        bar.thumb.base.background,
        Some(argui_paint::Fill::Solid(thumb))
    );

    assert!(matches!(
        registry.construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new().property(
                builtin::SCROLLBAR_SIDE,
                SchemaValue::String("center".into())
            )
        ),
        Err(SchemaError::InvalidPropertyValue { .. })
    ));
    assert!(
        registry
            .construct(
                builtin::SCROLL_VIEW,
                &NativeElementInput::new()
                    .property(builtin::SCROLLBAR_WIDTH, SchemaValue::Float(0.0)),
            )
            .is_err()
    );
}
