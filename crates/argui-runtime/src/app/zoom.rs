use argui_core::{Key, KeyInput, KeyState, ScrollDelta};

const LEVELS: [f32; 13] = [
    0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0,
];
const DEFAULT_ZOOM: f32 = 1.0;

/// Computes a bounded zoom factor from a Control/Command wheel gesture.
///
/// `current` is the active factor and `delta` follows Argui's positive-up
/// scroll convention. The return value is `None` for a zero or invalid delta.
pub(crate) fn wheel_zoom(current: f32, delta: ScrollDelta) -> Option<f32> {
    let amount = match delta {
        ScrollDelta::Lines(point) => point.y * 0.1,
        ScrollDelta::Pixels(point) => point.y * 0.002,
    };
    amount
        .is_finite()
        .then(|| (current * amount.exp()).clamp(LEVELS[0], LEVELS[LEVELS.len() - 1]))
        .filter(|next| (*next - current).abs() >= f32::EPSILON)
}

/// Computes a bounded factor from an incremental native magnification sample.
///
/// `current` is the active factor and `delta` is positive when magnifying. The
/// return value is `None` for a zero or invalid sample.
pub(crate) fn magnify_zoom(current: f32, delta: f64) -> Option<f32> {
    let delta = delta as f32;
    delta
        .is_finite()
        .then(|| (current * (1.0 + delta)).clamp(LEVELS[0], LEVELS[LEVELS.len() - 1]))
        .filter(|next| (*next - current).abs() >= f32::EPSILON)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ZoomCommand {
    In,
    Out,
    Reset,
}

impl ZoomCommand {
    /// Recognizes a runtime-owned UI zoom command from a normalized key event.
    ///
    /// `input` contains the logical key, transition, and active modifiers. The
    /// return value is `None` when the event should remain application-owned.
    pub(crate) fn from_key_input(input: &KeyInput) -> Option<Self> {
        if input.state != KeyState::Pressed || !input.modifiers.command() || input.modifiers.alt {
            return None;
        }
        match &input.key {
            Key::Character(value) if matches!(value.as_str(), "+" | "=") => Some(Self::In),
            Key::Character(value) if matches!(value.as_str(), "-" | "_") => Some(Self::Out),
            Key::Character(value) if value == "0" => Some(Self::Reset),
            _ => None,
        }
    }

    /// Resolves this command to the next bounded zoom factor.
    ///
    /// `current` is the active UI zoom. The return value is one of the stable
    /// accessibility zoom levels between 50% and 300%.
    pub(crate) fn apply(self, current: f32) -> f32 {
        match self {
            Self::In => LEVELS
                .iter()
                .copied()
                .find(|level| *level > current + f32::EPSILON)
                .unwrap_or(LEVELS[LEVELS.len() - 1]),
            Self::Out => LEVELS
                .iter()
                .rev()
                .copied()
                .find(|level| *level < current - f32::EPSILON)
                .unwrap_or(LEVELS[0]),
            Self::Reset => DEFAULT_ZOOM,
        }
    }
}
