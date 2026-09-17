use argui_core::Transform2D;
use argui_paint::{Border, CornerRadii, QuadStyle};
use argui_ui::{
    AlignItems, ContinuousValuePhase, Element, EventType, LengthPercentageAuto, RangeHandlerValue,
    Sides, StylePatch, StyleTransition, ValueHandler, VisualState, auto, length, percent,
};

use crate::{RangeAxis, RangeBehavior, RangeConfig, RangeDirection, RangePart, WidgetTheme};

#[derive(Clone, Debug)]
pub struct Slider {
    behavior: RangeBehavior,
    change_handlers: Vec<ValueHandler<f32>>,
    commit_handlers: Vec<ValueHandler<f32>>,
}

impl Slider {
    /// Creates a slider with the supplied label, value, bounds and step size.
    /// `key` identifies the control and `config` supplies its numeric range and step settings.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        value: f32,
        config: RangeConfig,
    ) -> Self {
        Self {
            behavior: RangeBehavior::new(key, label, value, config),
            change_handlers: Vec::new(),
            commit_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets whether the slider can be changed; `enabled` controls interaction availability.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.behavior = self.behavior.enabled(enabled);
        self
    }

    /// Adds a callback receiving each next controlled value during interaction.
    #[must_use]
    pub fn on_change(mut self, handler: ValueHandler<f32>) -> Self {
        self.change_handlers.push(handler);
        self
    }

    /// Adds a callback receiving the final controlled value for an interaction.
    #[must_use]
    pub fn on_commit(mut self, handler: ValueHandler<f32>) -> Self {
        self.commit_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the slider using `theme` for its track and thumb.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let ratio = self.behavior.ratio();
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
        let track = self.behavior.decorate(
            RangePart::Track,
            Element::container([fill])
                .width(percent(1.0))
                .height(length(6.0))
                .background(theme.muted)
                .radius(CornerRadii::all(999.0)),
        );
        let mut control = self.behavior.decorate(
            RangePart::Control,
            Element::row([track, thumb])
                .width(percent(1.0))
                .height(length(28.0))
                .align_items(AlignItems::CENTER)
                .when(
                    VisualState::FocusVisible,
                    StylePatch::from_quad(
                        QuadStyle::solid(argui_core::Color::TRANSPARENT)
                            .border(Border::all(2.0, theme.ring))
                            .radius(CornerRadii::all(8.0)),
                    ),
                )
                .transition(StyleTransition::default()),
        );
        if self.behavior.is_enabled() {
            let config = self.behavior.config();
            for (phase, handlers) in [
                (ContinuousValuePhase::Change, &self.change_handlers),
                (ContinuousValuePhase::Commit, &self.commit_handlers),
            ] {
                let source = RangeHandlerValue::new(
                    self.behavior.value(),
                    config.minimum,
                    config.maximum,
                    config.step,
                    config.axis == RangeAxis::Vertical,
                    config.direction == RangeDirection::Reverse,
                    phase,
                );
                for handler in handlers {
                    for event in [
                        EventType::Key,
                        EventType::Gesture,
                        EventType::SemanticAction,
                    ] {
                        control =
                            control.on(handler.direct_listener(event).range_handler_value(source));
                    }
                }
            }
        }
        self.behavior.decorate(RangePart::Root, control)
    }
}
