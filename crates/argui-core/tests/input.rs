use argui_core::{
    PinchRecognizer, Point, PointerButton, PointerEvent, PointerId, PointerKind, PointerPhase,
    PointerSettings, ScrollDelta,
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
        .activation_slop(8.0)
        .long_press(std::time::Duration::from_millis(650), 12.0);
    assert_eq!(
        settings.multi_click_interval(),
        std::time::Duration::from_millis(240)
    );
    assert_eq!(settings.multi_click_distance(), 3.0);
    assert_eq!(settings.activation_slop_distance(), 8.0);
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
#[should_panic]
fn activation_slop_rejects_negative_values() {
    let _ = PointerSettings::default().activation_slop(-1.0);
}

#[test]
#[should_panic]
fn activation_slop_rejects_nan() {
    let _ = PointerSettings::default().activation_slop(f32::NAN);
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

#[test]
fn pinch_recognizer_reports_incremental_scale_and_consumes_until_empty() {
    let first = PointerId::new(1);
    let second = PointerId::new(2);
    let mut pinch = PinchRecognizer::default();

    assert!(
        !pinch
            .observe(first, PointerPhase::Pressed, Point::new(0.0, 0.0))
            .consumed
    );
    let started = pinch.observe(second, PointerPhase::Pressed, Point::new(100.0, 0.0));
    assert!(started.started && started.consumed);
    assert_eq!(started.scale, None);

    let expanded = pinch.observe(second, PointerPhase::Moved, Point::new(150.0, 0.0));
    assert_eq!(expanded.scale, Some(1.5));
    assert!(expanded.consumed);
    let contracted = pinch.observe(first, PointerPhase::Entered, Point::new(30.0, 0.0));
    assert_eq!(contracted.scale, Some(0.8));

    assert!(
        pinch
            .observe(first, PointerPhase::Released, Point::new(30.0, 0.0))
            .consumed
    );
    assert!(
        pinch
            .observe(second, PointerPhase::Cancelled, Point::new(150.0, 0.0))
            .consumed
    );
    assert!(
        !pinch
            .observe(first, PointerPhase::Pressed, Point::new(4.0, 8.0))
            .consumed
    );
}

#[test]
fn pinch_recognizer_ignores_coincident_contacts_and_rebases_replacement_contacts() {
    let first = PointerId::new(1);
    let second = PointerId::new(2);
    let third = PointerId::new(3);
    let mut pinch = PinchRecognizer::default();

    let _ = pinch.observe(first, PointerPhase::Entered, Point::new(10.0, 10.0));
    let coincident = pinch.observe(second, PointerPhase::Pressed, Point::new(10.0, 10.0));
    assert_eq!(coincident.scale, None);
    assert!(!coincident.started);

    let started = pinch.observe(second, PointerPhase::Moved, Point::new(30.0, 10.0));
    assert!(started.started);
    let _ = pinch.observe(third, PointerPhase::Pressed, Point::new(50.0, 10.0));
    let replacement = pinch.observe(first, PointerPhase::Left, Point::new(10.0, 10.0));
    assert_eq!(replacement.scale, None);
    assert!(!replacement.started);
    assert!(replacement.consumed);

    let moved = pinch.observe(third, PointerPhase::Moved, Point::new(70.0, 10.0));
    assert_eq!(moved.scale, Some(2.0));
    assert!(
        pinch
            .observe(second, PointerPhase::Released, Point::new(30.0, 10.0))
            .consumed
    );
    assert!(
        pinch
            .observe(third, PointerPhase::Left, Point::new(70.0, 10.0))
            .consumed
    );
}
