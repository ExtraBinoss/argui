use argui_core::{Key, KeyInput, KeyState, Modifiers, Point};

#[path = "../../src/app/zoom.rs"]
mod implementation;

use implementation::{ZoomCommand, magnify_zoom, wheel_zoom};

fn key(value: &str, modifiers: Modifiers) -> KeyInput {
    KeyInput {
        key: Key::Character(value.into()),
        state: KeyState::Pressed,
        modifiers,
        repeat: false,
        text: None,
    }
}

#[test]
fn command_shortcuts_support_control_command_and_keyboard_layout_variants() {
    let control = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    let command = Modifiers {
        super_key: true,
        ..Modifiers::default()
    };

    assert_eq!(
        ZoomCommand::from_key_input(&key("+", control)),
        Some(ZoomCommand::In)
    );
    assert_eq!(
        ZoomCommand::from_key_input(&key("=", control)),
        Some(ZoomCommand::In)
    );
    assert_eq!(
        ZoomCommand::from_key_input(&key("-", command)),
        Some(ZoomCommand::Out)
    );
    assert_eq!(
        ZoomCommand::from_key_input(&key("_", command)),
        Some(ZoomCommand::Out)
    );
    assert_eq!(
        ZoomCommand::from_key_input(&key("0", command)),
        Some(ZoomCommand::Reset)
    );
    assert_eq!(
        ZoomCommand::from_key_input(&key("+", Modifiers::default())),
        None
    );
    assert_eq!(
        ZoomCommand::from_key_input(&key(
            "+",
            Modifiers {
                control: true,
                alt: true,
                ..Modifiers::default()
            }
        )),
        None
    );
    let mut released = key("+", control);
    released.state = KeyState::Released;
    assert_eq!(ZoomCommand::from_key_input(&released), None);
    let mut unrelated = key("x", control);
    unrelated.key = Key::Other;
    assert_eq!(ZoomCommand::from_key_input(&unrelated), None);
}

#[test]
fn keyboard_zoom_uses_stable_bounded_levels() {
    assert_eq!(ZoomCommand::In.apply(1.0), 1.1);
    assert_eq!(ZoomCommand::In.apply(1.24), 1.25);
    assert_eq!(ZoomCommand::Out.apply(1.0), 0.9);
    assert_eq!(ZoomCommand::Out.apply(0.91), 0.9);
    assert_eq!(ZoomCommand::Reset.apply(2.5), 1.0);
    assert_eq!(ZoomCommand::Out.apply(0.5), 0.5);
    assert_eq!(ZoomCommand::In.apply(3.0), 3.0);
}

#[test]
fn wheel_and_native_magnification_are_continuous_bounded_and_ignore_invalid_samples() {
    let wheel = wheel_zoom(1.0, argui_core::ScrollDelta::Lines(Point::new(0.0, 1.0))).unwrap();
    assert!(wheel > 1.1 && wheel < 1.11);
    assert_eq!(
        wheel_zoom(3.0, argui_core::ScrollDelta::Pixels(Point::new(0.0, 120.0)),),
        None
    );
    assert_eq!(
        wheel_zoom(
            1.0,
            argui_core::ScrollDelta::Pixels(Point::new(0.0, f32::NAN)),
        ),
        None
    );
    assert_eq!(magnify_zoom(1.0, 0.25), Some(1.25));
    assert_eq!(magnify_zoom(0.5, -0.5), None);
    assert_eq!(magnify_zoom(1.0, f64::NAN), None);
}
