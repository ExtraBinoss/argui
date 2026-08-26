//! Typed, platform-independent animation primitives for Argui.

mod clock;
mod interpolate;
mod scheduler;
mod time;

pub use clock::{Clock, ManualClock};
pub use interpolate::Interpolate;
pub use scheduler::{AnimationId, Frame, Scheduler};
pub use time::{Duration, Time};
