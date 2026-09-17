use argui_animation::{Curve, Duration, Easing, Transition, Tween, curves};
use argui_core::Transform2D;
use argui_paint::{Border, Color, CornerRadii, Fill, LayerStyle};
use argui_ui::{
    Dimension, Element, LengthPercentage, LengthPercentageAuto, PropertyKey, Sides,
    StyleTransition, TransitionRule,
};

const DEFAULT_OPACITY_DURATION: Duration = Duration::from_millis(200);
const DEFAULT_CONTAINER_DURATION: Duration = Duration::from_millis(300);

/// Implicitly transitions the group opacity of one element and its descendants.
///
/// The first build presents `opacity` immediately. Later builds with the same
/// retained identity animate from the currently presented opacity to the new
/// target. An opacity of zero does not disable pointer or semantic interaction;
/// configure those policies on `child` when hidden content must be inert.
#[derive(Clone, Debug)]
pub struct AnimatedOpacity {
    child: Element,
    opacity: f32,
    tween: Tween,
}

impl AnimatedOpacity {
    /// Creates an implicitly animated group opacity.
    ///
    /// * `key` — stable identity retained across target changes.
    /// * `opacity` — target group opacity, clamped to `0.0..=1.0` when built.
    /// * `child` — element whose complete rendered subtree fades.
    #[must_use]
    pub fn new(key: impl Into<String>, opacity: f32, child: Element) -> Self {
        Self {
            child: child.keyed(key),
            opacity,
            tween: Tween::new(DEFAULT_OPACITY_DURATION).easing(curves::EASE_OUT),
        }
    }

    /// Sets the interpolation duration.
    ///
    /// * `duration` — active time between the presented and target opacity.
    #[must_use]
    pub const fn duration(mut self, duration: Duration) -> Self {
        self.tween.duration = duration;
        self
    }

    /// Sets the delay before interpolation begins.
    ///
    /// * `delay` — time for which the currently presented opacity is retained.
    #[must_use]
    pub const fn delay(mut self, delay: Duration) -> Self {
        self.tween.delay = delay;
        self
    }

    /// Sets the easing curve used between the presented and target opacity.
    ///
    /// * `curve` — built-in easing or user-defined curve sampled by the transition.
    #[must_use]
    pub fn curve(mut self, curve: impl Curve) -> Self {
        self.tween.easing = Easing::curve(curve);
        self
    }

    /// Replaces the complete tween used when the opacity target changes.
    ///
    /// * `tween` — duration, delay and easing policy for the transition.
    #[must_use]
    pub fn tween(mut self, tween: Tween) -> Self {
        self.tween = tween;
        self
    }

    /// Builds the decorated element with its retained opacity transition.
    #[must_use]
    pub fn build(self) -> Element {
        let immediate = Transition::tween(Tween::new(Duration::ZERO));
        let opacity = Transition::tween(self.tween);
        self.child.opacity(self.opacity).transition(
            StyleTransition::new(immediate)
                .rule(TransitionRule::new(opacity).property(PropertyKey::LayerOpacity)),
        )
    }
}

/// Container whose compatible visual and layout properties animate after changes.
///
/// The first build presents its authored values immediately. Retained rebuilds
/// interpolate scalar properties such as size, padding, colors, radii, transforms,
/// shadows and group opacity. Changes between incompatible representations, such
/// as pixels and percentages or different gradient kinds, switch discretely.
#[derive(Clone, Debug)]
pub struct AnimatedContainer {
    element: Element,
    tween: Tween,
}

impl AnimatedContainer {
    /// Creates an animated container retaining `children` under `key`.
    ///
    /// * `key` — stable identity retained across property changes.
    /// * `children` — elements laid out inside the container.
    #[must_use]
    pub fn new(key: impl Into<String>, children: impl IntoIterator<Item = Element>) -> Self {
        Self {
            element: Element::container(children).keyed(key),
            tween: Tween::new(DEFAULT_CONTAINER_DURATION).easing(curves::STANDARD),
        }
    }

