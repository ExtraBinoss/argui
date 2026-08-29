use crate::Point;

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
            timestamp: std::time::Duration::ZERO,
        }
    }
}
