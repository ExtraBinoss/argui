use crate::{SemanticAction, SemanticValue, UiEvent, UiEventKind};

use super::ColorHandlerValue;

/// Delivery stage selected by a continuous-value widget handler.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuousValuePhase {
    Change,
    Commit,
}

/// Pure value mapping used by range-like widgets inside normal event dispatch.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeHandlerValue {
    current: f32,
    minimum: f32,
    maximum: f32,
    step: f32,
    vertical: bool,
    reverse: bool,
    phase: ContinuousValuePhase,
}

impl RangeHandlerValue {
    /// Creates a range value mapping for one controlled value and delivery phase.
    #[doc(hidden)]
    #[must_use]
    pub fn new(
        current: f32,
        minimum: f32,
        maximum: f32,
        step: f32,
        vertical: bool,
        reverse: bool,
        phase: ContinuousValuePhase,
    ) -> Self {
        let lower = minimum.min(maximum);
        let upper = minimum.max(maximum);
        Self {
            current: current.clamp(lower, upper),
            minimum: lower,
            maximum: upper,
            step: step.abs().max(f32::EPSILON),
            vertical,
            reverse,
            phase,
        }
    }

    fn clamp(self, value: f32) -> f32 {
        let stepped = ((value - self.minimum) / self.step).round() * self.step + self.minimum;
        stepped.clamp(self.minimum, self.maximum)
    }

    pub(super) fn event_value(
        self,
        kind: &UiEventKind,
        bounds: Option<argui_core::Rect>,
    ) -> Option<f32> {
        match kind {
            UiEventKind::KeyInput(input) => self.key_value(input),
            UiEventKind::SemanticAction { action, value } => self.semantic_value(*action, value),
            UiEventKind::Gesture(gesture) => self.gesture_value(*gesture, bounds),
            _ => None,
        }
    }

    fn key_value(self, input: &argui_core::KeyInput) -> Option<f32> {
        let delta = match input.key {
            argui_core::Key::ArrowLeft | argui_core::Key::ArrowDown => -self.step,
            argui_core::Key::ArrowRight | argui_core::Key::ArrowUp => self.step,
            argui_core::Key::Home => self.minimum - self.current,
            argui_core::Key::End => self.maximum - self.current,
            _ => return None,
        };
        match self.phase {
            ContinuousValuePhase::Change if input.state == argui_core::KeyState::Pressed => {
                Some(self.clamp(self.current + delta))
            }
            ContinuousValuePhase::Commit if input.state == argui_core::KeyState::Released => {
                Some(self.current)
            }
            _ => None,
        }
    }

    fn semantic_value(self, action: SemanticAction, value: &Option<SemanticValue>) -> Option<f32> {
        if self.phase != ContinuousValuePhase::Commit {
            return None;
        }
        let value = match action {
            SemanticAction::Increment => self.current + self.step,
            SemanticAction::Decrement => self.current - self.step,
            SemanticAction::SetValue => match value {
                Some(SemanticValue::Number { value, .. }) => *value as f32,
                _ => return None,
            },
            _ => return None,
        };
        Some(self.clamp(value))
    }

    fn gesture_value(
        self,
        gesture: crate::GestureEvent,
        bounds: Option<argui_core::Rect>,
    ) -> Option<f32> {
        let is_commit = gesture.phase == crate::GesturePhase::Ended;
        if (self.phase == ContinuousValuePhase::Commit) != is_commit
            || gesture.phase == crate::GesturePhase::Cancelled
        {
            return None;
        }
        let position = match gesture.kind {
            crate::GestureKind::Tap { position } | crate::GestureKind::Pan { position, .. } => {
                position
            }
            _ => return None,
        };
        let bounds = bounds?;
        let ratio = if self.vertical {
            (position.y - bounds.origin.y) / bounds.size.height.max(f32::EPSILON)
        } else {
            (position.x - bounds.origin.x) / bounds.size.width.max(f32::EPSILON)
        }
        .clamp(0.0, 1.0);
        let ratio = if self.reverse { 1.0 - ratio } else { ratio };
        Some(self.clamp(self.minimum + ratio * (self.maximum - self.minimum)))
    }
}

/// Pure split-pane size mapping used inside normal event dispatch.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitHandlerValue {
    current: f32,
    minimum: f32,
    maximum: f32,
    reset: f32,
    horizontal: bool,
    trailing: bool,
}

impl SplitHandlerValue {
    /// Creates a split size adapter from controlled bounds and orientation.
    #[must_use]
    pub fn new(
        current: f32,
        minimum: f32,
        maximum: f32,
        reset: f32,
        horizontal: bool,
        trailing: bool,
    ) -> Self {
        Self {
            current,
            minimum,
            maximum,
            reset,
            horizontal,
            trailing,
        }
    }

