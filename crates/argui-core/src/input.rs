use std::collections::HashMap;

use crate::{Modifiers, Point};

/// Platform-independent timing and distance thresholds for pointer gestures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerSettings {
    multi_click_interval: std::time::Duration,
    multi_click_distance: f32,
    activation_slop: f32,
    long_press_interval: std::time::Duration,
    touch_slop: f32,
}

impl Default for PointerSettings {
    fn default() -> Self {
        Self {
            multi_click_interval: std::time::Duration::from_millis(500),
            multi_click_distance: 5.0,
            activation_slop: 10.0,
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

    /// Sets the maximum movement allowed before a pointer press stops activating its target.
    ///
    /// * `distance` — maximum movement in logical pixels from the initial press position.
    ///
    /// Returns the updated pointer settings.
    ///
    /// # Panics
    /// Panics if `distance` is negative or not finite.
    #[must_use]
    pub fn activation_slop(mut self, distance: f32) -> Self {
        assert!(distance.is_finite() && distance >= 0.0);
        self.activation_slop = distance;
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

    /// Returns the maximum movement in logical pixels that preserves pointer activation.
    #[must_use]
    pub const fn activation_slop_distance(self) -> f32 {
        self.activation_slop
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

/// Incremental result produced by a two-contact pinch recognizer.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PinchUpdate {
    /// Scale change since the previous sample, when the distance changed.
    pub scale: Option<f32>,
    /// Whether this sample started a new pinch gesture.
    pub started: bool,
    /// Whether this contact belongs to a pinch and should bypass ordinary UI input.
    pub consumed: bool,
}

#[derive(Clone, Copy, Debug)]
struct PinchSample {
    contacts: [PointerId; 2],
    distance: f32,
}

/// Tracks raw touch contacts and recognizes an incremental two-finger pinch.
///
/// Once a pinch starts, every participating contact remains consumed until all
/// contacts end. This prevents a completed pinch from becoming an accidental
/// click or one-finger scroll.
#[derive(Debug, Default)]
pub struct PinchRecognizer {
    points: HashMap<PointerId, Point>,
    sample: Option<PinchSample>,
    suppress_until_empty: bool,
}

impl PinchRecognizer {
    /// Observes one touch contact and returns the resulting pinch state.
    ///
    /// `id`, `phase`, and `position` identify a contact in one consistent
    /// coordinate space. The returned scale is incremental, so callers should
    /// multiply their current value by it. Coincident contacts do not start a
    /// gesture.
    #[must_use]
    pub fn observe(&mut self, id: PointerId, phase: PointerPhase, position: Point) -> PinchUpdate {
        let was_suppressed = self.suppress_until_empty;
        match phase {
            PointerPhase::Pressed | PointerPhase::Moved | PointerPhase::Entered => {
                self.points.insert(id, position);
            }
            PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left => {
                self.points.remove(&id);
            }
        }

        if self.sample.is_some_and(|sample| {
            sample
                .contacts
                .iter()
                .any(|contact| !self.points.contains_key(contact))
        }) {
            self.sample = None;
        }
        if self.sample.is_none() && self.points.len() >= 2 {
            let mut contacts: Vec<_> = self.points.keys().copied().collect();
            contacts.sort_by_key(|contact| contact.get());
            let contacts = [contacts[0], contacts[1]];
            let distance = self.distance(contacts);
            if distance > f32::EPSILON {
                self.sample = Some(PinchSample { contacts, distance });
            }
        }

        let previous = self.sample;
        if let Some(sample) = previous {
            self.sample = Some(PinchSample {
                contacts: sample.contacts,
                distance: self.distance(sample.contacts),
            });
        }
        let started = !was_suppressed && self.sample.is_some();
        self.suppress_until_empty |= self.sample.is_some();
        let scale = previous
            .zip(self.sample)
            .map(|(before, after)| after.distance / before.distance)
            .filter(|scale| scale.is_finite() && (*scale - 1.0).abs() >= f32::EPSILON);
        if self.points.len() < 2 {
            self.sample = None;
        }
        if self.points.is_empty() {
            self.suppress_until_empty = false;
        }
        PinchUpdate {
            scale,
            started,
            consumed: was_suppressed || self.suppress_until_empty,
        }
    }

    /// Returns the distance between `contacts` in the recognizer's coordinate
    /// space, or zero if either contact is no longer active.
    fn distance(&self, contacts: [PointerId; 2]) -> f32 {
        let [Some(first), Some(second)] = contacts.map(|contact| self.points.get(&contact)) else {
            return 0.0;
        };
        (second.x - first.x).hypot(second.y - first.y)
    }
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
