use argui_core::{Point, PointerId};

use crate::NodeId;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PanAxis {
    Horizontal,
    Vertical,
    #[default]
    Both,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GestureCapture {
    #[default]
    None,
    OnPress,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GestureDelivery {
    #[default]
    Immediate,
    FrameCoalesced,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TapGesture {
    pub max_distance: f32,
    pub max_duration: std::time::Duration,
}

impl Default for TapGesture {
    fn default() -> Self {
        Self {
            max_distance: 8.0,
            max_duration: std::time::Duration::from_millis(500),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanGesture {
    pub axis: PanAxis,
    pub threshold: f32,
    pub capture: GestureCapture,
    pub delivery: GestureDelivery,
}

impl Default for PanGesture {
    fn default() -> Self {
        Self {
            axis: PanAxis::Both,
            threshold: 8.0,
            capture: GestureCapture::None,
            delivery: GestureDelivery::Immediate,
        }
    }
}

impl PanGesture {
    /// Sets the axis or axes on which this pan recognizer responds.
    #[must_use]
    pub const fn axis(mut self, axis: PanAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Starts the pan as soon as it receives a press.
    #[must_use]
    pub const fn immediate(mut self) -> Self {
        self.threshold = 0.0;
        self
    }

    /// Sets the movement distance required to start the pan.
    ///
    /// * `threshold` — required pointer movement in logical pixels.
    #[must_use]
    pub const fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    /// Sets whether the pan captures its pointer when pressed.
    ///
    /// * `capture` — pointer capture policy for this pan.
    #[must_use]
    pub const fn capture(mut self, capture: GestureCapture) -> Self {
        self.capture = capture;
        self
    }

    /// Sets whether pan updates are delivered immediately or coalesced per frame.
    ///
    /// * `delivery` — update delivery strategy.
    #[must_use]
    pub const fn delivery(mut self, delivery: GestureDelivery) -> Self {
        self.delivery = delivery;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PinchGesture {
    pub threshold: f32,
    pub delivery: GestureDelivery,
}

impl Default for PinchGesture {
    fn default() -> Self {
        Self {
            threshold: 0.02,
            delivery: GestureDelivery::Immediate,
        }
    }
}

impl PinchGesture {
    /// Sets whether pinch updates are delivered immediately or coalesced per frame.
    ///
    /// * `delivery` — update delivery strategy.
    #[must_use]
    pub const fn delivery(mut self, delivery: GestureDelivery) -> Self {
        self.delivery = delivery;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RotationGesture {
    pub threshold_radians: f32,
    pub delivery: GestureDelivery,
}

impl Default for RotationGesture {
    fn default() -> Self {
        Self {
            threshold_radians: 2.0_f32.to_radians(),
            delivery: GestureDelivery::Immediate,
        }
    }
}

impl RotationGesture {
    /// Sets whether rotation updates are delivered immediately or coalesced per frame.
    ///
    /// * `delivery` — update delivery strategy.
    #[must_use]
    pub const fn delivery(mut self, delivery: GestureDelivery) -> Self {
        self.delivery = delivery;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GestureSet {
    pub tap: Option<TapGesture>,
    pub pan: Option<PanGesture>,
    pub pinch: Option<PinchGesture>,
    pub rotation: Option<RotationGesture>,
}

impl GestureSet {
    /// An empty set with no enabled recognizers.
    pub const EMPTY: Self = Self {
        tap: None,
        pan: None,
        pinch: None,
        rotation: None,
    };

    /// Enables tap recognition with the supplied thresholds.
    ///
    /// * `gesture` — tap recognition configuration.
    #[must_use]
    pub const fn tap(mut self, gesture: TapGesture) -> Self {
        self.tap = Some(gesture);
        self
    }

    /// Enables pan recognition with the supplied axis and thresholds.
    ///
    /// * `gesture` — pan recognition configuration.
    #[must_use]
    pub const fn pan(mut self, gesture: PanGesture) -> Self {
        self.pan = Some(gesture);
        self
    }

    /// Enables pinch recognition with the supplied threshold.
    ///
    /// * `gesture` — pinch recognition configuration.
    #[must_use]
    pub const fn pinch(mut self, gesture: PinchGesture) -> Self {
        self.pinch = Some(gesture);
        self
    }

    /// Enables rotation recognition with the supplied threshold.
    ///
    /// * `gesture` — rotation recognition configuration.
    #[must_use]
    pub const fn rotation(mut self, gesture: RotationGesture) -> Self {
        self.rotation = Some(gesture);
        self
    }

    /// Returns whether pan recognition captures the pointer on press.
    #[must_use]
    pub const fn captures_on_press(self) -> bool {
        matches!(
            self.pan,
            Some(PanGesture {
                capture: GestureCapture::OnPress,
                ..
            })
        )
    }

    /// Returns whether this set contains any enabled recognizer.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.tap.is_none() && self.pan.is_none() && self.pinch.is_none() && self.rotation.is_none()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GesturePhase {
    Started,
    Changed,
    Ended,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GestureKind {
    Tap {
        position: Point,
    },
    Pan {
        position: Point,
        delta: Point,
        total: Point,
        velocity: Point,
    },
    Pinch {
        scale: f32,
    },
    Rotation {
        radians: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GestureEvent {
    pub target: NodeId,
    pub pointer: PointerId,
    pub phase: GesturePhase,
    pub kind: GestureKind,
    pub delivery: GestureDelivery,
}
