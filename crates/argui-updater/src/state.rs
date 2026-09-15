use crate::{Error, Result};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Eq, PartialEq)]
/// User-visible release version and notes.
pub struct ReleaseInfo {
    /// Semantic version string supplied by the trusted backend.
    pub version: String,
    /// Release notes associated with the version.
    pub notes: String,
}

/// Byte counts are cumulative. A missing or zero total is indeterminate.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Progress {
    /// Number of bytes received so far.
    pub downloaded: u64,
    /// Expected total byte count, if known.
    pub total: Option<u64>,
}

impl Progress {
    /// Returns the clamped completion percentage, or `None` for an indeterminate total.
    #[must_use]
    pub fn percent(self) -> Option<f32> {
        self.total
            .filter(|total| *total > 0)
            .map(|total| (self.downloaded as f64 / total as f64 * 100.0).min(100.0) as f32)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Result of an explicit installation request.
pub enum InstallOutcome {
    /// The application must shut down normally and restart to run the new version.
    RestartRequired,
    /// A system installer was started. Save work and shut down the application normally.
    InstallerLaunched,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Current state of an update transaction.
pub enum State {
    #[default]
    /// No update operation is active.
    Idle,
    /// A backend is checking for a newer release.
    Checking,
    /// The backend found no applicable update.
    UpToDate,
    /// A newer release is available for download.
    Available(ReleaseInfo),
    /// The release artifact is being downloaded.
    Downloading {
        /// Metadata for the release being downloaded.
        release: ReleaseInfo,
        /// Current download byte counts.
        progress: Progress,
    },
    /// The downloaded artifact is being authenticated.
    Verifying(ReleaseInfo),
    /// A verified package is staged and ready for explicit installation.
    Ready(ReleaseInfo),
    /// The staged package is being installed.
    Installing(ReleaseInfo),
    /// Installation completed with the given restart outcome.
    Installed {
        /// Metadata for the installed release.
        release: ReleaseInfo,
        /// Whether the app must restart or an installer was launched.
        outcome: InstallOutcome,
    },
    /// Download was cancelled.
    Cancelled(ReleaseInfo),
    /// An operation failed with the displayed error message.
    Failed(String),
}

impl State {
    /// Returns release metadata for states associated with a release.
    #[must_use]
    pub fn release(&self) -> Option<&ReleaseInfo> {
        match self {
            Self::Available(info)
            | Self::Verifying(info)
            | Self::Ready(info)
            | Self::Installing(info)
            | Self::Cancelled(info)
            | Self::Downloading { release: info, .. }
            | Self::Installed { release: info, .. } => Some(info),
            _ => None,
        }
    }
}

/// Cooperative cancellation checked between network reads and before installation.
/// Construct a fresh token for a retry; cancellation cannot be reset.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    /// Requests cancellation of operations using this token.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Checks whether cancellation has been requested.
    ///
    /// # Errors
    /// Returns [`Error::Cancelled`] after cancellation was requested.
    pub fn check(&self) -> Result<()> {
        if self.0.load(Ordering::Acquire) {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}
