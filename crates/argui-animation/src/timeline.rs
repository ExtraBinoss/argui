use crate::{
    Direction, Duration, Easing, FillMode, Interpolate, Iterations, Keyframe, Keyframes, Time,
    Timing, TimingError,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PlaybackState {
    #[default]
    Idle,
    Running,
    Paused,
    Finished,
    Canceled,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TimelineEvents {
    pub started: bool,
    pub iterations: u64,
    pub finished: bool,
    pub canceled: bool,
}

#[derive(Clone, Debug)]
pub struct TimelineSample<T> {
    pub value: Option<T>,
    pub events: TimelineEvents,
    pub state: PlaybackState,
}

#[derive(Clone, Debug)]
pub struct Timeline<T> {
    keyframes: Keyframes<T>,
    timing: Timing,
    state: PlaybackState,
    anchor_time: Time,
    anchor_position: f64,
    playback_rate: f64,
    started_emitted: bool,
    iteration_marker: u64,
    pending_finished: bool,
    pending_canceled: bool,
}

impl<T> Timeline<T> {
    pub fn new(keyframes: Keyframes<T>, timing: Timing) -> Result<Self, TimingError> {
        let timing = timing.validate()?;
        Ok(Self {
            keyframes,
            timing,
            state: PlaybackState::Idle,
            anchor_time: Time::ZERO,
            anchor_position: 0.0,
            playback_rate: timing.playback_rate,
            started_emitted: false,
            iteration_marker: 0,
            pending_finished: false,
            pending_canceled: false,
        })
    }

    #[must_use]
    pub const fn state(&self) -> PlaybackState {
        self.state
    }

    #[must_use]
    pub const fn timing(&self) -> Timing {
        self.timing
    }

    #[must_use]
    pub fn needs_frame(&self) -> bool {
        self.state == PlaybackState::Running
    }

    pub fn play(&mut self, now: Time) {
        match self.state {
            PlaybackState::Running => return,
            PlaybackState::Paused => {
                self.anchor_time = now;
                self.state = PlaybackState::Running;
                return;
            }
            PlaybackState::Idle | PlaybackState::Finished | PlaybackState::Canceled => {}
        }
        self.reset_position();
        self.anchor_time = now;
        self.state = PlaybackState::Running;
    }

    pub fn pause(&mut self, now: Time) {
        if self.state == PlaybackState::Running {
            self.anchor_position = self.position(now);
            self.anchor_time = now;
            self.state = PlaybackState::Paused;
        }
    }

    pub fn resume(&mut self, now: Time) {
        if self.state == PlaybackState::Paused {
            self.anchor_time = now;
            self.state = PlaybackState::Running;
        }
    }

    pub fn restart(&mut self, now: Time) {
        self.reset_position();
        self.anchor_time = now;
        self.state = PlaybackState::Running;
    }

    pub fn seek(&mut self, position: Duration, now: Time) {
        self.anchor_position = self.clamp_position(position.as_secs_f64());
        self.anchor_time = now;
        self.reset_events_for_position();
    }

    pub fn set_playback_rate(&mut self, playback_rate: f64, now: Time) -> Result<(), TimingError> {
        if !playback_rate.is_finite() || playback_rate == 0.0 {
            return Err(TimingError::InvalidPlaybackRate);
        }
        self.anchor_position = self.position(now);
        self.anchor_time = now;
        self.playback_rate = playback_rate;
        Ok(())
    }

    pub fn reverse(&mut self, now: Time) {
        let rate = -self.playback_rate;
        if matches!(
            self.state,
            PlaybackState::Idle | PlaybackState::Finished | PlaybackState::Canceled
        ) {
            self.playback_rate = rate;
            self.restart(now);
        } else {
            self.anchor_position = self.position(now);
            self.anchor_time = now;
            self.playback_rate = rate;
        }
    }

    pub fn finish(&mut self) {
        self.anchor_position = if self.playback_rate >= 0.0 {
            self.timing.total_seconds()
        } else {
            0.0
        };
        self.state = PlaybackState::Finished;
        self.pending_finished = true;
    }

    pub fn cancel(&mut self) {
        self.state = PlaybackState::Canceled;
        self.pending_canceled = true;
    }

    fn position(&self, now: Time) -> f64 {
        if self.state != PlaybackState::Running {
            return self.anchor_position;
        }
        let elapsed = now.duration_since(self.anchor_time).as_secs_f64();
        self.clamp_position(self.anchor_position + elapsed * self.playback_rate)
    }

    fn clamp_position(&self, position: f64) -> f64 {
        position.max(0.0).min(self.timing.total_seconds())
    }

    fn reset_position(&mut self) {
        self.anchor_position = if self.playback_rate >= 0.0 {
            0.0
        } else {
            self.timing.total_seconds()
        };
        self.started_emitted = false;
        self.iteration_marker = if self.playback_rate >= 0.0 {
            0
        } else {
            self.maximum_iteration_events()
        };
        self.pending_finished = false;
        self.pending_canceled = false;
    }

    fn reset_events_for_position(&mut self) {
        let active = (self.anchor_position - self.timing.delay.as_secs_f64()).max(0.0);
        self.started_emitted = active > 0.0;
        self.iteration_marker = self.iteration_events_at(active);
        self.pending_finished = false;
        self.pending_canceled = false;
    }

    fn maximum_iteration_events(&self) -> u64 {
        match self.timing.iterations {
            Iterations::Finite(iterations) => iterations.ceil().max(1.0) as u64 - 1,
            Iterations::Infinite => u64::MAX,
        }
    }

    fn iteration_events_at(&self, active_seconds: f64) -> u64 {
        let completed = (active_seconds / self.timing.duration.as_secs_f64()).floor();
        (completed as u64).min(self.maximum_iteration_events())
    }
}

impl<T: Clone + Interpolate> Timeline<T> {
    pub fn retarget(
        &mut self,
        target: T,
        duration: Duration,
        easing: Easing,
        now: Time,
    ) -> Result<(), TimingError> {
        let current = self
            .sample(now)
            .value
            .unwrap_or_else(|| self.keyframes.sample(0.0));
        self.keyframes = Keyframes::new(vec![
            Keyframe::new(0.0, current).easing(easing),
            Keyframe::new(1.0, target),
        ])?;
        self.timing = Timing::new(duration).fill(FillMode::Forwards).validate()?;
        self.playback_rate = 1.0;
        self.restart(now);
        Ok(())
    }

    #[must_use]
    pub fn sample(&mut self, now: Time) -> TimelineSample<T> {
        let mut events = TimelineEvents {
            finished: std::mem::take(&mut self.pending_finished),
            canceled: std::mem::take(&mut self.pending_canceled),
            ..TimelineEvents::default()
        };
        if matches!(self.state, PlaybackState::Idle | PlaybackState::Canceled) {
            return TimelineSample {
                value: None,
                events,
                state: self.state,
            };
        }

        let mut position = self.position(now);
        let delay = self.timing.delay.as_secs_f64();
        let active_end = delay + self.timing.active_seconds();
        if position >= delay && !self.started_emitted {
            self.started_emitted = true;
            events.started = true;
        }
        if position >= delay {
            let marker =
                self.iteration_events_at((position - delay).min(self.timing.active_seconds()));
            events.iterations = marker.abs_diff(self.iteration_marker);
            self.iteration_marker = marker;
        }

        if self.state == PlaybackState::Running && self.reached_end(position) {
            position = if self.playback_rate >= 0.0 {
                self.timing.total_seconds()
            } else {
                0.0
            };
            self.anchor_position = position;
            self.anchor_time = now;
            self.state = PlaybackState::Finished;
            events.finished = true;
        }

        let inside_active = position < active_end
            || (position == active_end && self.state != PlaybackState::Finished);
        let progress = if position < delay {
            self.timing
                .fill
                .fills_before()
                .then(|| self.directed_progress(0.0, 0))
        } else if inside_active {
            Some(self.progress_at((position - delay).max(0.0)))
        } else {
            self.timing
                .fill
                .fills_after()
                .then(|| self.final_progress())
        };
        TimelineSample {
            value: progress.map(|progress| self.keyframes.sample(progress)),
            events,
            state: self.state,
        }
    }

    fn reached_end(&self, position: f64) -> bool {
        const TIME_EPSILON: f64 = 1.0e-9;
        if self.playback_rate >= 0.0 {
            position + TIME_EPSILON >= self.timing.total_seconds()
        } else {
            position <= TIME_EPSILON
        }
    }

    fn progress_at(&self, active_seconds: f64) -> f32 {
        let duration = self.timing.duration.as_secs_f64();
        let iterations = active_seconds / duration;
        let index = iterations.floor() as u64;
        let fraction = iterations.fract();
        if active_seconds == self.timing.active_seconds() {
            return self.final_progress();
        }
        self.directed_progress(fraction as f32, index)
    }

    fn final_progress(&self) -> f32 {
        let iterations = match self.timing.iterations {
            Iterations::Finite(iterations) => iterations,
            Iterations::Infinite => return 1.0,
        };
        let fraction = iterations.fract();
        let index = iterations.ceil().max(1.0) as u64 - 1;
        self.directed_progress(
            if fraction == 0.0 {
                1.0
            } else {
                fraction as f32
            },
            index,
        )
    }

    fn directed_progress(&self, progress: f32, iteration: u64) -> f32 {
        let reverse = match self.timing.direction {
            Direction::Normal => false,
            Direction::Reverse => true,
            Direction::Alternate => !iteration.is_multiple_of(2),
            Direction::AlternateReverse => iteration.is_multiple_of(2),
        };
        if reverse { 1.0 - progress } else { progress }
    }
}
