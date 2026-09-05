use argui_core::{
    Point, PointerButton, PointerEvent, PointerId, PointerKind, PointerPhase, PointerSettings,
    ScrollDelta,
};

#[test]
fn scroll_units_stay_explicit_until_ui_dispatch() {
    assert_ne!(
        ScrollDelta::Lines(Point::new(0.0, 1.0)),
        ScrollDelta::Pixels(Point::new(0.0, 1.0))
    );
}

#[test]
fn pointer_thresholds_are_configurable_as_one_policy() {
    let settings = PointerSettings::default()
        .multi_click(std::time::Duration::from_millis(240), 3.0)
        .long_press(std::time::Duration::from_millis(650), 12.0);
    assert_eq!(
        settings.multi_click_interval(),
        std::time::Duration::from_millis(240)
    );
    assert_eq!(settings.multi_click_distance(), 3.0);
    assert_eq!(
        settings.long_press_interval(),
        std::time::Duration::from_millis(650)
    );
    assert_eq!(settings.touch_slop(), 12.0);
}

#[test]
#[should_panic]
fn multi_click_distance_rejects_negative_values() {
    let _ = PointerSettings::default().multi_click(std::time::Duration::from_millis(240), -1.0);
}

#[test]
#[should_panic]
fn multi_click_distance_rejects_nan() {
    let _ = PointerSettings::default().multi_click(std::time::Duration::from_millis(240), f32::NAN);
}

#[test]
#[should_panic]
fn touch_slop_rejects_negative_values() {
    let _ = PointerSettings::default().long_press(std::time::Duration::from_millis(650), -1.0);
}

#[test]
#[should_panic]
fn touch_slop_rejects_nan() {
    let _ = PointerSettings::default().long_press(std::time::Duration::from_millis(650), f32::NAN);
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
        modifiers: argui_core::Modifiers::default(),
        timestamp: std::time::Duration::from_millis(8),
    };

    assert_eq!(event.id.get(), 42);
    assert_eq!(event.pressure, Some(0.75));
    assert_eq!(event.kind, PointerKind::Pen);
}
