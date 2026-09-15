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
    /// Sets the maximum time and movement distance used to recognize repeated clicks.
    /// * `interval` — maximum time between clicks.
    ///
    /// # Panics
    /// Panics if `distance` is negative or not finite.
    pub fn multi_click(mut self, interval: std::time::Duration, distance: f32) -> Self {
        assert!(distance.is_finite() && distance >= 0.0);
        self.multi_click_interval = interval;
        self.multi_click_distance = distance;
        self
    }

    #[must_use]
    /// Sets the long-press delay and maximum movement tolerated for touch input.
    /// * `interval` — minimum press duration before recognition.
    ///
    /// # Panics
    /// Panics if `touch_slop` is negative or not finite.
    pub fn long_press(mut self, interval: std::time::Duration, touch_slop: f32) -> Self {
        assert!(touch_slop.is_finite() && touch_slop >= 0.0);
        self.long_press_interval = interval;
        self.touch_slop = touch_slop;
        self
    }

    #[must_use]
    /// Returns the configured repeated-click interval.
    pub const fn multi_click_interval(self) -> std::time::Duration {
        self.multi_click_interval
    }

    #[must_use]
    /// Returns the maximum movement allowed between repeated clicks.
    pub const fn multi_click_distance(self) -> f32 {
        self.multi_click_distance
    }

    #[must_use]
    /// Returns the configured long-press interval.
    pub const fn long_press_interval(self) -> std::time::Duration {
        self.long_press_interval
    }

    #[must_use]
    /// Returns the maximum movement tolerated during a touch gesture.
    pub const fn touch_slop(self) -> f32 {
        self.touch_slop
    }
}

/// Device-independent wheel data. Line deltas remain distinct until a scroll
/// container applies its configured logical line size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollDelta {
    /// Movement expressed in wheel lines; containers choose the logical line size.
    Lines(Point),
    /// Movement expressed in physical pixels.
    Pixels(Point),
}

/// Stable identifier for a pointer device or contact.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PointerId(u64);

impl PointerId {
    /// Identifier reserved for the mouse pointer.
    pub const MOUSE: Self = Self(0);

    /// Creates an identifier from its platform-provided numeric value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerKind {
    /// Mouse or trackpad pointer.
    Mouse,
    /// Touch contact.
    Touch,
    /// Pen or stylus input.
    Pen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerPhase {
    /// The pointer entered a region.
    Entered,
    /// The pointer moved without a button transition.
    Moved,
    /// A button or contact was pressed.
    Pressed,
    /// A button or contact was released.
    Released,
    /// The pointer left a region.
    Left,
    /// The active pointer interaction was cancelled.
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerButton {
    /// Primary/left button.
    Primary,
    /// Secondary/right button.
    Secondary,
    /// Middle button.
    Middle,
    /// Back navigation button.
    Back,
    /// Forward navigation button.
    Forward,
    /// A platform-specific button number.
    Other(u16),
}

/// Platform-independent pointer input sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    /// Source pointer identifier.
    pub id: PointerId,
    /// Kind of pointing device.
    pub kind: PointerKind,
    /// Interaction phase represented by this sample.
    pub phase: PointerPhase,
    /// Pointer position in logical coordinates.
    pub position: Point,
    /// Button associated with this transition, when known.
    pub button: Option<PointerButton>,
    /// Bit mask of buttons currently held by the platform.
    pub buttons: u16,
    /// Device pressure, when supported by the platform.
    pub pressure: Option<f32>,
    /// Whether this is the primary pointer among concurrent pointers.
    pub primary: bool,
    /// Keyboard modifiers active with this sample.
    pub modifiers: Modifiers,
    /// Event time relative to the platform's chosen origin.
    pub timestamp: std::time::Duration,
}

impl PointerEvent {
    /// Creates a default mouse event at `position` with the given `phase`.
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
