use crate::graph;

/// Runs an operation inside a nested reactive transaction.
///
/// Property writes are visible immediately to reads, while observers and
/// computed reevaluation are coalesced until the outermost transaction exits.
/// If `operation` unwinds, completed writes remain and are flushed during
/// unwinding; transactions are batching boundaries, not rollback boundaries.
///
/// * `operation` — property mutations and reads to execute as one notification batch.
///
/// Returns the value returned by `operation`.
pub fn transaction<R>(operation: impl FnOnce() -> R) -> R {
    graph::begin_transaction();
    struct EndTransaction;
    impl Drop for EndTransaction {
        fn drop(&mut self) {
            graph::end_transaction();
        }
    }
    let guard = EndTransaction;
    let result = operation();
    drop(guard);
    result
}
