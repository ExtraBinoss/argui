//! Owned executor for Linux portal calls, independent of the application's executor.

use std::sync::OnceLock;

static RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();

/// Returns the persistent executor that owns portal connection background tasks.
///
/// # Errors
/// Returns the OS error if the runtime's worker threads cannot be created.
pub(crate) fn runtime() -> Result<&'static tokio::runtime::Runtime, String> {
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .thread_name("argui-portal")
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)
}
