use argui_platform::PlatformEvent;

#[test]
fn only_surface_changes_request_a_frame() {
    assert!(
        PlatformEvent::Opened {
            width: 800,
            height: 600,
            scale_factor: 1.0,
        }
        .requires_redraw()
    );
    assert!(
        PlatformEvent::Resized {
            width: 400,
            height: 300,
        }
        .requires_redraw()
    );
    assert!(PlatformEvent::ScaleFactorChanged(2.0).requires_redraw());
    assert!(!PlatformEvent::RedrawRequested.requires_redraw());
    assert!(!PlatformEvent::Suspended.requires_redraw());
}

#[test]
fn close_and_creation_failure_end_the_window() {
    assert!(PlatformEvent::CloseRequested.closes_window());
    assert!(PlatformEvent::WindowCreationFailed("no display".into()).closes_window());
    assert!(!PlatformEvent::RedrawRequested.closes_window());
}
