//! Typed, platform-independent animation primitives for Argui.

mod clock;
mod composition;
mod easing;
mod interpolate;
mod keyframe;
mod schedule;
mod scheduler;
mod time;
mod timeline;
mod timing;

pub use clock::{Clock, ManualClock};
pub use composition::{Compose, Composition, Contribution, compose};
pub use easing::{CubicBezier, Easing, EasingError, LinearStop, StepPosition, Steps};
pub use interpolate::Interpolate;
pub use keyframe::{Keyframe, Keyframes};
pub use schedule::{Cue, CueId, Schedule, ScheduleBuilder};
pub use scheduler::{AnimationId, Frame, Scheduler};
pub use time::{Duration, Time};
pub use timeline::{PlaybackState, Timeline, TimelineEvents, TimelineSample};
pub use timing::{Direction, FillMode, Iterations, Timing, TimingError};
