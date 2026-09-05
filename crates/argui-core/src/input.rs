use crate::{Modifiers, Point};

/// Platform-independent timing and distance thresholds for pointer gestures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerSettings {
    multi_click_interval: std::time::Duration,
    multi_click_distance: f32,
    long_press_interval: std::time::Duration,
    touch_slop: f32,
}

impl Default for PointerSettings {
    fn default() -> Self {
        Self {
            multi_click_interval: std::time::Duration::from_millis(500),
            multi_click_distance: 5.0,
            long_press_interval: std::time::Duration::from_millis(500),
            touch_slop: 10.0,
        }
    }
}

impl PointerSettings {
    #[must_use]
    pub fn multi_click(mut self, interval: std::time::Duration, distance: f32) -> Self {
        assert!(distance.is_finite() && distance >= 0.0);
        self.multi_click_interval = interval;
        self.multi_click_distance = distance;
        self
    }

    #[must_use]
    pub fn long_press(mut self, interval: std::time::Duration, touch_slop: f32) -> Self {
        assert!(touch_slop.is_finite() && touch_slop >= 0.0);
        self.long_press_interval = interval;
        self.touch_slop = touch_slop;
        self
    }

    #[must_use]
    pub const fn multi_click_interval(self) -> std::time::Duration {
        self.multi_click_interval
    }

    #[must_use]
    pub const fn multi_click_distance(self) -> f32 {
        self.multi_click_distance
    }

    #[must_use]
    pub const fn long_press_interval(self) -> std::time::Duration {
        self.long_press_interval
    }

    #[must_use]
    pub const fn touch_slop(self) -> f32 {
        self.touch_slop
    }
}

/// Device-independent wheel data. Line deltas remain distinct until a scroll
/// container applies its configured logical line size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollDelta {
    Lines(Point),
    Pixels(Point),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PointerId(u64);

impl PointerId {
    pub const MOUSE: Self = Self(0);

    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerKind {
    Mouse,
    Touch,
    Pen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerPhase {
    Entered,
    Moved,
    Pressed,
    Released,
    Left,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    pub id: PointerId,
    pub kind: PointerKind,
    pub phase: PointerPhase,
    pub position: Point,
    pub button: Option<PointerButton>,
    pub buttons: u16,
    pub pressure: Option<f32>,
    pub primary: bool,
    pub modifiers: Modifiers,
    pub timestamp: std::time::Duration,
}

impl PointerEvent {
    #[must_use]
    pub const fn mouse(phase: PointerPhase, position: Point) -> Self {
        Self {
            id: PointerId::MOUSE,
            kind: PointerKind::Mouse,
            phase,
            position,
            button: None,
            buttons: 0,
            pressure: None,
            primary: true,
            modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                super_key: false,
            },
            timestamp: std::time::Duration::ZERO,
        }
    }
}
