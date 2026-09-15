use crate::{CancellationToken, Error, InstallOutcome, Progress, ReleaseInfo, Result, State};

/// Release metadata and the backend-specific artifact selected for it.
pub struct Release<T> {
    /// User-visible version and release notes.
    pub info: ReleaseInfo,
    /// Opaque artifact consumed by the backend during download and installation.
    pub artifact: T,
}

#[derive(Clone, Copy, Debug)]
/// Progress emitted by a backend while downloading and verifying a package.
pub enum DownloadEvent {
    /// Cumulative downloaded byte counts changed.
    Progress(Progress),
    /// Download has completed and signature verification is in progress.
    Verifying,
}

/// Trusted adapter: downloads must be authenticated before returning a package.
/// Package managers, private APIs and other protocols implement this same contract.
pub trait Backend {
    /// Backend-owned release reference returned by a check.
    type Artifact;
    /// Verified staged package returned by a download.
    type Package;
    /// Checks for a release newer than the running application.
    ///
    /// # Errors
    /// Returns an error if the backend cannot complete or verify the check.
    fn check(&self) -> Result<Option<Release<Self::Artifact>>>;
    /// Downloads and verifies the selected artifact, reporting progress through `emit`.
    ///
    /// # Arguments
    /// * `artifact` — artifact selected by `check`.
    /// * `cancel` — token used to request cancellation.
    /// * `emit` — callback for progress and verification events.
    ///
    /// # Errors
    /// Returns an error if downloading, verification, or cancellation fails.
    fn download(
        &self,
        artifact: &Self::Artifact,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(DownloadEvent),
    ) -> Result<Self::Package>;
    /// Installs a previously verified package.
    ///
    /// # Errors
    /// Returns an error if installation fails.
    /// `artifact` is the release selected by `check`; `package` is its verified downloaded payload.
    fn install(&self, artifact: &Self::Artifact, package: Self::Package) -> Result<InstallOutcome>;
}

/// One update transaction. Exclusive access serializes checks, downloads and installation.
/// Dropping this value also drops any staged package owned by the backend.
pub struct Updater<B: Backend> {
    backend: B,
    state: State,
    release: Option<Release<B::Artifact>>,
    package: Option<B::Package>,
}

impl<B: Backend> Updater<B> {
    /// Creates an idle update transaction using `backend`.
    /// `backend` supplies release lookup, authenticated download, and installation.
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            state: State::Idle,
            release: None,
            package: None,
        }
    }

    /// Returns the current transaction state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Checks for an update and publishes each state transition to `emit`.
    /// `emit` is called with each resulting transaction state.
    ///
    /// Returns `true` when a release is available.
    ///
    /// # Errors
    /// Returns the backend error if the check fails.
    pub fn check(&mut self, mut emit: impl FnMut(&State)) -> Result<bool> {
        self.release = None;
        self.package = None;
        self.state = State::Checking;
        emit(&self.state);
        let result = self.backend.check();
        match result {
            Ok(release) => {
                self.state = release.as_ref().map_or(State::UpToDate, |release| {
                    State::Available(release.info.clone())
                });
                self.release = release;
                emit(&self.state);
                Ok(self.release.is_some())
            }
            Err(error) => self.fail(error, &mut emit),
        }
    }

    /// Downloads and verifies the available release, publishing state changes to `emit`.
    ///
    /// # Arguments
    /// * `cancel` — token that can stop the download.
    /// * `emit` — callback receiving transaction states.
    ///
    /// # Errors
    /// Returns an error if no release is available, cancellation is requested, or download/verification fails.
    pub fn download(
        &mut self,
        cancel: &CancellationToken,
        mut emit: impl FnMut(&State),
    ) -> Result<()> {
        let release = self.release.as_ref().ok_or(Error::InvalidState(
            "check for an update before downloading",
        ))?;
        self.package = None;
        let info = release.info.clone();
        self.state = State::Downloading {
            release: info.clone(),
            progress: Progress::default(),
        };
        emit(&self.state);
        let state = &mut self.state;
        let result = cancel
            .check()
            .and_then(|()| {
                self.backend
                    .download(&release.artifact, cancel, &mut |event| {
                        *state = match event {
                            DownloadEvent::Progress(progress) => State::Downloading {
                                release: info.clone(),
                                progress,
                            },
                            DownloadEvent::Verifying => State::Verifying(info.clone()),
                        };
                        emit(state);
                    })
            })
            .and_then(|package| {
                cancel.check()?;
                Ok(package)
            });
        match result {
            Ok(package) => {
                self.package = Some(package);
                self.state = State::Ready(info);
                emit(&self.state);
                Ok(())
            }
            Err(Error::Cancelled) => {
                self.state = State::Cancelled(info);
                emit(&self.state);
                Err(Error::Cancelled)
            }
            Err(error) => self.fail(error, &mut emit),
        }
    }

    /// Explicit installation only: never called by `check` or `download`.
    /// Installation is not cancellable once it starts. On error, download again before retrying.
    ///
    /// # Errors
    /// Returns an error if there is no staged verified package or installation fails.
    /// `emit` receives each installation state transition.
    ///
    /// # Panics
    /// Panics if the updater's internal invariant is broken and a staged package has no release.
    pub fn install(&mut self, mut emit: impl FnMut(&State)) -> Result<InstallOutcome> {
        let package = self.package.take().ok_or(Error::InvalidState(
            "download and verify an update before installing",
        ))?;
        let release = self
            .release
            .as_ref()
            .expect("a staged package belongs to a release");
        self.state = State::Installing(release.info.clone());
        emit(&self.state);
        match self.backend.install(&release.artifact, package) {
            Ok(outcome) => {
                self.state = State::Installed {
                    release: release.info.clone(),
                    outcome,
                };
                self.release = None;
                emit(&self.state);
                Ok(outcome)
            }
            Err(error) => self.fail(error, &mut emit),
        }
    }

    fn fail<T>(&mut self, error: Error, emit: &mut impl FnMut(&State)) -> Result<T> {
        self.state = State::Failed(error.to_string());
        emit(&self.state);
        Err(error)
    }
}
