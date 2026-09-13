use crate::{WidgetTheme, shadcn};
use argui_animation::Frame;
use argui_paint::CornerRadii;
use argui_runtime::{Context, Render};
use argui_ui::{
    Element, Position, Role, SemanticState, SemanticValue, Semantics, Sides, auto, length, percent,
};

/// Percentage progress. `None` means indeterminate; non-finite values are also indeterminate.
/// `build` draws a static indicator; mount as an entity for indeterminate motion.
#[derive(Clone, Debug)]
pub struct Progress {
    key: String,
    label: String,
    value: Option<f32>,
    phase: f32,
    reduced_motion: bool,
}

impl Progress {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: Option<f32>) -> Self {
        let mut progress = Self {
            key: key.into(),
            label: label.into(),
            value: None,
            phase: 0.0,
            reduced_motion: false,
        };
        progress.set_value(value);
        progress
    }

    pub fn set_value(&mut self, value: Option<f32>) {
        self.value = value
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(0.0, 100.0));
    }

    #[must_use]
    pub const fn value(&self) -> Option<f32> {
        self.value
    }

    #[must_use]
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let (width, offset) = self.value.map_or_else(
            || {
                let offset = if self.reduced_motion {
                    0.35
                } else {
                    (1.0 - (self.phase * std::f32::consts::TAU).cos()) * 0.35
                };
                (0.3, offset)
            },
            |value| (value / 100.0, 0.0),
        );
        let indicator = Element::container([])
            .keyed(format!("{}::indicator", self.key))
            .width(percent(width))
            .height(percent(1.0))
            .background(theme.primary)
            .radius(CornerRadii::all(999.0))
            .semantic_hidden(true)
            .absolute(Sides {
                left: percent(offset),
                right: auto(),
                top: length(0.0),
                bottom: auto(),
            });
        let mut semantics = Semantics::new(Role::Progress)
            .label(self.label.clone())
            .state(SemanticState {
                busy: self.value.is_none_or(|value| value < 100.0),
                ..SemanticState::default()
            });
        if let Some(value) = self.value {
            semantics = semantics.value(SemanticValue::Number {
                value: f64::from(value),
                minimum: Some(0.0),
                maximum: Some(100.0),
                step: None,
            });
        }
        Element::container([indicator])
            .keyed(self.key.clone())
            .semantics(semantics)
            .width(percent(1.0))
            .height(length(8.0))
            .min_width(length(0.0))
            .shrink(0.0)
            .background(theme.secondary)
            .position(Position::Relative)
            .clip(CornerRadii::all(999.0))
    }
}

impl Render for Progress {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.reduced_motion = cx.environment().reduced_motion;
        let themes = shadcn(cx.environment());
        self.build(themes.resolve(cx.environment().color_scheme))
    }

    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        if self.wants_animation_frame() {
            self.phase = (self.phase + frame.elapsed.as_secs_f64() as f32 / 1.6) % 1.0;
            cx.notify();
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.value.is_none() && !self.reduced_motion
    }
}
