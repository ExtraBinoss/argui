use crate::Duration;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CueId(usize);

impl CueId {
    #[must_use]
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cue {
    pub start: Duration,
    pub duration: Duration,
}

impl Cue {
    #[must_use]
    pub fn progress(self, elapsed: Duration) -> f32 {
        if self.duration == Duration::ZERO {
            return (elapsed >= self.start) as u8 as f32;
        }
        let local = elapsed.as_nanos().saturating_sub(self.start.as_nanos());
        (local as f32 / self.duration.as_nanos() as f32).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Schedule {
    cues: Box<[Cue]>,
    duration: Duration,
}

impl Schedule {
    #[must_use]
    pub fn sequence(durations: impl IntoIterator<Item = Duration>) -> Self {
        let mut builder = ScheduleBuilder::new();
        for duration in durations {
            builder.then(duration);
        }
        builder.build()
    }

    #[must_use]
    pub fn parallel(durations: impl IntoIterator<Item = Duration>) -> Self {
        let cues: Vec<_> = durations
            .into_iter()
            .map(|duration| Cue {
                start: Duration::ZERO,
                duration,
            })
            .collect();
        Self::from_cues(cues)
    }

    #[must_use]
    pub fn stagger(count: usize, duration: Duration, interval: Duration) -> Self {
        let cues = (0..count)
            .map(|index| Cue {
                start: Duration::from_nanos(interval.as_nanos().saturating_mul(index as u64)),
                duration,
            })
            .collect();
        Self::from_cues(cues)
    }

    fn from_cues(cues: Vec<Cue>) -> Self {
        let duration = cues
            .iter()
            .map(|cue| cue.start + cue.duration)
            .max()
            .unwrap_or(Duration::ZERO);
        Self {
            cues: cues.into_boxed_slice(),
            duration,
        }
    }

    #[must_use]
    pub fn cue(&self, id: CueId) -> Option<Cue> {
        self.cues.get(id.0).copied()
    }

    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.cues.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScheduleBuilder {
    cues: Vec<Cue>,
    cursor: Duration,
    group_start: Duration,
}

impl ScheduleBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cues: Vec::new(),
            cursor: Duration::ZERO,
            group_start: Duration::ZERO,
        }
    }

    pub fn then(&mut self, duration: Duration) -> CueId {
        self.group_start = self.cursor;
        let id = self.push(self.cursor, duration);
        self.cursor += duration;
        id
    }

    pub fn with(&mut self, duration: Duration) -> CueId {
        let id = self.push(self.group_start, duration);
        self.cursor = self.cursor.max(self.group_start + duration);
        id
    }

    pub fn after(&mut self, dependency: CueId, delay: Duration, duration: Duration) -> CueId {
        let start = self
            .cues
            .get(dependency.0)
            .map_or(delay, |cue| cue.start + cue.duration + delay);
        let id = self.push(start, duration);
        self.cursor = self.cursor.max(start + duration);
        id
    }

    fn push(&mut self, start: Duration, duration: Duration) -> CueId {
        let id = CueId(self.cues.len());
        self.cues.push(Cue { start, duration });
        id
    }

    #[must_use]
    pub fn build(self) -> Schedule {
        Schedule::from_cues(self.cues)
    }
}
