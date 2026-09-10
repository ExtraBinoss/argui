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
            WinitKey::Named(NamedKey::Space) => Key::Character(" ".into()),
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
        NamedKey::ContextMenu => Key::ContextMenu,
        NamedKey::F1 => Key::Function(1),
        NamedKey::F2 => Key::Function(2),
        NamedKey::F3 => Key::Function(3),
        NamedKey::F4 => Key::Function(4),
        NamedKey::F5 => Key::Function(5),
        NamedKey::F6 => Key::Function(6),
        NamedKey::F7 => Key::Function(7),
        NamedKey::F8 => Key::Function(8),
        NamedKey::F9 => Key::Function(9),
        NamedKey::F10 => Key::Function(10),
        NamedKey::F11 => Key::Function(11),
        NamedKey::F12 => Key::Function(12),
        NamedKey::F13 => Key::Function(13),
        NamedKey::F14 => Key::Function(14),
        NamedKey::F15 => Key::Function(15),
        NamedKey::F16 => Key::Function(16),
        NamedKey::F17 => Key::Function(17),
        NamedKey::F18 => Key::Function(18),
        NamedKey::F19 => Key::Function(19),
        NamedKey::F20 => Key::Function(20),
        NamedKey::F21 => Key::Function(21),
        NamedKey::F22 => Key::Function(22),
        NamedKey::F23 => Key::Function(23),
        NamedKey::F24 => Key::Function(24),
        NamedKey::F25 => Key::Function(25),
        NamedKey::F26 => Key::Function(26),
        NamedKey::F27 => Key::Function(27),
        NamedKey::F28 => Key::Function(28),
        NamedKey::F29 => Key::Function(29),
        NamedKey::F30 => Key::Function(30),
        NamedKey::F31 => Key::Function(31),
        NamedKey::F32 => Key::Function(32),
        NamedKey::F33 => Key::Function(33),
        NamedKey::F34 => Key::Function(34),
        NamedKey::F35 => Key::Function(35),

        NamedKey::ArrowLeft => Key::ArrowLeft,
        NamedKey::ArrowRight => Key::ArrowRight,
        NamedKey::ArrowUp => Key::ArrowUp,
        NamedKey::ArrowDown => Key::ArrowDown,
        NamedKey::PageUp => Key::PageUp,
        NamedKey::PageDown => Key::PageDown,
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
