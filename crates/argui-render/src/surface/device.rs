use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DEVICE_GENERATION: AtomicU64 = AtomicU64::new(1);

/// Allocates an opaque process-local device generation for cache diagnostics.
///
/// # Panics
///
/// Panics if the generation identity space is exhausted.
pub(super) fn next_device_generation() -> u64 {
    let generation = NEXT_DEVICE_GENERATION.fetch_add(1, Ordering::Relaxed);
    assert_ne!(generation, u64::MAX, "renderer device generation exhausted");
    generation
}
