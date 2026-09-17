use argui_animation::{Duration, Interpolate, Keyframes, Time, TimingError};
use argui_core::{Color, Point, Rect, Size, Transform2D};
use argui_paint::QuadStyle;

use crate::NodeId;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CaretAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaretHeight {
    Line,
    Pixels(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaretPrimitive {
    pub offset: Point,
    pub width: f32,
    pub height: CaretHeight,
    pub align: CaretAlign,
    pub paint: QuadStyle,
}

impl CaretPrimitive {
    /// Creates a caret primitive with a start-aligned origin offset of zero.
    ///
    /// * `width` — primitive width in logical pixels.
    /// * `height` — line-relative or fixed pixel height.
    /// * `paint` — quad paint style for the primitive.
    #[must_use]
    pub const fn new(width: f32, height: CaretHeight, paint: QuadStyle) -> Self {
        Self {
            offset: Point::new(0.0, 0.0),
            width,
            height,
            align: CaretAlign::Start,
            paint,
        }
    }

    /// Sets the primitive's x and y offset from the text line.
    #[must_use]
    pub const fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Point::new(x, y);
        self
    }

    /// Sets how a fixed-height primitive aligns within the text line.
    ///
    /// * `align` — vertical alignment within the line.
    #[must_use]
    pub const fn align(mut self, align: CaretAlign) -> Self {
        self.align = align;
        self
    }

    /// Computes the primitive's bounds relative to a shaped text line.
    ///
    /// * `line` — bounds of the text line containing the caret.
    #[must_use]
    pub fn bounds(&self, line: Rect) -> Rect {
        let height = match self.height {
            CaretHeight::Line => line.size.height,
            CaretHeight::Pixels(height) => height.max(0.0),
        };
        let y = match self.align {
            CaretAlign::Start => line.origin.y,
            CaretAlign::Center => line.origin.y + (line.size.height - height) * 0.5,
            CaretAlign::End => line.origin.y + line.size.height - height,
        };
        Rect::new(
            Point::new(line.origin.x + self.offset.x, y + self.offset.y),
            Size::new(self.width.max(0.0), height),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CaretVisual {
    pub primitives: Vec<CaretPrimitive>,
}

impl CaretVisual {
    /// Creates a visual from its caret primitives.
    ///
    /// * `primitives` — primitives to paint for the caret.
    #[must_use]
    pub fn new(primitives: impl IntoIterator<Item = CaretPrimitive>) -> Self {
        Self {
            primitives: primitives.into_iter().collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretFrame {
    pub opacity: f32,
    pub tint: Color,
    pub transform: Transform2D,
}

impl CaretFrame {
    /// Creates a frame with identity transform.
    ///
    /// * `opacity` — frame opacity.
    /// * `tint` — frame color tint.
    #[must_use]
    pub const fn new(opacity: f32, tint: Color) -> Self {
        Self {
            opacity,
            tint,
            transform: Transform2D::IDENTITY,
        }
    }

    /// Sets the frame's 2D transform.
    #[must_use]
    pub const fn transform(mut self, transform: Transform2D) -> Self {
        self.transform = transform;
        self
    }
}

impl Default for CaretFrame {
    fn default() -> Self {
        Self::new(1.0, Color::WHITE)
    }
}

impl Interpolate for CaretFrame {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        Self {
            opacity: self.opacity.interpolate(target.opacity, progress),
            tint: self.tint.interpolate(target.tint, progress),
            transform: self.transform.interpolate(target.transform, progress),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaretAnimation {
    pub keyframes: Keyframes<CaretFrame>,
    pub duration: Duration,
}

impl CaretAnimation {
    /// Creates a looping animation from keyframes and a non-zero duration.
    ///
    /// # Arguments
    ///
    /// * `keyframes` — frames sampled across the animation's normalized interval.
    /// * `duration` — time for one animation cycle.
    ///
    /// # Errors
    ///
    /// Returns [`TimingError::ZeroDuration`] when `duration` is zero.
    pub fn new(keyframes: Keyframes<CaretFrame>, duration: Duration) -> Result<Self, TimingError> {
        if duration == Duration::ZERO {
            return Err(TimingError::ZeroDuration);
        }
        Ok(Self {
            keyframes,
            duration,
        })
    }

    /// Samples the animation at elapsed time, wrapping at its duration.
    ///
    /// * `elapsed` — elapsed time since the animation began.
    #[must_use]
    pub fn sample(&self, elapsed: Duration) -> CaretFrame {
        let progress = (elapsed.as_nanos() % self.duration.as_nanos()) as f32
            / self.duration.as_nanos() as f32;
        self.keyframes.sample(progress)
    }

    /// Returns the delay until a held keyframe can visibly change.
    ///
    /// * `elapsed` — elapsed time since the looping animation began.
    ///
    /// Returns `None` while the current segment interpolates continuously.
    pub(crate) fn next_sample_in(&self, elapsed: Duration) -> Option<Duration> {
        let duration = self.duration.as_nanos();
        let cycle = elapsed.as_nanos() % duration;
        let progress = cycle as f64 / duration as f64;
        let frames = self.keyframes.as_slice();
        let upper = frames.partition_point(|frame| f64::from(frame.offset) <= progress);
        let from = &frames[upper.saturating_sub(1)];
        if !from.hold {
            return None;
        }
        let target = frames.get(upper).map_or(duration, |frame| {
            (f64::from(frame.offset) * duration as f64).ceil() as u64
        });
        Some(Duration::from_nanos(target.saturating_sub(cycle).max(1)))
    }

    /// Returns whether the animation can reuse retained pixels in the compositor.
    ///
    /// Opacity and transforms are composition properties. A changing tint must
    /// repaint the caret primitives to preserve authored color interpolation.
    #[must_use]
    pub fn supports_composition(&self) -> bool {
        let tint = self.keyframes.as_slice()[0].value.tint;
        self.keyframes
            .as_slice()
            .iter()
            .all(|frame| frame.value.tint == tint)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaretStyle {
    pub visual: CaretVisual,
    pub animation: Option<CaretAnimation>,
}

impl CaretStyle {
    /// Creates a static caret style from its visual primitives.
    ///
    /// * `visual` — caret primitives to display.
    #[must_use]
    pub const fn new(visual: CaretVisual) -> Self {
        Self {
            visual,
            animation: None,
        }
    }

    /// Adds an animation to this caret style.
    #[must_use]
    pub fn animated(mut self, animation: CaretAnimation) -> Self {
        self.animation = Some(animation);
        self
    }

    /// Samples the animation, or returns the default frame for a static style.
    ///
    /// * `elapsed` — elapsed time since the animation began.
    #[must_use]
    pub fn sample(&self, elapsed: Duration) -> CaretFrame {
        self.animation
            .as_ref()
            .map_or_else(CaretFrame::default, |animation| animation.sample(elapsed))
    }

    /// Returns whether this style has an animation.
    #[must_use]
    pub const fn is_animated(&self) -> bool {
        self.animation.is_some()
    }
}

impl Default for CaretStyle {
    fn default() -> Self {
        use argui_animation::{Keyframe, Keyframes};

        let visual = CaretVisual::new([CaretPrimitive::new(
            1.5,
            CaretHeight::Line,
            QuadStyle::solid(Color::WHITE),
        )]);
        let frames = Keyframes::new([
            Keyframe::new(0.0, CaretFrame::default()).hold(),
            Keyframe::new(0.5, CaretFrame::default()),
            Keyframe::new(0.5, CaretFrame::new(0.0, Color::WHITE)).hold(),
            Keyframe::new(1.0, CaretFrame::new(0.0, Color::WHITE)),
        ])
        .expect("the built-in caret keyframes must cover a sorted 0..=1 range");
        Self::new(visual).animated(
            CaretAnimation::new(frames, Duration::from_millis(1_000))
                .expect("the built-in caret duration must be non-zero"),
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CaretAnimator {
    node: Option<NodeId>,
    started: Option<Time>,
    elapsed: Duration,
}

impl CaretAnimator {
    /// Resets the retained caret identity and timeline.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Advances the retained caret timeline to `now` for `node`.
    ///
    /// Returns whether an animated caret should be repainted.
    pub fn advance(&mut self, node: Option<NodeId>, now: Time) -> bool {
        let Some(node) = node else {
            self.reset();
            return false;
        };
        if self.node != Some(node) || self.started.is_none() {
            self.node = Some(node);
            self.started = Some(now);
            self.elapsed = Duration::ZERO;
        } else if let Some(started) = self.started {
            self.elapsed = now.duration_since(started);
        }
        true
    }

    /// Returns whether `node` needs display-linked frames for `animation`.
    ///
    /// A new caret starts immediately; held keyframe segments sleep until their
    /// next visible change.
    pub fn wants_frame(self, node: NodeId, animation: &CaretAnimation) -> bool {
        self.node != Some(node)
            || self.started.is_none()
            || animation.next_sample_in(self.elapsed).is_none()
    }

    /// Returns the absolute animation time of the next held-keyframe change.
    ///
    /// Returns `None` for an uninitialized caret or a continuously interpolated
    /// segment.
    pub fn next_frame_at(self, node: NodeId, animation: &CaretAnimation) -> Option<Time> {
        if self.node != Some(node) {
            return None;
        }
        let started = self.started?;
        animation
            .next_sample_in(self.elapsed)
            .map(|delay| started + self.elapsed + delay)
    }

    /// Returns the retained elapsed time for `node`, or zero for another caret.
    pub fn elapsed(self, node: NodeId) -> Duration {
        if self.node == Some(node) {
            self.elapsed
        } else {
            Duration::ZERO
        }
    }
}
