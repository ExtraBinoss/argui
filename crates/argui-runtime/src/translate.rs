use argui_core::Point;
use argui_platform::{
    ButtonState, ImeInput, Key, KeyInput, KeyState, Modifiers, PointerButton, ScrollDelta,
};
use winit::{
    event::{ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
};

pub(crate) fn scroll_delta(delta: MouseScrollDelta, scale_factor: f32) -> ScrollDelta {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => ScrollDelta::Lines(Point::new(x, y)),
        MouseScrollDelta::PixelDelta(position) => ScrollDelta::Pixels(Point::new(
            position.x as f32 / scale_factor,
            position.y as f32 / scale_factor,
        )),
    }
}

pub(crate) fn key_input(event: KeyEvent, modifiers: Modifiers) -> KeyInput {
    KeyInput {
        key: match event.logical_key {
            WinitKey::Character(value) => Key::Character(value.to_string()),
            WinitKey::Named(named) => named_key(named),
            _ => Key::Other,
        },
        state: match event.state {
            ElementState::Pressed => KeyState::Pressed,
            ElementState::Released => KeyState::Released,
        },
        modifiers,
        repeat: event.repeat,
        text: event.text.map(|text| text.to_string()),
    }
}

const fn named_key(key: NamedKey) -> Key {
    match key {
        NamedKey::ArrowLeft => Key::ArrowLeft,
        NamedKey::ArrowRight => Key::ArrowRight,
        NamedKey::ArrowUp => Key::ArrowUp,
        NamedKey::ArrowDown => Key::ArrowDown,
        NamedKey::Home => Key::Home,
        NamedKey::End => Key::End,
        NamedKey::Backspace => Key::Backspace,
        NamedKey::Delete => Key::Delete,
        NamedKey::Enter => Key::Enter,
        NamedKey::Tab => Key::Tab,
        NamedKey::Escape => Key::Escape,
        _ => Key::Other,
    }
}

pub(crate) fn modifiers_state(state: ModifiersState) -> Modifiers {
    Modifiers {
        shift: state.shift_key(),
        control: state.control_key(),
        alt: state.alt_key(),
        super_key: state.super_key(),
    }
}

pub(crate) fn ime_input(input: Ime) -> ImeInput {
    match input {
        Ime::Enabled => ImeInput::Enabled,
        Ime::Disabled => ImeInput::Disabled,
        Ime::Preedit(text, cursor) => ImeInput::Preedit { text, cursor },
        Ime::Commit(text) => ImeInput::Commit(text),
    }
}

pub(crate) const fn button_state(state: ElementState) -> ButtonState {
    match state {
        ElementState::Pressed => ButtonState::Pressed,
        ElementState::Released => ButtonState::Released,
    }
}

pub(crate) const fn pointer_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Back => PointerButton::Back,
        MouseButton::Forward => PointerButton::Forward,
        MouseButton::Other(value) => PointerButton::Other(value),
    }
}

#[cfg(test)]
mod tests {
    use winit::{dpi::PhysicalPosition, keyboard::ModifiersState};

    use super::*;

    #[test]
    fn platform_inputs_are_normalized_without_losing_units() {
        assert_eq!(
            scroll_delta(MouseScrollDelta::LineDelta(1.0, -2.0), 2.0),
            ScrollDelta::Lines(Point::new(1.0, -2.0))
        );
        assert_eq!(
            scroll_delta(
                MouseScrollDelta::PixelDelta(PhysicalPosition::new(20.0, -10.0)),
                2.0,
            ),
            ScrollDelta::Pixels(Point::new(10.0, -5.0))
        );
        let modifiers = modifiers_state(
            ModifiersState::SHIFT
                | ModifiersState::CONTROL
                | ModifiersState::ALT
                | ModifiersState::SUPER,
        );
        assert_eq!(
            modifiers,
            Modifiers {
                shift: true,
                control: true,
                alt: true,
                super_key: true,
            }
        );
    }

    #[test]
    fn named_keys_buttons_and_ime_have_explicit_mappings() {
        for (named, expected) in [
            (NamedKey::ArrowLeft, Key::ArrowLeft),
            (NamedKey::ArrowRight, Key::ArrowRight),
            (NamedKey::ArrowUp, Key::ArrowUp),
            (NamedKey::ArrowDown, Key::ArrowDown),
            (NamedKey::Home, Key::Home),
            (NamedKey::End, Key::End),
            (NamedKey::Backspace, Key::Backspace),
            (NamedKey::Delete, Key::Delete),
            (NamedKey::Enter, Key::Enter),
            (NamedKey::Tab, Key::Tab),
            (NamedKey::Escape, Key::Escape),
            (NamedKey::F1, Key::Other),
        ] {
            assert_eq!(named_key(named), expected);
        }
        assert_eq!(button_state(ElementState::Pressed), ButtonState::Pressed);
        assert_eq!(button_state(ElementState::Released), ButtonState::Released);
        for (button, expected) in [
            (MouseButton::Left, PointerButton::Primary),
            (MouseButton::Right, PointerButton::Secondary),
            (MouseButton::Middle, PointerButton::Middle),
            (MouseButton::Back, PointerButton::Back),
            (MouseButton::Forward, PointerButton::Forward),
            (MouseButton::Other(9), PointerButton::Other(9)),
        ] {
            assert_eq!(pointer_button(button), expected);
        }
        assert_eq!(ime_input(Ime::Enabled), ImeInput::Enabled);
        assert_eq!(ime_input(Ime::Disabled), ImeInput::Disabled);
        assert_eq!(
            ime_input(Ime::Preedit("é".into(), Some((0, 2)))),
            ImeInput::Preedit {
                text: "é".into(),
                cursor: Some((0, 2)),
            }
        );
        assert_eq!(
            ime_input(Ime::Commit("é".into())),
            ImeInput::Commit("é".into())
        );
    }
}
