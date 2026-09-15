use crate::Duration;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Index-based identifier for a cue in a schedule.
pub struct CueId(usize);

impl CueId {
    /// Creates an identifier from its zero-based cue index.
    #[must_use]
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based cue index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Start time and duration of one scheduled cue.
pub struct Cue {
    /// Cue start relative to the schedule origin.
    pub start: Duration,
    /// Cue duration.
    pub duration: Duration,
}

impl Cue {
    /// Returns normalized cue progress at `elapsed`, clamped to zero through one.
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
/// Collection of cues with a computed total duration.
pub struct Schedule {
    cues: Box<[Cue]>,
    duration: Duration,
}

impl Schedule {
    /// Creates cues that play consecutively in input order.
    /// * `durations` — cue durations in sequence order.
    #[must_use]
    pub fn sequence(durations: impl IntoIterator<Item = Duration>) -> Self {
        let mut builder = ScheduleBuilder::new();
        for duration in durations {
            builder.then(duration);
        }
        builder.build()
    }

    /// Creates cues that all start at the schedule origin.
    /// * `durations` — durations of cues sharing the origin.
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

    /// Creates `count` cues separated by `interval`, each with `duration`.
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

    /// Returns the cue identified by `id`, or `None` if it is out of range.
    #[must_use]
    pub fn cue(&self, id: CueId) -> Option<Cue> {
        self.cues.get(id.0).copied()
    }

    /// Returns the end time of the latest cue, or zero for an empty schedule.
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the number of cues.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cues.len()
    }

    /// Returns whether the schedule contains no cues.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
/// Builder for sequential, overlapping, and dependency-relative cues.
pub struct ScheduleBuilder {
    cues: Vec<Cue>,
    cursor: Duration,
    group_start: Duration,
}

impl ScheduleBuilder {
    /// Creates an empty builder with its cursor at the schedule origin.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cues: Vec::new(),
            cursor: Duration::ZERO,
            group_start: Duration::ZERO,
        }
    }

    /// Adds a cue after the current sequence cursor and advances that cursor.
    /// * `duration` — length of the cue to append.
    pub fn then(&mut self, duration: Duration) -> CueId {
        self.group_start = self.cursor;
        let id = self.push(self.cursor, duration);
        self.cursor += duration;
        id
    }

    /// Adds a cue alongside the current group, extending the sequence cursor if needed.
    /// * `duration` — length of the cue added to the current group.
    pub fn with(&mut self, duration: Duration) -> CueId {
        let id = self.push(self.group_start, duration);
        self.cursor = self.cursor.max(self.group_start + duration);
        id
    }

    /// Adds a cue after another cue and an additional `delay`.
    /// * `dependency` — cue establishing the start point; `duration` — new cue length.
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

    /// Builds the schedule from all cues added to this builder.
    #[must_use]
    pub fn build(self) -> Schedule {
        Schedule::from_cues(self.cues)
    }
}
