use argui_animation::Tween;
use argui_core::{Point, Rect};

use crate::{FocusTarget, Sides};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollAlignment {
    Start,
    Center,
    End,
    #[default]
    Nearest,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ScrollBehavior {
    #[default]
    Instant,
    Smooth(Tween),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScrollTarget {
    Offset {
        container: FocusTarget,
        offset: Point,
    },
    Element(FocusTarget),
    Rect {
        container: FocusTarget,
        rect: Rect,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollRequest {
    pub target: ScrollTarget,
    pub x: ScrollAlignment,
    pub y: ScrollAlignment,
    pub margin: Sides<f32>,
    pub behavior: ScrollBehavior,
}

impl ScrollRequest {
    /// Requests an offset for a scroll container.
    ///
    /// * `container` — node or key identifying the scroll container.
    /// * `offset` — requested horizontal and vertical scroll offsets.
    #[must_use]
    pub fn offset(container: impl Into<FocusTarget>, offset: Point) -> Self {
        Self::new(ScrollTarget::Offset {
            container: container.into(),
            offset,
        })
    }

    /// Requests that a target element be brought into view.
    /// `target` identifies the element by node or key.
    #[must_use]
    pub fn reveal(target: impl Into<FocusTarget>) -> Self {
        Self::new(ScrollTarget::Element(target.into()))
    }

    /// Requests that a rectangle in a scroll container be brought into view.
    /// `container` identifies the container and `rect` is in its content coordinates.
    #[must_use]
    pub fn rect(container: impl Into<FocusTarget>, rect: Rect) -> Self {
        Self::new(ScrollTarget::Rect {
            container: container.into(),
            rect,
        })
    }

    fn new(target: ScrollTarget) -> Self {
        Self {
            target,
            x: ScrollAlignment::Nearest,
            y: ScrollAlignment::Nearest,
            margin: Sides {
                left: 0.0,
                right: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
            behavior: ScrollBehavior::Instant,
        }
    }

    /// Sets horizontal and vertical alignment for reveal requests.
    /// `x` and `y` are the horizontal and vertical alignment policies.
    #[must_use]
    pub const fn align(mut self, x: ScrollAlignment, y: ScrollAlignment) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Sets the margin to leave around the revealed content.
    /// `margin` is the inset applied around the reveal target.
    #[must_use]
    pub const fn margin(mut self, margin: Sides<f32>) -> Self {
        self.margin = margin;
        self
    }

    /// Sets whether the request is applied immediately or animated.
    /// `behavior` selects how this request is applied.
    #[must_use]
    pub fn behavior(mut self, behavior: ScrollBehavior) -> Self {
        self.behavior = behavior;
        self
    }
}
