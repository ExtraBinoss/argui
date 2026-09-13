use crate::{Error, Result};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseInfo {
    pub version: String,
    pub notes: String,
}

/// Byte counts are cumulative. A missing or zero total is indeterminate.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Progress {
    pub downloaded: u64,
    pub total: Option<u64>,
}

impl Progress {
    #[must_use]
    pub fn percent(self) -> Option<f32> {
        self.total
            .filter(|total| *total > 0)
            .map(|total| (self.downloaded as f64 / total as f64 * 100.0).min(100.0) as f32)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallOutcome {
    /// The application must shut down normally and restart to run the new version.
    RestartRequired,
    /// A system installer was started. Save work and shut down the application normally.
    InstallerLaunched,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(ReleaseInfo),
    Downloading {
        release: ReleaseInfo,
        progress: Progress,
    },
    Verifying(ReleaseInfo),
    Ready(ReleaseInfo),
    Installing(ReleaseInfo),
    Installed {
        release: ReleaseInfo,
        outcome: InstallOutcome,
    },
    Cancelled(ReleaseInfo),
    Failed(String),
}

impl State {
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
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn check(&self) -> Result<()> {
        if self.0.load(Ordering::Acquire) {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}
