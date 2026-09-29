//! Display-linked sampling of retained native motion drivers.

use super::{Driver, MotionInner, MotionState};
use crate::{
    Duration, FillMode, Interpolate, Keyframe, Keyframes, MotionValue, PlaybackState, Time,
    Timeline, Timing, TimingError,
};

/// Advances a timeline driver while its caller holds the motion lock.
///
/// * `inner` — mutable retained motion state protected by its outer mutex.
/// * `now` — display-linked sample time.
///
/// Returns whether the sampled value changed.
pub(super) fn advance_timeline<T: Clone + Interpolate + PartialEq>(
    inner: &mut MotionInner<T>,
    now: Time,
) -> Result<bool, TimingError> {
    if inner.state != MotionState::Running {
        return Ok(false);
    }
    if let Driver::PendingTween { from, tween } = &inner.driver {
        let frames = Keyframes::new(vec![
            Keyframe::new(0.0, from.clone()).easing(tween.easing.clone()),
            Keyframe::new(1.0, inner.target.clone()),
        ])?;
        let mut timeline = Timeline::new(
            frames,
            Timing::new(tween.duration)
                .delay(tween.delay)
                .fill(FillMode::Both),
        )?;
        timeline.play(now);
        inner.driver = Driver::Timeline(timeline);
    }
    if inner.resume_pending {
        if let Driver::Timeline(timeline) = &mut inner.driver {
            match timeline.state() {
                PlaybackState::Idle | PlaybackState::Finished | PlaybackState::Canceled => {
                    timeline.play(now);
                }
                PlaybackState::Paused => timeline.resume(now),
                PlaybackState::Running => {}
            }
        }
        inner.resume_pending = false;
    }
    let sample = match &mut inner.driver {
        Driver::Timeline(timeline) => Some(timeline.sample(now)),
        _ => None,
    };
    let mut changed = false;
    if let Some(sample) = sample {
        inner.completed_iterations = inner
            .completed_iterations
            .saturating_add(sample.events.iterations);
        if sample.state == PlaybackState::Finished {
            changed = inner.value != inner.target;
            inner.value = inner.target.clone();
            // Keep the finished definition so declarative updates cannot replay an entry.
            inner.state = MotionState::Finished;
            inner.last_frame = None;
            return Ok(changed);
        }
        if let Some(value) = sample.value {
            changed = value != inner.value;
            inner.value = value;
        }
    }
    inner.last_frame = Some(now);
    Ok(changed)
}

/// Advances a spring driver while its caller holds the motion lock.
///
/// * `inner` — mutable retained motion state protected by its outer mutex.
/// * `now` — display-linked sample time.
///
/// Returns whether the sampled value changed.
pub(super) fn advance_spring<T: MotionValue>(inner: &mut MotionInner<T>, now: Time) -> bool {
    if inner.state != MotionState::Running {
        return false;
    }
    let elapsed = inner
        .last_frame
        .map_or(Duration::ZERO, |last| now.duration_since(last));
    let (changed, value, active) = match &mut inner.driver {
        Driver::Spring(spring) => {
            let changed = spring.advance(elapsed);
            (changed, spring.value(), spring.is_active())
        }
        _ => return false,
    };
    inner.value = value;
    inner.last_frame = Some(now);
    if !active {
        inner.driver = Driver::None;
        inner.state = MotionState::Finished;
        inner.last_frame = None;
    }
    changed
}