    fn event_value(self, kind: &UiEventKind) -> Option<f32> {
        let direction = if self.trailing { -1.0 } else { 1.0 };
        let value = match kind {
            UiEventKind::Gesture(gesture) => {
                let crate::GestureKind::Pan { delta, .. } = gesture.kind else {
                    return None;
                };
                if gesture.phase != crate::GesturePhase::Changed {
                    return None;
                }
                // Controlled views can rebuild between coalesced pan samples. Applying the
                // accumulated frame delta to the latest controlled value avoids counting `total`
                // again every time the handler source is rebuilt. Gesture boundaries carry no
                // value so an Ended event cannot overwrite a Changed event flushed beside it.
                self.current + (if self.horizontal { delta.x } else { delta.y }) * direction
            }
            UiEventKind::Click(click) if click.count >= 2 => self.reset,
            UiEventKind::KeyInput(input) if input.state == argui_core::KeyState::Pressed => {
                let step = if input.modifiers.shift { 1.0 } else { 10.0 };
                match (&input.key, self.horizontal) {
                    (argui_core::Key::ArrowLeft, true) | (argui_core::Key::ArrowUp, false) => {
                        self.current - step * direction
                    }
                    (argui_core::Key::ArrowRight, true) | (argui_core::Key::ArrowDown, false) => {
                        self.current + step * direction
                    }
                    (argui_core::Key::Home, _) => self.minimum,
                    (argui_core::Key::End, _) => self.maximum,
                    _ => return None,
                }
            }
            _ => return None,
        };
        value
            .is_finite()
            .then(|| value.clamp(self.minimum, self.maximum))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HandlerValueSource {
    Static(HandlerValue),
    Range(RangeHandlerValue),
    Color(ColorHandlerValue),
    Split(SplitHandlerValue),
}

impl HandlerValueSource {
    pub(crate) fn resolve(
        &self,
        kind: &UiEventKind,
        bounds: Option<argui_core::Rect>,
    ) -> Option<HandlerValue> {
        match self {
            Self::Static(value) => Some(value.clone()),
            Self::Range(range) => range.event_value(kind, bounds).map(HandlerValue::Number),
            Self::Color(color) => color.event_value(kind, bounds).map(HandlerValue::Color),
            Self::Split(split) => split.event_value(kind).map(HandlerValue::Number),
        }
    }
}

/// Typed value carried from a widget listener to a runtime callback.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub enum HandlerValue {
    Bool(bool),
    Checked(crate::CheckedState),
    Color(argui_core::Color),
    Index(usize),
    IndexList(Vec<usize>),
    Number(f32),
    Text(String),
    TextList(Vec<String>),
}

impl From<bool> for HandlerValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<crate::CheckedState> for HandlerValue {
    fn from(value: crate::CheckedState) -> Self {
        Self::Checked(value)
    }
}

impl From<argui_core::Color> for HandlerValue {
    fn from(value: argui_core::Color) -> Self {
        Self::Color(value)
    }
}

impl From<usize> for HandlerValue {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

impl From<Vec<usize>> for HandlerValue {
    fn from(value: Vec<usize>) -> Self {
        Self::IndexList(value)
    }
}

impl From<f32> for HandlerValue {
    fn from(value: f32) -> Self {
        Self::Number(value)
    }
}

impl From<String> for HandlerValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for HandlerValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<Vec<String>> for HandlerValue {
    fn from(value: Vec<String>) -> Self {
        Self::TextList(value)
    }
}

/// Converts the current widget delivery into its typed callback value.
pub trait FromHandlerValue: Sized + 'static {
    /// Returns the typed value for `event`, or `None` when incompatible.
    #[doc(hidden)]
    fn from_handler_event(event: &UiEvent) -> Option<Self>;
}

macro_rules! from_value {
    ($ty:ty, $variant:ident) => {
        impl FromHandlerValue for $ty {
            fn from_handler_event(event: &UiEvent) -> Option<Self> {
                match event.handler_value()? {
                    HandlerValue::$variant(value) => Some(value.clone()),
                    _ => None,
                }
            }
        }
    };
}

from_value!(bool, Bool);
from_value!(crate::CheckedState, Checked);
from_value!(argui_core::Color, Color);
from_value!(usize, Index);
from_value!(Vec<usize>, IndexList);
from_value!(f32, Number);
from_value!(Vec<String>, TextList);

impl FromHandlerValue for String {
    fn from_handler_event(event: &UiEvent) -> Option<Self> {
        match event.handler_value() {
            Some(HandlerValue::Text(value)) => Some(value.clone()),
            _ => match &event.kind {
                UiEventKind::TextChanged(value) | UiEventKind::Submitted(value) => {
                    Some(value.clone())
                }
                _ => None,
            },
        }
    }
}
