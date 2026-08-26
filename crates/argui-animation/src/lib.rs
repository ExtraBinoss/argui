//! Typed, platform-independent animation primitives for Argui.

mod clock;
mod easing;
mod interpolate;
mod keyframe;
mod scheduler;
mod time;
mod timeline;
mod timing;

pub use clock::{Clock, ManualClock};
pub use easing::{CubicBezier, Easing, EasingError, LinearStop, StepPosition, Steps};
pub use interpolate::Interpolate;
pub use keyframe::{Keyframe, Keyframes};
pub use scheduler::{AnimationId, Frame, Scheduler};
pub use time::{Duration, Time};
pub use timeline::{PlaybackState, Timeline, TimelineEvents, TimelineSample};
pub use timing::{Direction, FillMode, Iterations, Timing, TimingError};
