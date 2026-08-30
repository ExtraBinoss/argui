//! Typed, platform-independent animation primitives for Argui.

mod clock;
mod composition;
mod controller;
mod decay;
mod easing;
mod inertia;
mod interpolate;
mod keyframe;
mod motion;
mod schedule;
mod scheduler;
mod time;
mod timeline;
mod timing;
mod transition;

pub use clock::{Clock, ManualClock};
pub use composition::{Compose, Composition, Contribution, compose};
pub use controller::{Motion, MotionBinding, MotionState, MotionTrack, Tween};
pub use decay::{Decay, DecayConfig};
pub use easing::{CubicBezier, Easing, EasingError, LinearStop, StepPosition, Steps};
pub use inertia::{Inertia, InertiaConfig, InertiaState};
pub use interpolate::Interpolate;
pub use keyframe::{Keyframe, Keyframes};
pub use motion::{MotionValue, PhysicsError};
pub use schedule::{Cue, CueId, Schedule, ScheduleBuilder};
pub use scheduler::{AnimationId, Frame, Scheduler};
pub use spring::{Spring, SpringConfig};
pub use time::{Duration, Time};
pub use timeline::{PlaybackState, Timeline, TimelineEvents, TimelineSample};
pub use timing::{Direction, FillMode, Iterations, Timing, TimingError};
pub use transition::Transition;

mod spring;
