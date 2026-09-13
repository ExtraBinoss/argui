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
pub enum Error {
    #[error("update cancelled")]
    Cancelled,
    #[error("{0}")]
    InvalidState(&'static str),
    #[error("{0}")]
    Backend(String),
}

impl Error {
    pub fn backend(error: impl std::fmt::Display) -> Self {
        Self::Backend(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
