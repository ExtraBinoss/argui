use crate::{CancellationToken, Error, InstallOutcome, Progress, ReleaseInfo, Result, State};

pub struct Release<T> {
    pub info: ReleaseInfo,
    pub artifact: T,
}

#[derive(Clone, Copy, Debug)]
pub enum DownloadEvent {
    Progress(Progress),
    Verifying,
}

/// Trusted adapter: downloads must be authenticated before returning a package.
/// Package managers, private APIs and other protocols implement this same contract.
pub trait Backend {
    type Artifact;
    type Package;
    fn check(&self) -> Result<Option<Release<Self::Artifact>>>;
    fn download(
        &self,
        artifact: &Self::Artifact,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(DownloadEvent),
    ) -> Result<Self::Package>;
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
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            state: State::Idle,
            release: None,
            package: None,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

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
