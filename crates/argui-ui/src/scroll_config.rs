use std::time::Duration;

use argui_core::{Point, ScrollDelta};
use argui_paint::QuadStyle;

use crate::{Sides, StyleCondition, StylePatch, StyleTransition};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollAxes {
    Horizontal,
    #[default]
    Vertical,
    Both,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollPolarity {
    #[default]
    Normal,
    Inverted,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollPropagation {
    #[default]
    Chain,
    Contain,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OverscrollBehavior {
    #[default]
    Clamp,
    Elastic(ElasticScroll),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElasticScroll {
    pub resistance: f32,
    pub limit: f32,
    pub spring: f32,
    pub damping: f32,
}

impl Default for ElasticScroll {
    fn default() -> Self {
        Self {
            resistance: 0.35,
            limit: 96.0,
            spring: 240.0,
            damping: 28.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollAnchoring {
    #[default]
    Auto,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ScrollPhysics {
    Direct,
    Native,
    #[default]
    Hybrid,
    Inertial(InertialScroll),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InertialScroll {
    pub continuation_grace: Duration,
    pub velocity_limit: f32,
    pub stop_velocity: f32,
    pub decay: f32,
    pub sample_weight: f32,
}

impl Default for InertialScroll {
    fn default() -> Self {
        Self {
            continuation_grace: Duration::from_millis(18),
            velocity_limit: 2_400.0,
            stop_velocity: 24.0,
            decay: 8.5,
            sample_weight: 0.55,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollbarVisibility {
    Always,
    #[default]
    Auto,
    Hidden,
}

/// Horizontal edge used by a vertical scrollbar.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollbarSide {
    /// Place the scrollbar at the physical left edge of the viewport.
    Left,
    /// Place the scrollbar at the physical right edge of the viewport.
    #[default]
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollConfig {
    pub enabled: bool,
    pub axes: ScrollAxes,
    pub polarity: ScrollPolarity,
    pub natural_touch_scroll: bool,
    pub propagation: ScrollPropagation,
    pub line_size: f32,
    pub multiplier: f32,
    pub physics: ScrollPhysics,
    pub overscroll: OverscrollBehavior,
    pub anchoring: ScrollAnchoring,
    pub scrollbar: Option<ScrollbarStyle>,
    pub effects: Vec<crate::ScrollEffect>,
}

impl Default for ScrollConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            axes: ScrollAxes::Vertical,
            polarity: ScrollPolarity::Normal,
            natural_touch_scroll: true,
            propagation: ScrollPropagation::Chain,
            line_size: 40.0,
            multiplier: 1.0,
            physics: ScrollPhysics::Hybrid,
            overscroll: OverscrollBehavior::Clamp,
            anchoring: ScrollAnchoring::Auto,
            scrollbar: None,
            effects: Vec::new(),
        }
    }
}

impl ScrollConfig {
    /// Adds a paint effect driven by this scroll configuration's metrics.
    #[must_use]
    pub fn effect(mut self, effect: crate::ScrollEffect) -> Self {
        self.effects.push(effect);
        self
    }

    /// Sets whether this scroll container accepts scroll input.
    /// * `enabled` — whether the container processes scroll input.
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Restricts scrolling to the selected axes.
    #[must_use]
    pub const fn axes(mut self, axes: ScrollAxes) -> Self {
        self.axes = axes;
        self
    }

    /// Sets whether native scroll deltas are inverted.
    /// * `polarity` — interpretation of native scroll delta signs.
    #[must_use]
    pub const fn polarity(mut self, polarity: ScrollPolarity) -> Self {
        self.polarity = polarity;
        self
    }

    /// Sets whether content follows a finger during direct touch scrolling.
    ///
    /// * `natural` — `true` makes content follow the finger; `false` reverses touch movement.
    ///
    /// Wheel and trackpad deltas remain controlled by [`Self::polarity`].
    #[must_use]
    pub const fn natural_touch_scroll(mut self, natural: bool) -> Self {
        self.natural_touch_scroll = natural;
        self
    }

    /// Sets whether unconsumed scroll deltas chain to ancestors.
    /// * `propagation` — policy for forwarding unused scroll movement.
    #[must_use]
    pub const fn propagation(mut self, propagation: ScrollPropagation) -> Self {
        self.propagation = propagation;
        self
    }

    /// Sets the logical-pixel size of one input scroll line.
    /// * `line_size` — logical-pixel distance represented by one line delta.
    #[must_use]
    pub const fn line_size(mut self, line_size: f32) -> Self {
        self.line_size = line_size;
        self
    }

    /// Sets the multiplier applied to input scroll deltas.
    #[must_use]
    pub const fn multiplier(mut self, multiplier: f32) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Sets the scroll physics model.
    #[must_use]
    pub const fn physics(mut self, physics: ScrollPhysics) -> Self {
        self.physics = physics;
        self
    }

    /// Sets the behavior used when scrolling beyond the content extent.
    /// * `overscroll` — policy for handling movement beyond the extent.
    #[must_use]
    pub const fn overscroll(mut self, overscroll: OverscrollBehavior) -> Self {
        self.overscroll = overscroll;
        self
    }

    /// Sets how scroll offsets are preserved when content changes.
    /// * `anchoring` — scroll anchoring policy.
    #[must_use]
    pub const fn anchoring(mut self, anchoring: ScrollAnchoring) -> Self {
        self.anchoring = anchoring;
        self
    }

    /// Sets the scrollbar style for this scroll container.
    #[must_use]
    pub fn scrollbar(mut self, scrollbar: ScrollbarStyle) -> Self {
        self.scrollbar = Some(scrollbar);
        self
    }

    pub(crate) fn logical_delta(&self, delta: ScrollDelta) -> Point {
        let mut delta = match delta {
            ScrollDelta::Lines(point) => {
                let line_size = finite(self.line_size);
                Point::new(point.x * line_size, point.y * line_size)
            }
            ScrollDelta::Pixels(point) => point,
        };
        let polarity = match self.polarity {
            ScrollPolarity::Normal => -1.0,
            ScrollPolarity::Inverted => 1.0,
        } * finite(self.multiplier);
        delta.x *= polarity;
        delta.y *= polarity;
        delta.x = finite(delta.x);
        delta.y = finite(delta.y);
        match self.axes {
            ScrollAxes::Horizontal => {
                if delta.x.abs() <= f32::EPSILON {
                    delta.x = delta.y;
                }
                delta.y = 0.0;
            }
            ScrollAxes::Vertical => delta.x = 0.0,
            ScrollAxes::Both => {}
        }
        delta
    }
}

fn finite(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollbarStyle {
    pub width: f32,
    pub side: ScrollbarSide,
    pub insets: Sides<f32>,
    pub min_thumb: f32,
    pub visibility: ScrollbarVisibility,
    pub hide_delay: Duration,
    pub fade_duration: Duration,
    pub track: ScrollbarPartStyle,
    pub thumb: ScrollbarPartStyle,
}

impl ScrollbarStyle {
    /// Creates a scrollbar with separate styles for its track and thumb.
    ///
    /// * `track` — visual style for the track.
    /// * `thumb` — visual style for the draggable thumb.
    #[must_use]
    pub const fn new(track: ScrollbarPartStyle, thumb: ScrollbarPartStyle) -> Self {
        Self {
            width: 10.0,
            side: ScrollbarSide::Right,
            insets: Sides {
                left: 4.0,
                right: 4.0,
                top: 4.0,
                bottom: 4.0,
            },
            min_thumb: 28.0,
            visibility: ScrollbarVisibility::Auto,
            hide_delay: Duration::from_millis(700),
            fade_duration: Duration::from_millis(160),
            track,
            thumb,
        }
    }

    /// Sets the scrollbar width in logical pixels.
    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the physical edge used by the vertical scrollbar.
    ///
    /// * `side` — left or right edge of the scroll viewport.
    #[must_use]
    pub const fn side(mut self, side: ScrollbarSide) -> Self {
        self.side = side;
        self
    }

    /// Sets the scrollbar insets from the scroll viewport edges.
    #[must_use]
    pub const fn insets(mut self, insets: Sides<f32>) -> Self {
        self.insets = insets;
        self
    }

    /// Sets the minimum thumb length in logical pixels.
    /// * `min_thumb` — shortest permitted scrollbar thumb.
    #[must_use]
    pub const fn min_thumb(mut self, min_thumb: f32) -> Self {
        self.min_thumb = min_thumb;
        self
    }

    /// Sets when the scrollbar is visible.
    /// * `visibility` — visibility policy.
    #[must_use]
    pub const fn visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Sets the delay before an automatically shown scrollbar starts fading.
    /// * `hide_delay` — time to wait before fading begins.
    #[must_use]
    pub const fn hide_delay(mut self, hide_delay: Duration) -> Self {
        self.hide_delay = hide_delay;
        self
    }

    /// Sets how long a fading scrollbar takes to disappear.
    /// * `fade_duration` — duration of the fade-out.
    #[must_use]
    pub const fn fade_duration(mut self, fade_duration: Duration) -> Self {
        self.fade_duration = fade_duration;
        self
    }

    #[must_use]
    /// Returns the width reserved by this scrollbar's gutter and insets.
    pub fn gutter_width(&self) -> f32 {
        (self.width + self.insets.right.max(self.insets.bottom)).max(0.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollbarPartStyle {
    pub base: QuadStyle,
    states: crate::state::ConditionalStyles,
    pub transition: Option<StyleTransition>,
}

impl ScrollbarPartStyle {
    /// Creates a part style from its base quad paint.
    ///
    /// * `base` — default paint style before conditional rules are applied.
    #[must_use]
    pub const fn new(base: QuadStyle) -> Self {
        Self {
            base,
            states: crate::state::ConditionalStyles::new(),
            transition: None,
        }
    }

    #[must_use]
    /// Adds a conditional style for this scrollbar part.
    /// * `condition` — condition under which the style patch is applied.
    /// * `style` — style changes applied when the condition matches.
    pub fn when(mut self, condition: impl Into<StyleCondition>, style: StylePatch) -> Self {
        assert!(
            style.values().iter().all(|property| property.key.is_quad()),
            "scrollbar states only accept quad paint properties"
        );
        self.states.set(condition.into(), style);
        self
    }

    #[must_use]
    /// Sets transitions between conditional styles for this part.
    /// * `transition` — animation configuration for style changes.
    pub fn transition(mut self, transition: StyleTransition) -> Self {
        self.transition = Some(transition);
        self
    }

    pub(crate) fn state_rules(&self) -> &[crate::state::StyleRule] {
        self.states.rules()
    }

    pub(crate) fn has_states(&self) -> bool {
        !self.states.is_empty()
    }

    pub(crate) fn has_container_queries(&self) -> bool {
        self.states.has_container_queries()
    }
}
