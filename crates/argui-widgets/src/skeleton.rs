use argui_animation::Frame;
use argui_paint::CornerRadii;
use argui_runtime::{Context, Render};
use argui_ui::{Dimension, Element, length, percent};

use crate::{WidgetTheme, shadcn};

/// Decorative loading shape. `build` is static; mount an entity for a gentle pulse.
/// Announce loading once on the containing region, rather than on every shape.
#[derive(Clone, Debug)]
pub struct Skeleton {
    key: String,
    width: Dimension,
    height: Dimension,
    radius: f32,
    animated: bool,
    reduced_motion: bool,
    phase: f32,
}

impl Skeleton {
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            width: percent(1.0),
            height: length(16.0),
            radius: 6.0,
            animated: true,
            reduced_motion: false,
            phase: 0.0,
        }
    }

    #[must_use]
    pub fn size(mut self, width: Dimension, height: Dimension) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    #[must_use]
    pub fn radius(mut self, radius: f32) -> Self {
        assert!(
            radius.is_finite() && radius >= 0.0,
            "skeleton radius must be finite and nonnegative"
        );
        self.radius = radius;
        self
    }

    pub fn set_animated(&mut self, animated: bool) {
        self.animated = animated;
    }

    #[must_use]
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let opacity = if self.wants_animation_frame() {
            0.75 + 0.25 * (self.phase * std::f32::consts::TAU).cos()
        } else {
            1.0
        };
        Element::container([])
            .keyed(self.key.clone())
            .width(self.width)
            .height(self.height)
            .min_width(length(0.0))
            .shrink(0.0)
            .background(theme.muted)
            .radius(CornerRadii::all(self.radius))
            .paint_opacity(opacity)
            .semantic_hidden(true)
    }
}

impl Render for Skeleton {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.reduced_motion = cx.environment().reduced_motion;
        let themes = shadcn(cx.environment());
        self.build(themes.resolve(cx.environment().color_scheme))
    }

    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        if self.wants_animation_frame() {
            self.phase = (self.phase + frame.elapsed.as_secs_f64() as f32 / 2.0) % 1.0;
            cx.notify();
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.animated && !self.reduced_motion
    }
}
