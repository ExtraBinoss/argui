use crate::{FrameRecord, InspectorHandle};

/// Position of a frontend snapshot in the bounded recording history.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameCursor {
    sequence: u64,
    epoch: u64,
}

impl InspectorHandle {
    /// Enables detailed renderer measurements independently of tree inspection.
    pub fn set_gpu_profiling(&self, enabled: bool) {
        self.0.borrow_mut().gpu_profiling = enabled;
    }

    #[must_use]
    pub fn gpu_profiling(&self) -> bool {
        let state = self.0.borrow();
        state.gpu_profiling && state.recording && !state.paused
    }

    /// Synchronizes new frames and delayed render results without cloning unchanged records.
    pub fn sync_frames(
        &self,
        destination: &mut Vec<FrameRecord>,
        cursor: &mut FrameCursor,
    ) -> bool {
        let state = self.0.borrow();
        let added = if cursor.epoch == state.frame_epoch {
            state
                .frame_sequence
                .wrapping_sub(cursor.sequence)
                .min(usize::MAX as u64) as usize
        } else {
            state.frames.len()
        };
        let retained = destination
            .len()
            .min(state.frames.len().saturating_sub(added));
        let removed = destination.len() - retained;
        let mut changed = removed > 0 || destination.len() != state.frames.len() || added > 0;
        destination.drain(..removed);
        for (existing, recorded) in destination.iter_mut().zip(&state.frames) {
            if existing != recorded {
                existing.clone_from(recorded);
                changed = true;
            }
        }
        destination.extend(state.frames.iter().skip(retained).cloned());
        cursor.sequence = state.frame_sequence;
        cursor.epoch = state.frame_epoch;
        changed
    }

    /// Whether tree inspection is attached, independently of recording pause.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.0.borrow().recording
    }
}
