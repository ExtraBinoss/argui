use argui_schema::{
    NativeElementInput, NativeEventValue, ObservationKind, SchemaError, SchemaValue, builtin,
};
use argui_ui::{
    EventHandler, EventHandlerId, EventOwnerId, EventType, Overflow, ScrollAxes, length,
};

/// Flickable leaves the scrollbar and effects to surrounding components.
#[test]
fn flickable_scrolls_both_axes_without_prescribed_paint() {
    let element = builtin::registry()
        .unwrap()
        .construct(
            builtin::FLICKABLE,
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
            builtin::FLICKABLE,
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
    let schema = registry.schema(builtin::FLICKABLE).unwrap();
    for (name, observation) in [
        ("offset_x", ObservationKind::ScrollX),
        ("offset_y", ObservationKind::ScrollY),
        ("viewport_width", ObservationKind::ViewportWidth),
        ("viewport_height", ObservationKind::ViewportHeight),
        ("content_width", ObservationKind::ContentWidth),
        ("content_height", ObservationKind::ContentHeight),
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
            builtin::FLICKABLE,
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
            builtin::FLICKABLE,
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
