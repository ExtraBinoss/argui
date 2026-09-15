//! Optional application update engine. No UI, executor, startup hook or hidden network work.
//!
//! Call `Updater::check` once from the application's startup task. Operations are blocking:
//! run them on a worker and deliver the state callback to your UI's event loop.
//! The `native` feature supplies HTTPS manifests, streaming Minisign verification and
//! desktop installation. Custom backends can use any release service or package manager.

mod engine;
mod state;
pub use engine::{Backend, DownloadEvent, Release, Updater};
pub use state::{CancellationToken, InstallOutcome, Progress, ReleaseInfo, State};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod http;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod install;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
/// Errors produced by the update transaction and its backends.
pub enum Error {
    #[error("update cancelled")]
    /// The operation observed a cancellation request.
    Cancelled,
    #[error("{0}")]
    /// An operation was called before its required prior state.
    InvalidState(&'static str),
    #[error("{0}")]
    /// A backend operation failed.
    Backend(String),
}

impl Error {
    /// Converts a backend error to the updater's owned error type.
    /// `error` is formatted into an owned message.
    pub fn backend(error: impl std::fmt::Display) -> Self {
        Self::Backend(error.to_string())
    }
}

/// Result type used by updater operations and backend implementations.
pub type Result<T> = std::result::Result<T, Error>;
