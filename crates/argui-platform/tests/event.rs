use argui_core::Point;
use argui_platform::{ImeInput, PlatformEvent, PointerEvent, PointerPhase, ScrollDelta};

#[test]
fn surface_changes_and_returning_to_a_window_request_a_frame() {
    assert!(
        PlatformEvent::Opened {
            width: 800,
            height: 600,
            scale_factor: 1.0,
            capabilities: argui_platform::WindowBackend::X11.capabilities(),
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
    assert!(
        PlatformEvent::SafeAreaChanged(argui_platform::Insets::new(10.0, 0.0, 20.0, 0.0))
            .requires_redraw()
    );
    // A compositor may stop frame callbacks while another window covers the surface.
    // Re-entry must kick presentation even when no focused widget changes visually.
    assert!(PlatformEvent::Focused(true).requires_redraw());
    assert!(PlatformEvent::VisibilityChanged(true).requires_redraw());
    assert!(!PlatformEvent::Focused(false).requires_redraw());
    assert!(!PlatformEvent::VisibilityChanged(false).requires_redraw());
    assert!(!PlatformEvent::RedrawRequested.requires_redraw());
    assert!(!PlatformEvent::Suspended.requires_redraw());
    assert!(
        !PlatformEvent::Pointer(PointerEvent::mouse(PointerPhase::Pressed, Point::default()))
            .requires_redraw()
    );
    assert!(
        !PlatformEvent::PointerScrolled(ScrollDelta::Lines(Point::new(0.0, -1.0)))
            .requires_redraw()
    );
    assert!(!PlatformEvent::Ime(ImeInput::Enabled).requires_redraw());
}

#[test]
fn close_and_creation_failure_end_the_window() {
    assert!(PlatformEvent::CloseRequested.closes_window());
    assert!(PlatformEvent::WindowCreationFailed("no display".into()).closes_window());
    assert!(!PlatformEvent::RedrawRequested.closes_window());
}
