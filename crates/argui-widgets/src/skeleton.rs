use argui_animation::{
    Duration, Iterations, Keyframe, Keyframes, Motion, Timeline, Timing, curves,
};
use argui_paint::CornerRadii;
use argui_runtime::{Context, Render};
use argui_ui::{Dimension, Element, length, percent, property};

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
    opacity: Option<Motion<f32>>,
}

impl Skeleton {
    /// Creates a placeholder skeleton identified by `key`.
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            width: percent(1.0),
            height: length(16.0),
            radius: 6.0,
            animated: true,
            opacity: None,
        }
    }

    #[must_use]
    /// Sets the skeleton dimensions using `width` and `height` constraints.
    pub fn size(mut self, width: Dimension, height: Dimension) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    #[must_use]
    /// Sets its corner radius in logical pixels.
    ///
    /// # Panics
    ///
    /// Panics if `radius` is non-finite or negative.
    pub fn radius(mut self, radius: f32) -> Self {
        assert!(
            radius.is_finite() && radius >= 0.0,
            "skeleton radius must be finite and nonnegative"
        );
        self.radius = radius;
        self
    }

    /// Sets whether the skeleton's shimmer animation is enabled; `animated` toggles the shimmer.
    pub fn set_animated(&mut self, animated: bool) {
        self.animated = animated;
        if !animated && let Some(opacity) = &self.opacity {
            opacity.set(1.0);
        }
    }

    #[must_use]
    /// Builds the skeleton using `theme` for its placeholder color.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        Element::container([])
            .keyed(self.key.clone())
            .width(self.width)
            .height(self.height)
            .min_width(length(0.0))
            .shrink(0.0)
            .background(theme.muted)
            .radius(CornerRadii::all(self.radius))
            .semantic_hidden(true)
    }
}

impl Render for Skeleton {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let skeleton = self.build(themes.resolve(cx.environment().color_scheme));
        if !self.animated || cx.environment().reduced_motion {
            if let Some(opacity) = &self.opacity {
                opacity.set(1.0);
            }
            return skeleton;
        }
        let opacity = self.opacity.get_or_insert_with(|| Motion::new(1.0));
        if !opacity.is_active() {
            opacity.set(1.0);
            opacity.play(pulse_timeline());
        }
        skeleton.bind(property::LayerOpacity, opacity.clone())
    }
}

fn pulse_timeline() -> Timeline<f32> {
    Timeline::new(
        Keyframes::new([
            Keyframe::new(0.0, 1.0).easing(curves::EASE_IN_OUT),
            Keyframe::new(0.5, 0.5).easing(curves::EASE_IN_OUT),
            Keyframe::new(1.0, 1.0),
        ])
        .expect("skeleton pulse keyframes are ordered"),
        Timing::new(Duration::from_secs(2)).iterations(Iterations::Infinite),
    )
    .expect("skeleton pulse timing is finite and non-zero")
}
