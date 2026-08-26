use crate::{Duration, Time};

/// Stable identity for one scheduled animation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AnimationId(u64);

/// Timing shared by every animation sampled during one presentation frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Frame {
    pub now: Time,
    pub elapsed: Duration,
}

/// Compact storage containing active animations only.
#[derive(Debug, Default)]
pub struct Scheduler {
    active: Vec<AnimationId>,
    next_id: u64,
    last_frame: Option<Time>,
}

impl Scheduler {
    #[must_use]
    pub fn start(&mut self) -> AnimationId {
        let id = AnimationId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        self.active.push(id);
        id
    }

    pub fn stop(&mut self, id: AnimationId) -> bool {
        let Some(index) = self.active.iter().position(|active| *active == id) else {
            return false;
        };
        self.active.swap_remove(index);
        if self.active.is_empty() {
            self.last_frame = None;
        }
        true
    }

    #[must_use]
    pub fn contains(&self, id: AnimationId) -> bool {
        self.active.contains(&id)
    }

    #[must_use]
    pub fn active(&self) -> &[AnimationId] {
        &self.active
    }

    #[must_use]
    pub fn needs_frame(&self) -> bool {
        !self.active.is_empty()
    }

    pub fn frame(&mut self, now: Time) -> Option<Frame> {
        self.needs_frame().then(|| {
            let elapsed = self
                .last_frame
                .map_or(Duration::ZERO, |last| now.duration_since(last));
            self.last_frame = Some(now);
            Frame { now, elapsed }
        })
    }
}
