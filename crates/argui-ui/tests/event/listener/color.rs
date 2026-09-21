use argui_core::{Key, KeyInput, KeyState, Modifiers};
use argui_ui::{
    ColorHandlerValue, ColorValueFormat, ContinuousValuePhase, Element, EventHandlerId,
    EventListener, EventOwnerId, HandlerValue, UiEventKind, UiTree,
};

/// Resolves a color adapter through normal UI event delivery.
///
/// `source` describes the controlled picker channel and `event` is the user
/// input. The return value is absent when the adapter rejects that input.
fn delivered_color(source: ColorHandlerValue, event: UiEventKind) -> Option<[f32; 4]> {
    let listener = EventListener::new(event.event_type(), EventHandlerId::new(EventOwnerId(1), 0))
        .color_handler_value(source);
    let mut tree = UiTree::new(Element::container([]).on(listener));
    let target = tree.node_id_at(0)?;
    let events = tree.event_deliveries(target, event);
    match events.first()?.handler_value()? {
        HandlerValue::Color(color) => Some(color.to_srgba()),
        _ => None,
    }
}

/// Builds one pressed key event for a color picker channel.
///
/// `key` selects the direction or endpoint and `shift` requests the fine step.
fn key_event(key: Key, shift: bool) -> UiEventKind {
    UiEventKind::KeyInput(KeyInput {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            shift,
            ..Modifiers::default()
        },
        repeat: false,
        text: None,
    })
}

/// Asserts an RGBA result while allowing color-space rounding.
///
/// `actual` is the dispatched color and `expected` is its sRGBA target.
fn assert_color(actual: Option<[f32; 4]>, expected: [f32; 4]) {
    let actual = actual.expect("color event should be delivered");
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!((actual - expected).abs() < 0.01, "{actual} != {expected}");
    }
}

#[test]
fn text_fields_map_hex_rgb_hsv_hsl_and_alpha_channels() {
    let red = [0.0, 1.0, 1.0];
    let field = |format, index, text: &str| {
        delivered_color(
            ColorHandlerValue::field(red, 1.0, format, index),
            UiEventKind::TextChanged(text.into()),
        )
    };
    assert_color(
        field(ColorValueFormat::Hex, 0, " #00ff00 "),
        [0.0, 1.0, 0.0, 1.0],
    );
    assert_color(field(ColorValueFormat::Rgb, 2, "255"), [1.0, 0.0, 1.0, 1.0]);
    assert_color(field(ColorValueFormat::Hsv, 0, "120"), [0.0, 1.0, 0.0, 1.0]);
    assert_color(field(ColorValueFormat::Hsl, 0, "120"), [0.0, 1.0, 0.0, 1.0]);
    assert_color(field(ColorValueFormat::Rgb, 3, "50"), [1.0, 0.0, 0.0, 0.5]);
    assert!(field(ColorValueFormat::Hex, 0, "not a color").is_none());
    assert!(field(ColorValueFormat::Rgb, 0, "256").is_none());
    assert!(field(ColorValueFormat::Rgb, 4, "20").is_none());
    assert!(field(ColorValueFormat::Hsv, 0, "NaN").is_none());
    assert!(field(ColorValueFormat::Hsl, 3, "101").is_none());
}

#[test]
fn pad_and_track_keyboard_values_cover_endpoints_and_fine_steps() {
    let red = [0.0, 0.5, 0.5];
    assert_color(
        delivered_color(
            ColorHandlerValue::pad(red, 1.0),
            key_event(Key::Home, false),
        ),
        [1.0, 1.0, 1.0, 1.0],
    );
    assert_color(
        delivered_color(ColorHandlerValue::pad(red, 1.0), key_event(Key::End, false)),
        [0.0, 0.0, 0.0, 1.0],
    );
    let fine = delivered_color(
        ColorHandlerValue::pad(red, 1.0),
        key_event(Key::ArrowRight, true),
    )
    .expect("fine saturation step");
    assert!(fine[0] > fine[1]);
    assert!(fine[1] > 0.249 && fine[1] < 0.251);
    assert_color(
        delivered_color(
            ColorHandlerValue::hue(red, 1.0, ContinuousValuePhase::Change),
            key_event(Key::ArrowRight, false),
        ),
        [0.5, 0.254, 0.25, 1.0],
    );
    assert_color(
        delivered_color(
            ColorHandlerValue::alpha(red, 0.5, ContinuousValuePhase::Change),
            key_event(Key::ArrowUp, false),
        ),
        [0.5, 0.25, 0.25, 0.51],
    );
    assert!(
        delivered_color(
            ColorHandlerValue::pad(red, 1.0),
            key_event(Key::Enter, false)
        )
        .is_none()
    );
}
