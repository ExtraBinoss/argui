use super::*;

/// Creates the real model-free runtime without an OS surface or GPU.
fn application() -> Application {
    Application::new(
        Default::default(),
        Default::default(),
        Default::default(),
        None,
        None,
        None,
        |_| {},
    )
}

#[test]
fn returning_focus_recovers_stale_occlusion_and_schedules_fresh_paint() {
    let mut app = application();
    app.occluded = true;
    app.sync_host_visibility();
    assert!(!app.presentation_visible);
    app.host_focus_changed(true);
    assert!(app.presentation_visible);
    assert!(!app.occluded);
    assert!(app.pending_ui_frame.needs_frame());
}

#[test]
fn returning_focus_repaints_even_without_a_visibility_change() {
    let mut app = application();
    assert!(app.presentation_visible && !app.occluded);
    assert!(!app.pending_ui_frame.needs_frame());
    app.host_focus_changed(true);
    assert!(app.presentation_visible && !app.occluded);
    assert!(app.pending_ui_frame.needs_frame());
}

#[test]
fn focus_does_not_remap_an_explicitly_hidden_window() {
    let mut app = application();
    app.set_window_visible(false);
    app.host_focus_changed(true);
    assert!(!app.presentation_visible);
    assert!(!app.initial_visible);
}

#[test]
fn compositor_activation_resumes_a_wayland_window_hidden_by_minimization() {
    let mut app = application();
    app.set_window_visible(false);
    app.host_focus_changed_on_backend(true, Some(WindowBackend::Wayland));
    assert!(app.initial_visible && app.presentation_visible);
    assert!(app.pending_ui_frame.needs_frame());
}

#[test]
fn pointer_entry_resumes_restored_wayland_surface_without_an_occlusion_or_focus_event() {
    let mut app = application();
    app.set_window_visible(false);
    assert!(!app.occluded);
    app.host_pointer_entered_on_backend(Some(WindowBackend::Wayland));
    assert!(app.initial_visible && app.presentation_visible);
    assert!(app.pending_ui_frame.needs_frame());
}

#[test]
fn pointer_entry_on_x11_does_not_unhide_a_window() {
    let mut app = application();
    app.set_window_visible(false);
    app.host_pointer_entered_on_backend(Some(WindowBackend::X11));
    assert!(!app.initial_visible && !app.presentation_visible);
}

#[test]
fn focus_loss_or_activation_on_other_backends_does_not_unhide_a_window() {
    for backend in [WindowBackend::Wayland, WindowBackend::X11] {
        let mut app = application();
        app.set_window_visible(false);
        app.host_focus_changed_on_backend(false, Some(backend));
        assert!(!app.initial_visible && !app.presentation_visible);
    }
    let mut app = application();
    app.set_window_visible(false);
    app.host_focus_changed_on_backend(true, Some(WindowBackend::X11));
    assert!(!app.initial_visible && !app.presentation_visible);
}

#[test]
fn losing_focus_preserves_occlusion_and_does_not_schedule_idle_work() {
    let mut app = application();
    app.occluded = true;
    app.host_focus_changed(false);
    assert!(app.occluded);
    assert!(!app.presentation_visible);
    assert!(!app.pending_ui_frame.needs_frame());
}

#[test]
fn pointer_entry_recovers_occlusion_without_a_new_focus_event() {
    let mut app = application();
    app.host_occlusion_changed(true);
    assert!(!app.presentation_visible);
    app.host_pointer_entered();
    assert!(app.presentation_visible);
    assert!(!app.occluded);
    assert!(app.pending_ui_frame.needs_frame());
}

#[test]
fn clearing_occlusion_schedules_the_first_frame_without_user_input() {
    let mut app = application();
    app.host_occlusion_changed(true);
    app.host_occlusion_changed(false);
    assert!(app.presentation_visible);
    assert!(app.pending_ui_frame.needs_frame());
}

#[test]
fn pointer_entry_does_not_repaint_a_visible_idle_window_or_unhide_a_hidden_one() {
    let mut app = application();
    app.host_pointer_entered();
    assert!(!app.pending_ui_frame.needs_frame());
    app.set_window_visible(false);
    app.occluded = true;
    app.host_pointer_entered();
    assert!(!app.presentation_visible);
    assert!(!app.initial_visible);
}
