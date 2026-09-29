use super::*;
use crate::{AppEvent, AppModel, AppUpdate, WindowEnvironment, app::Application};
use argui_platform::{ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowSpec};

struct EmptyApp;

impl AppModel for EmptyApp {
    fn view(&self, _: &WindowKey, _: WindowEnvironment) -> Option<argui_ui::Element> {
        None
    }
    fn update(&mut self, _: &AppEvent) -> AppUpdate {
        AppUpdate::none()
    }
}

fn application(windows: &[(&str, bool)]) -> MultiApplication {
    let config = ApplicationConfig::new(
        ApplicationIdentity::new(
            ApplicationId::new("dev.argui.activation").unwrap(),
            "Activation",
            IconSet::default(),
        ),
        Default::default(),
    );
    let mut app = MultiApplication::new(config, Default::default(), EmptyApp, |_| {}).unwrap();
    for (name, visible) in windows {
        let key = WindowKey::new(*name);
        let runtime = Application::new(
            Default::default(),
            Default::default(),
            Default::default(),
            None,
            None,
            None,
            |_| {},
        )
        .initially_visible(*visible);
        app.windows.insert(
            key.clone(),
            super::super::WindowEntry {
                spec: WindowSpec::new(key, Default::default()),
                runtime,
                input_region: argui_platform::WindowInputRegion::Full,
            },
        );
    }
    app
}

#[test]
fn open_windows_raise_in_stable_order_with_activated_window_last() {
    let app = application(&[("settings", true), ("main", true), ("teleprompter", true)]);
    let order = app.open_activation_order(&WindowKey::new("settings"));
    assert_eq!(
        order,
        [
            WindowKey::main(),
            WindowKey::new("teleprompter"),
            WindowKey::new("settings")
        ]
    );
}

#[test]
fn minimized_or_occluded_presentations_remain_eligible_but_hidden_scenes_do_not() {
    let mut app = application(&[("settings", true), ("main", true), ("countdown", false)]);
    app.activation
        .observe(&WindowKey::main(), true, Instant::now());
    assert_eq!(
        app.open_activation_order(&WindowKey::new("settings")),
        [WindowKey::main(), WindowKey::new("settings")]
    );
}

#[test]
fn stale_focus_cannot_restore_hidden_or_removed_sources() {
    let app = application(&[("settings", false), ("main", true)]);
    assert!(
        app.open_activation_order(&WindowKey::new("settings"))
            .is_empty()
    );
    assert!(
        app.open_activation_order(&WindowKey::new("removed"))
            .is_empty()
    );
}

#[test]
fn a_single_open_window_does_not_restack_itself() {
    let mut app = application(&[("main", true), ("settings", false)]);
    assert!(app.open_activation_order(&WindowKey::main()).is_empty());
    app.raise_open_windows(&WindowKey::main());
}

#[test]
fn minimizing_a_peer_does_not_immediately_undo_the_minimize_on_focus_fallback() {
    let mut state = WindowActivationState::default();
    let now = Instant::now();
    state.observe(&WindowKey::new("settings"), true, now);
    assert!(!state.may_restore(&WindowKey::main(), now));
    assert!(!state.may_restore(
        &WindowKey::main(),
        now + MINIMIZE_FOCUS_GRACE - Duration::from_nanos(1)
    ));
    assert!(state.may_restore(&WindowKey::main(), now + MINIMIZE_FOCUS_GRACE));
}

#[test]
fn an_explicit_restore_bypasses_focus_fallback_protection() {
    let mut state = WindowActivationState::default();
    let now = Instant::now();
    let settings = WindowKey::new("settings");
    state.observe(&settings, true, now);
    state.observe(&settings, false, now);
    assert!(state.may_restore(&settings, now));
    assert!(!state.may_restore(&settings, now));
}

#[test]
fn observing_an_unchanged_minimized_window_does_not_extend_the_guard() {
    let mut state = WindowActivationState::default();
    let now = Instant::now();
    let settings = WindowKey::new("settings");
    state.observe(&settings, true, now);
    state.observe(&settings, true, now + MINIMIZE_FOCUS_GRACE);
    assert!(state.may_restore(&WindowKey::main(), now + MINIMIZE_FOCUS_GRACE));
}

#[test]
fn a_native_minimize_request_guards_focus_before_the_os_reports_minimization() {
    let mut state = WindowActivationState::default();
    let now = Instant::now();
    state.minimize_requested(now);
    assert!(!state.may_restore(&WindowKey::main(), now));
    assert!(state.may_restore(&WindowKey::main(), now + MINIMIZE_FOCUS_GRACE));
}
