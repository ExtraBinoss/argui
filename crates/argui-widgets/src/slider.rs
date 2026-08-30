use argui_core::{Key, KeyState, Rect, Transform2D};
use argui_paint::{Border, CornerRadii, QuadStyle};
use argui_runtime::LayoutSnapshot;
use argui_ui::{
    AlignItems, CursorIcon, Element, GestureKind, GesturePhase, GestureSet, Interaction,
    LengthPercentageAuto, Role, SemanticAction, SemanticValue, Semantics, Sides, StateStyle,
    StyleTransition, UiEvent, UiEventKind, VisualState, auto, length, percent,
};

use crate::WidgetTheme;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderConfig {
    pub minimum: f32,
    pub maximum: f32,
    pub step: f32,
}

impl SliderConfig {
    #[must_use]
    pub const fn new(minimum: f32, maximum: f32, step: f32) -> Self {
        Self {
            minimum,
            maximum,
            step,
        }
    }

    fn normalized(self) -> Self {
        let minimum = self.minimum.min(self.maximum);
        let maximum = self.maximum.max(self.minimum);
        Self {
            minimum,
            maximum,
            step: self.step.abs().max(f32::EPSILON),
        }
    }

    #[must_use]
    pub fn clamp(self, value: f32) -> f32 {
        let config = self.normalized();
        let stepped =
            ((value - config.minimum) / config.step).round() * config.step + config.minimum;
        stepped.max(config.minimum).min(config.maximum)
    }

    fn ratio(self, value: f32) -> f32 {
        let config = self.normalized();
        let range = config.maximum - config.minimum;
        if range <= f32::EPSILON {
            0.0
        } else {
            ((config.clamp(value) - config.minimum) / range).clamp(0.0, 1.0)
        }
    }
}

impl Default for SliderConfig {
    fn default() -> Self {
        Self::new(0.0, 100.0, 1.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderState {
    bounds: Option<Rect>,
    drag_start: Option<f32>,
}

impl SliderState {
    pub fn layout_changed(&mut self, layout: &LayoutSnapshot, key: &str) {
        self.bounds = layout.bounds(key);
    }

    #[must_use]
    pub fn update(
        &mut self,
        event: &UiEvent,
        key: &str,
        value: f32,
        config: SliderConfig,
    ) -> Option<f32> {
        if event.key.as_deref() != Some(key) {
            return None;
        }
        let config = config.normalized();
        match &event.kind {
            UiEventKind::KeyInput(input) if input.state == KeyState::Pressed => match input.key {
                Key::ArrowLeft | Key::ArrowDown => Some(config.clamp(value - config.step)),
                Key::ArrowRight | Key::ArrowUp => Some(config.clamp(value + config.step)),
                Key::Home => Some(config.minimum),
                Key::End => Some(config.maximum),
                _ => None,
            },
            UiEventKind::SemanticAction { action, value: set } => match action {
                SemanticAction::Increment => Some(config.clamp(value + config.step)),
                SemanticAction::Decrement => Some(config.clamp(value - config.step)),
                SemanticAction::SetValue => match set {
                    Some(SemanticValue::Number { value, .. }) => Some(config.clamp(*value as f32)),
                    _ => None,
                },
                _ => None,
            },
            UiEventKind::Gesture(gesture) => match gesture.kind {
                GestureKind::Tap { position } => self.value_at(position.x, config),
                GestureKind::Pan { total, .. } => match gesture.phase {
                    GesturePhase::Started => {
                        self.drag_start = Some(value);
                        self.drag_value(total.x, config)
                    }
                    GesturePhase::Changed => self.drag_value(total.x, config),
                    GesturePhase::Ended | GesturePhase::Cancelled => {
                        let next = self.drag_value(total.x, config);
                        self.drag_start = None;
                        next
                    }
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn value_at(self, x: f32, config: SliderConfig) -> Option<f32> {
        let bounds = self.bounds?;
        if bounds.size.width <= f32::EPSILON {
            return None;
        }
        let ratio = ((x - bounds.origin.x) / bounds.size.width).clamp(0.0, 1.0);
        Some(config.clamp(config.minimum + ratio * (config.maximum - config.minimum)))
    }

    fn drag_value(self, delta: f32, config: SliderConfig) -> Option<f32> {
        let bounds = self.bounds?;
        let start = self.drag_start?;
        if bounds.size.width <= f32::EPSILON {
            return None;
        }
        Some(config.clamp(start + delta / bounds.size.width * (config.maximum - config.minimum)))
    }
}

#[derive(Clone, Debug)]
pub struct Slider {
    key: String,
    label: String,
    value: f32,
    config: SliderConfig,
    enabled: bool,
}

impl Slider {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        value: f32,
        config: SliderConfig,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value,
            config,
            enabled: true,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let config = self.config.normalized();
        let value = config.clamp(self.value);
        let ratio = config.ratio(value);
        let fill = Element::container([])
            .width(percent(ratio))
            .height(length(6.0))
            .background(theme.primary)
            .radius(CornerRadii::all(999.0));
        let thumb = Element::container([])
            .width(length(16.0))
            .height(length(16.0))
            .background(theme.card)
            .border(Border::all(2.0, theme.primary))
            .radius(CornerRadii::all(999.0))
            .absolute(Sides {
                left: LengthPercentageAuto::percent(ratio),
                right: auto(),
                top: LengthPercentageAuto::percent(0.5),
                bottom: auto(),
            })
            .transform(Transform2D::IDENTITY.translate(-8.0, -8.0));
        let track = Element::container([fill])
            .width(percent(1.0))
            .height(length(6.0))
            .background(theme.muted)
            .radius(CornerRadii::all(999.0));
        Element::row([track, thumb])
            .keyed(self.key)
            .width(percent(1.0))
            .height(length(28.0))
            .align_items(AlignItems::CENTER)
            .interaction(
                Interaction::default()
                    .enabled(self.enabled)
                    .focusable(self.enabled)
                    .cursor(CursorIcon::Pointer)
                    .gestures(GestureSet::NONE.tap().pan()),
            )
            .state(
                VisualState::Focused,
                StateStyle::from_quad(
                    QuadStyle::solid(argui_core::Color::TRANSPARENT)
                        .border(Border::all(2.0, theme.ring))
                        .radius(CornerRadii::all(8.0)),
                ),
            )
            .transition(StyleTransition::default())
            .semantics(
                Semantics::new(Role::Slider)
                    .label(self.label)
                    .value(SemanticValue::Number {
                        value: f64::from(value),
                        minimum: Some(f64::from(config.minimum)),
                        maximum: Some(f64::from(config.maximum)),
                        step: Some(f64::from(config.step)),
                    })
                    .action(SemanticAction::Focus)
                    .action(SemanticAction::Increment)
                    .action(SemanticAction::Decrement)
                    .action(SemanticAction::SetValue),
            )
    }
}
