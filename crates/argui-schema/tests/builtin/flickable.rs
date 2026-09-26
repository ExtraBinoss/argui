use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{EffectValue, Filter};
use argui_schema::{
    NativeElementInput, NativeEventValue, ObservationKind, SchemaError, SchemaValue, builtin,
};
use argui_ui::{
    Color, EventHandler, EventHandlerId, EventOwnerId, EventType, Overflow, ScrollAxes,
    ScrollMetrics, ScrollPhysics, ScrollbarSide, ScrollbarVisibility, length,
};

/// ScrollView enables a native scrollbar by default while keeping effects optional.
#[test]
fn scroll_view_scrolls_both_axes_with_a_default_growing_scrollbar() {
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
    let scrollbar = scroll.scrollbar.as_ref().unwrap();
    assert_eq!(scrollbar.width, 4.0);
    assert_eq!(scrollbar.hover_width, Some(8.0));
    assert_eq!(scrollbar.track.base.radii.top_left, 4.0);
    assert_eq!(scrollbar.thumb.base.radii.top_left, 4.0);
    assert_eq!(scrollbar.visibility, ScrollbarVisibility::Always);
    assert_eq!(scrollbar.insets.top, 4.0);
    assert_eq!(scrollbar.insets.bottom, 4.0);
    assert!(scroll.effects.is_empty());
    assert_eq!(element.style.overflow.x, Overflow::Auto);
    assert_eq!(element.style.overflow.y, Overflow::Auto);
}

#[test]
fn scroll_view_fade_tracks_real_overflow_without_an_overlay() {
    let registry = builtin::registry().unwrap();
    let element = registry
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new()
                .property(builtin::SCROLL_SHADOW_WIDTH, SchemaValue::Float(18.0)),
        )
        .unwrap();
    let effects = &element.scroll.as_ref().unwrap().effects;
    assert_eq!(effects.len(), 1);
    let mut metrics = ScrollMetrics {
        viewport: Rect::new(Point::default(), Size::new(256.0, 500.0)),
        transform: Affine2D::IDENTITY,
        offset: Point::default(),
        max_offset: Point::default(),
    };
    assert!(effects[0].resolve(metrics).is_none());
    metrics.max_offset.y = 100.0;
    let edges = |metrics| {
        let layer = effects[0].resolve(metrics).unwrap();
        let Filter::Effect(filter) = &layer.filters[0] else {
            panic!()
        };
        filter.parameters[2].value.clone()
    };
    assert_eq!(edges(metrics), EffectValue::Vec4([0.0, 0.0, 0.0, 1.0]));
    metrics.offset.y = 6.0;
    assert_eq!(edges(metrics), EffectValue::Vec4([0.0, 0.5, 0.0, 1.0]));
    metrics.offset.y = 100.0;
    assert_eq!(edges(metrics), EffectValue::Vec4([0.0, 1.0, 0.0, 0.0]));
    metrics.offset.y = 0.0;
    metrics.max_offset.y = 4.0;
    let EffectValue::Vec4(edges) = edges(metrics) else {
        panic!()
    };
    assert!(edges[3] > 0.0 && edges[3] < 1.0);
    for width in [-1.0, f32::NAN] {
        assert!(
            registry
                .construct(
                    builtin::SCROLL_VIEW,
                    &NativeElementInput::new()
                        .property(builtin::SCROLL_SHADOW_WIDTH, SchemaValue::Float(width))
                )
                .is_err()
        );
    }
}

/// Disabled axes stay clipped and do not receive scroll input.
#[test]
fn flickable_can_disable_scrolling() {
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

/// Applications can suppress automatic scrollbar paint without disabling scroll input.
#[test]
fn scroll_view_can_hide_its_default_scrollbar() {
    let element = builtin::registry()
        .unwrap()
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new()
                .property(builtin::SCROLLBAR_VISIBLE, SchemaValue::Bool(false)),
        )
        .unwrap();
    assert!(element.scroll.as_ref().unwrap().enabled);
    assert!(element.scroll.as_ref().unwrap().scrollbar.is_none());
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
                .property(builtin::SCROLLBAR_END_INSET, SchemaValue::Float(0.0))
                .property(builtin::SCROLLBAR_HOVER_WIDTH, SchemaValue::Float(7.0))
                .property(
                    builtin::SCROLLBAR_TRACK_COLOR,
                    SchemaValue::Color(Color::srgb(0.1, 0.1, 0.1)),
                )
                .property(
                    builtin::SCROLLBAR_PRESSED_COLOR,
                    SchemaValue::Color(Color::srgb(0.9, 0.2, 0.2)),
                )
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
    assert_eq!(bar.insets.top, 0.0);
    assert_eq!(bar.insets.bottom, 0.0);
    assert_eq!(bar.hover_width, Some(7.0));
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
    assert!(
        registry
            .construct(
                builtin::SCROLL_VIEW,
                &NativeElementInput::new()
                    .property(builtin::SCROLLBAR_END_INSET, SchemaValue::Float(-1.0)),
            )
            .is_err()
    );
}

/// Declarative scroll strength selects native direct or bounded inertial physics.
#[test]
fn scroll_momentum_validates_and_selects_native_physics() {
    let registry = builtin::registry().unwrap();
    let direct = registry
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new().property(builtin::SCROLL_MOMENTUM, SchemaValue::Float(0.0)),
        )
        .unwrap();
    assert_eq!(
        direct.scroll.as_ref().unwrap().physics,
        ScrollPhysics::Direct
    );
    let light = registry
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new().property(builtin::SCROLL_MOMENTUM, SchemaValue::Float(0.35)),
        )
        .unwrap();
    assert!(matches!(
        light.scroll.as_ref().unwrap().physics,
        ScrollPhysics::Inertial(config) if config.decay > 10.0
    ));
    for value in [-0.1, 1.1, f32::NAN] {
        assert!(
            registry
                .construct(
                    builtin::SCROLL_VIEW,
                    &NativeElementInput::new()
                        .property(builtin::SCROLL_MOMENTUM, SchemaValue::Float(value)),
                )
                .is_err()
        );
    }
    assert!(
        registry
            .construct(
                builtin::SCROLL_VIEW,
                &NativeElementInput::new()
                    .property(builtin::SCROLLBAR_WIDTH, SchemaValue::Float(8.0))
                    .property(builtin::SCROLLBAR_HOVER_WIDTH, SchemaValue::Float(6.0)),
            )
            .is_err()
    );
}
