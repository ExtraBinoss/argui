use crate::InspectorHandle;

/// Known allocated capacities, sampled independently from the frame history.
/// These are allocator payload sizes, not resident pages; do not subtract them
/// from RSS or include them again in a combined process/GPU total.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MemorySnapshot {
    pub ui_nodes: usize,
    pub layout_nodes: usize,
    pub ui_index_bytes: usize,
    pub layout_metadata_bytes: usize,
    pub layout_cache_bytes: usize,
    pub layout_geometry_bytes: usize,
    pub layout_output_bytes: usize,
    pub paint_command_bytes: usize,
}

impl InspectorHandle {
    /// Requests a memory-capacity sample from the next eligible host update.
    pub fn request_memory_sample(&self) {
        let mut state = self.0.borrow_mut();
        state.memory_requested = state.recording && !state.paused;
    }

    /// Consumes a pending memory-sample request, if recording is currently active.
    pub fn take_memory_request(&self) -> bool {
        let mut state = self.0.borrow_mut();
        let requested = std::mem::take(&mut state.memory_requested);
        requested && state.recording && !state.paused
    }

    /// Publishes the latest allocator-capacity snapshot.
    pub fn publish_memory(&self, snapshot: MemorySnapshot) {
        self.0.borrow_mut().memory = Some(snapshot);
    }

    /// Returns the most recently published capacity snapshot, if available.
    #[must_use]
    pub fn memory(&self) -> Option<MemorySnapshot> {
        self.0.borrow().memory
    }
}
