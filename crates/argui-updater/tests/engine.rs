use argui_updater::*;
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Available,
    Current,
    CheckError,
    DownloadError,
    InstallError,
}
struct Fake {
    mode: Rc<Cell<Mode>>,
    drops: Rc<Cell<usize>>,
}
struct Package(Rc<Cell<usize>>);
impl Drop for Package {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
impl Backend for Fake {
    type Artifact = ();
    type Package = Package;
    fn check(&self) -> Result<Option<Release<()>>> {
        match self.mode.get() {
            Mode::Current => Ok(None),
            Mode::CheckError => Err(Error::backend("offline")),
            _ => Ok(Some(Release {
                info: info(),
                artifact: (),
            })),
        }
    }
    fn download(
        &self,
        _: &(),
        _: &CancellationToken,
        emit: &mut dyn FnMut(DownloadEvent),
    ) -> Result<Package> {
        emit(DownloadEvent::Progress(Progress {
            downloaded: 10,
            total: Some(10),
        }));
        emit(DownloadEvent::Verifying);
        if self.mode.get() == Mode::DownloadError {
            Err(Error::backend("bad signature"))
        } else {
            Ok(Package(self.drops.clone()))
        }
    }
    fn install(&self, _: &(), _: Package) -> Result<InstallOutcome> {
        if self.mode.get() == Mode::InstallError {
            Err(Error::backend("permission denied"))
        } else {
            Ok(InstallOutcome::RestartRequired)
        }
    }
}
fn info() -> ReleaseInfo {
    ReleaseInfo {
        version: "2.0.0".into(),
        notes: "Release notes".into(),
    }
}
fn setup() -> (Updater<Fake>, Rc<Cell<Mode>>, Rc<Cell<usize>>) {
    let mode = Rc::new(Cell::new(Mode::Available));
    let drops = Rc::new(Cell::new(0));
    (
        Updater::new(Fake {
            mode: mode.clone(),
            drops: drops.clone(),
        }),
        mode,
        drops,
    )
}

#[test]
fn explicit_transaction_emits_ordered_states_and_cannot_install_twice() {
    let (mut updater, _, drops) = setup();
    assert_eq!(updater.state(), &State::Idle);
    assert!(matches!(
        updater.install(|_| {}),
        Err(Error::InvalidState(_))
    ));
    assert!(matches!(
        updater.download(&CancellationToken::default(), |_| {}),
        Err(Error::InvalidState(_))
    ));
    let mut states = Vec::new();
    assert!(updater.check(|state| states.push(state.clone())).unwrap());
    assert_eq!(states, [State::Checking, State::Available(info())]);
    assert_eq!(drops.get(), 0);
    updater
        .download(&CancellationToken::default(), |state| {
            states.push(state.clone())
        })
        .unwrap();
    assert!(matches!(&states[2], State::Downloading { progress, .. } if progress.downloaded == 0));
    assert!(matches!(&states[3], State::Downloading { progress, .. } if progress.downloaded == 10));
    assert_eq!(states[4], State::Verifying(info()));
    assert_eq!(updater.state(), &State::Ready(info()));
    assert_eq!(
        updater.install(|state| states.push(state.clone())).unwrap(),
        InstallOutcome::RestartRequired
    );
    assert_eq!(states[6], State::Installing(info()));
    assert!(matches!(updater.state(), State::Installed { .. }));
    assert!(updater.install(|_| {}).is_err());
    assert!(
        updater
            .download(&CancellationToken::default(), |_| {})
            .is_err()
    );
    assert_eq!(drops.get(), 1);
}

#[test]
fn cancellation_discards_even_a_backend_result_that_raced_with_cancel_and_allows_retry() {
    let (mut updater, _, drops) = setup();
    updater.check(|_| {}).unwrap();
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert_eq!(updater.download(&cancelled, |_| {}), Err(Error::Cancelled));
    assert_eq!(drops.get(), 0);
    let cancel = CancellationToken::default();
    assert_eq!(
        updater.download(&cancel, |state| {
            if matches!(state, State::Verifying(_)) {
                cancel.cancel();
            }
        }),
        Err(Error::Cancelled)
    );
    assert_eq!(updater.state(), &State::Cancelled(info()));
    assert_eq!(drops.get(), 1);
    assert!(updater.install(|_| {}).is_err());
    updater
        .download(&CancellationToken::default(), |_| {})
        .unwrap();
    drop(updater);
    assert_eq!(drops.get(), 2);
}

#[test]
fn errors_are_observable_and_rechecks_invalidate_staged_versions() {
    let (mut updater, mode, drops) = setup();
    mode.set(Mode::CheckError);
    assert!(updater.check(|_| {}).is_err());
    assert_eq!(updater.state(), &State::Failed("offline".into()));
    mode.set(Mode::DownloadError);
    updater.check(|_| {}).unwrap();
    assert!(
        updater
            .download(&CancellationToken::default(), |_| {})
            .is_err()
    );
    assert_eq!(updater.state(), &State::Failed("bad signature".into()));
    mode.set(Mode::InstallError);
    updater
        .download(&CancellationToken::default(), |_| {})
        .unwrap();
    assert!(updater.install(|_| {}).is_err());
    assert_eq!(updater.state(), &State::Failed("permission denied".into()));
    assert!(updater.install(|_| {}).is_err());
    mode.set(Mode::Available);
    updater
        .download(&CancellationToken::default(), |_| {})
        .unwrap();
    mode.set(Mode::Current);
    assert!(!updater.check(|_| {}).unwrap());
    assert_eq!(updater.state(), &State::UpToDate);
    assert!(updater.install(|_| {}).is_err());
    assert_eq!(drops.get(), 2);
}
