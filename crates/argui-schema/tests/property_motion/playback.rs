use super::*;

/// Pausing holds presentation, stops scheduling, and resumes without restarting.
#[test]
fn pause_preserves_phase_and_excludes_elapsed_wall_time() {
    let mut store = PropertyMotionStore::new();
    let spec = PropertyAnimation::new(Some(0.0), Some(100.0), 1000.0, "infinite").unwrap();
    store
        .sample_number(key(1), 0.0, spec.clone(), false)
        .unwrap();
    store.advance(Time::from_nanos(1));
    store.advance(Time::from_nanos(250_000_001));
    let paused = store
        .sample_number(key(1), 0.0, spec.clone().with_playing(false), false)
        .unwrap();
    assert!((paused - 25.0).abs() < 0.001);
    assert!(!store.needs_frame());
    assert!(!store.advance(Time::from_nanos(9_000_000_001)));
    assert_eq!(
        store
            .sample_number(key(1), 0.0, spec.clone().with_playing(false), false)
            .unwrap(),
        paused
    );
    assert_eq!(
        store
            .sample_number(key(1), 0.0, spec.clone(), false)
            .unwrap(),
        paused
    );
    assert!(store.needs_frame());
    store.advance(Time::from_nanos(10_000_000_001));
    assert_eq!(
        store
            .sample_number(key(1), 0.0, spec.clone(), false)
            .unwrap(),
        paused
    );
    store.advance(Time::from_nanos(10_250_000_001));
    let halfway = store.sample_number(key(1), 0.0, spec, false).unwrap();
    assert!((halfway - 50.0).abs() < 0.001);
}

/// Paused dimension/color slots and reduced motion do not schedule background work.
#[test]
fn all_motion_types_can_mount_paused() {
    let mut store = PropertyMotionStore::new();
    let red = Color::from_srgba8(255, 0, 0, 255);
    let blue = Color::from_srgba8(0, 0, 255, 255);
    let spec = PropertyAnimation::new(Some(red), Some(blue), 1000.0, "infinite")
        .unwrap()
        .with_playing(false);
    assert_eq!(store.sample_color(key(2), blue, spec, false).unwrap(), red);
    let spec = PropertyAnimation::new(
        Some(Dimension::length(0.0)),
        Some(Dimension::length(100.0)),
        1000.0,
        "infinite",
    )
    .unwrap()
    .with_playing(false);
    assert_eq!(
        store
            .sample_dimension(key(3), Dimension::length(100.0), spec, false)
            .unwrap(),
        Dimension::length(0.0)
    );
    assert!(!store.needs_frame());
    let spec = PropertyAnimation::new(Some(0.0), Some(100.0), 1000.0, "infinite")
        .unwrap()
        .with_playing(false);
    assert_eq!(
        store
            .sample_number(key(4), 80.0, spec.clone(), true)
            .unwrap(),
        80.0
    );
    assert!(!store.needs_frame());
    store
        .sample_number(key(4), 80.0, spec.with_playing(true), true)
        .unwrap();
    assert!(!store.needs_frame());
}