    /// Turns an existing element into an implicitly animated container.
    ///
    /// `element` keeps its kind, children, identity and authored styles. The
    /// animation policy installed by [`AnimatedContainer::build`] replaces any
    /// earlier style-transition policy on that element.
    #[must_use]
    pub fn from_element(element: Element) -> Self {
        Self {
            element,
            tween: Tween::new(DEFAULT_CONTAINER_DURATION).easing(curves::STANDARD),
        }
    }

    /// Applies arbitrary element configuration before installing the transition.
    ///
    /// * `configure` — function receiving and returning the underlying element.
    #[must_use]
    pub fn configure(mut self, configure: impl FnOnce(Element) -> Element) -> Self {
        self.element = configure(self.element);
        self
    }

    /// Sets the target preferred width.
    #[must_use]
    pub fn width(mut self, width: Dimension) -> Self {
        self.element = self.element.width(width);
        self
    }

    /// Sets the target preferred height.
    #[must_use]
    pub fn height(mut self, height: Dimension) -> Self {
        self.element = self.element.height(height);
        self
    }

    /// Sets the target minimum width.
    #[must_use]
    pub fn min_width(mut self, width: LengthPercentageAuto) -> Self {
        self.element = self.element.min_width(width);
        self
    }

    /// Sets the target minimum height.
    #[must_use]
    pub fn min_height(mut self, height: LengthPercentageAuto) -> Self {
        self.element = self.element.min_height(height);
        self
    }

    /// Sets the target maximum width.
    #[must_use]
    pub fn max_width(mut self, width: LengthPercentageAuto) -> Self {
        self.element = self.element.max_width(width);
        self
    }

    /// Sets the target maximum height.
    #[must_use]
    pub fn max_height(mut self, height: LengthPercentageAuto) -> Self {
        self.element = self.element.max_height(height);
        self
    }

    /// Sets the target inner spacing on each side.
    #[must_use]
    pub fn padding(mut self, padding: Sides<LengthPercentage>) -> Self {
        self.element = self.element.padding(padding);
        self
    }

    /// Sets the target row and column gap in logical pixels.
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.element = self.element.gap(gap);
        self
    }

    /// Sets the target solid background color.
    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self.element = self.element.background(color);
        self
    }

    /// Sets the target background fill.
    #[must_use]
    pub fn fill(mut self, fill: Fill) -> Self {
        self.element = self.element.fill(fill);
        self
    }

    /// Sets the target border.
    #[must_use]
    pub fn border(mut self, border: Border) -> Self {
        self.element = self.element.border(border);
        self
    }

    /// Sets the target corner radii.
    #[must_use]
    pub fn radius(mut self, radii: CornerRadii) -> Self {
        self.element = self.element.radius(radii);
        self
    }

    /// Sets the target transform.
    #[must_use]
    pub fn transform(mut self, transform: Transform2D) -> Self {
        self.element = self.element.transform(transform);
        self
    }

    /// Sets the target group opacity for the container and its descendants.
    #[must_use]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.element = self.element.opacity(opacity);
        self
    }

    /// Sets the target compositing layer, including masks, shadows and effects.
    #[must_use]
    pub fn layer(mut self, layer: LayerStyle) -> Self {
        self.element = self.element.layer(layer);
        self
    }

    /// Sets the interpolation duration for compatible property changes.
    #[must_use]
    pub const fn duration(mut self, duration: Duration) -> Self {
        self.tween.duration = duration;
        self
    }

    /// Sets the delay before compatible property changes begin interpolating.
    #[must_use]
    pub const fn delay(mut self, delay: Duration) -> Self {
        self.tween.delay = delay;
        self
    }

    /// Sets the built-in or user-defined curve shared by compatible changes.
    #[must_use]
    pub fn curve(mut self, curve: impl Curve) -> Self {
        self.tween.easing = Easing::curve(curve);
        self
    }

    /// Replaces the complete tween used by compatible property changes.
    #[must_use]
    pub fn tween(mut self, tween: Tween) -> Self {
        self.tween = tween;
        self
    }

    /// Builds the container and installs its retained transition policy.
    #[must_use]
    pub fn build(self) -> Element {
        self.element
            .transition(StyleTransition::new(Transition::tween(self.tween)))
    }
}
