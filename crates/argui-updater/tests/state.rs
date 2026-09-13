use argui_updater::*;

#[test]
fn progress_handles_unknown_empty_and_inaccurate_totals_without_nan() {
    for (downloaded, total, percent) in [
        (0, None, None),
        (10, Some(0), None),
        (0, Some(10), Some(0.0)),
        (5, Some(10), Some(50.0)),
        (12, Some(10), Some(100.0)),
        (u64::MAX, Some(u64::MAX), Some(100.0)),
    ] {
        assert_eq!(Progress { downloaded, total }.percent(), percent);
    }
}

#[test]
fn release_metadata_and_cancellation_are_independent_of_ui_and_native_features() {
    let info = ReleaseInfo {
        version: "1.0.0".into(),
        notes: String::new(),
    };
    let states = [
        State::Available(info.clone()),
        State::Verifying(info.clone()),
        State::Ready(info.clone()),
        State::Installing(info.clone()),
        State::Cancelled(info.clone()),
        State::Downloading {
            release: info.clone(),
            progress: Progress::default(),
        },
        State::Installed {
            release: info.clone(),
            outcome: InstallOutcome::InstallerLaunched,
        },
    ];
    for state in states {
        assert_eq!(state.release(), Some(&info));
    }
    for state in [
        State::Idle,
        State::Checking,
        State::UpToDate,
        State::Failed("offline".into()),
    ] {
        assert_eq!(state.release(), None);
    }
    let token = CancellationToken::default();
    let clone = token.clone();
    assert!(clone.check().is_ok());
    token.cancel();
    assert_eq!(clone.check(), Err(Error::Cancelled));
    assert_eq!(Error::Cancelled.to_string(), "update cancelled");
    assert_eq!(Error::InvalidState("invalid").to_string(), "invalid");
    assert_eq!(Error::backend("offline").to_string(), "offline");
}
