use argui_core::{
    Point, PointerButton, PointerEvent, PointerId, PointerKind, PointerPhase, ScrollDelta,
};

#[test]
fn scroll_units_stay_explicit_until_ui_dispatch() {
    assert_ne!(
        ScrollDelta::Lines(Point::new(0.0, 1.0)),
        ScrollDelta::Pixels(Point::new(0.0, 1.0))
    );
}

#[test]
fn pointer_events_keep_device_identity_and_contact_data() {
    let event = PointerEvent {
        id: PointerId::new(42),
        kind: PointerKind::Pen,
        phase: PointerPhase::Moved,
        position: Point::new(12.0, 24.0),
        button: Some(PointerButton::Primary),
        buttons: 1,
        pressure: Some(0.75),
        primary: true,
        timestamp: std::time::Duration::from_millis(8),
    };

    assert_eq!(event.id.get(), 42);
    assert_eq!(event.pressure, Some(0.75));
    assert_eq!(event.kind, PointerKind::Pen);
}
