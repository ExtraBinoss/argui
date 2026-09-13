use super::*;
use argui_ui::{SemanticAction, SemanticValue};

fn layout() -> LayoutSnapshot {
    let node = UiTree::new(Element::container([])).node_ids()[0];
    LayoutSnapshot {
        viewport: Rect::default(),
        nodes: ["pad", "hue::track", "alpha::track"]
            .map(|part| LayoutBounds {
                node,
                key: Some(format!("color::{part}")),
                bounds: Rect::new(Point::new(10.0, 20.0), Size::new(200.0, 100.0)),
            })
            .to_vec(),
    }
}

fn pan(target: &str, phase: GesturePhase, x: f32, y: f32) -> UiEvent {
    let node = UiTree::new(Element::container([])).node_ids()[0];
    event(
        target,
        UiEventKind::Gesture(GestureEvent {
            target: node,
            pointer: argui_core::PointerId::MOUSE,
            phase,
            delivery: Default::default(),
            kind: GestureKind::Pan {
                position: Point::new(x, y),
                delta: Point::default(),
                total: Point::default(),
                velocity: Point::default(),
            },
        }),
    )
}

#[test]
fn pad_tracks_layout_captures_drags_clamps_and_restores_cancelled_changes() {
    let mut state = ColorPickerState::new(Color::srgb(1.0, 0.0, 0.0));
    assert!(!state.update(
        "color",
        &pan("color::pad", GesturePhase::Changed, 50.0, 50.0)
    ));
    state.layout_changed("color", &layout());
    let start = state.color();
    let gesture = pan("color::pad", GesturePhase::Started, 110.0, 70.0);
    assert!(state.update("color", &gesture));
    assert!(gesture.default_prevented());
    close(state.color(), [0.5, 0.25, 0.25, 1.0]);
    assert!(state.update(
        "color",
        &pan("color::pad", GesturePhase::Changed, -50.0, -20.0)
    ));
    assert_eq!(state.color(), Color::WHITE);
    assert!(state.update(
        "color",
        &pan("color::pad", GesturePhase::Cancelled, 0.0, 0.0)
    ));
    assert_eq!(state.color(), start);
    assert!(state.update(
        "color",
        &pan("color::pad", GesturePhase::Ended, 500.0, 500.0)
    ));
    assert_eq!(state.color(), Color::BLACK);
    assert!(!state.update(
        "color",
        &pan("color::pad", GesturePhase::Changed, f32::NAN, 70.0)
    ));
    state.layout_changed("color", &LayoutSnapshot::default());
    assert!(!state.update("color", &pan("color::pad", GesturePhase::Ended, 50.0, 50.0)));
}

#[test]
fn keyboard_accessibility_and_format_buttons_offer_full_editing_without_pointer() {
    let mut state = ColorPickerState::new(Color::WHITE);
    for key_value in [
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowDown,
        Key::ArrowUp,
        Key::Home,
        Key::End,
    ] {
        let input = key("color::pad", key_value);
        assert!(state.update("color", &input));
        assert!(input.default_prevented());
    }
    assert_eq!(state.color(), Color::BLACK);
    let mut fine = key("color::pad", Key::ArrowUp);
    if let UiEventKind::KeyInput(input) = &mut fine.kind {
        input.modifiers.shift = true;
    }
    assert!(state.update("color", &fine));
    close(state.color(), [0.001, 0.0, 0.0, 1.0]);
    let set = |part, value| {
        event(
            part,
            UiEventKind::SemanticAction {
                action: SemanticAction::SetValue,
                value: Some(SemanticValue::Number {
                    value,
                    minimum: None,
                    maximum: None,
                    step: None,
                }),
            },
        )
    };
    assert!(state.update("color", &set("color::hue", 120.0)));
    assert!(state.update("color", &set("color::alpha", 50.0)));
    close(state.color(), [0.0, 0.001, 0.0, 0.5]);
    assert!(!state.update("color", &set("color::alpha", f64::NAN)));
    for format in ColorFormat::ALL {
        assert!(state.update(
            "color",
            &event(
                &format!("color::format::{}", format.label()),
                UiEventKind::Click(argui_ui::ClickEvent::accessibility())
            )
        ));
        assert_eq!(state.format(), format);
    }
    for target in [
        "color::format::unknown",
        "color::field::99",
        "other::field::0",
        "colorful::pad",
        "color::unknown",
    ] {
        assert!(!state.update(
            "color",
            &event(target, UiEventKind::TextChanged("0".into()))
        ));
    }
    assert!(!state.update("color", &key("color::pad", Key::Escape)));
}

#[test]
fn disabled_picker_rejects_edits_and_preserves_readable_channels_in_both_schemes() {
    let mut state = ColorPickerState::new(Color::srgb(0.2, 0.4, 0.8));
    state.layout_changed("color", &layout());
    let original = state.color();
    assert!(state.update(
        "color",
        &pan("color::pad", GesturePhase::Started, 20.0, 20.0)
    ));
    state.set_enabled(false);
    assert_eq!(state.color(), original);
    assert!(!state.update("color", &key("color::hue", Key::ArrowRight)));
    assert!(!state.update(
        "color",
        &event("color::field::0", UiEventKind::TextChanged("#fff".into()))
    ));
    let themes = shadcn(Color::BLACK);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let view = ColorPicker::new("color", "Accent", &state).build(themes.resolve(scheme));
        let pad = find(&view, "color::pad").unwrap();
        assert!(!pad.interaction.as_ref().unwrap().enabled);
        assert!(matches!(
            pad.paint.quad.background,
            Some(argui_paint::Fill::Bilinear(_))
        ));
        assert!(
            !find(&view, "color::field::0")
                .unwrap()
                .interaction
                .as_ref()
                .unwrap()
                .enabled
        );
    }
    state.set_enabled(true);
    assert!(state.update("color", &key("color::alpha", Key::ArrowLeft)));
}
