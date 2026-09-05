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

pub(crate) const fn pointer_button_mask(button: PointerButton) -> u16 {
    match button {
        PointerButton::Primary => 1,
        PointerButton::Secondary => 2,
        PointerButton::Middle => 4,
        PointerButton::Back => 8,
        PointerButton::Forward => 16,
        PointerButton::Other(value) if value < 11 => 1 << (value + 5),
        PointerButton::Other(_) => 0,
    }
}

pub(crate) const fn pointer_phase(phase: winit::event::TouchPhase) -> argui_core::PointerPhase {
    match phase {
        winit::event::TouchPhase::Started => argui_core::PointerPhase::Pressed,
        winit::event::TouchPhase::Moved => argui_core::PointerPhase::Moved,
        winit::event::TouchPhase::Ended => argui_core::PointerPhase::Released,
        winit::event::TouchPhase::Cancelled => argui_core::PointerPhase::Cancelled,
    }
}
