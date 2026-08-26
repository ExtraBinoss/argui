use crate::Point;

/// Device-independent wheel data. Line deltas remain distinct until a scroll
/// container applies its configured logical line size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollDelta {
    Lines(Point),
    Pixels(Point),
}
