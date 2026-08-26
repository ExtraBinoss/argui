use argui_animation::{Clock, Duration, ManualClock, Time};

#[test]
fn manual_clock_can_advance_and_seek_exactly() {
    let clock = ManualClock::new(Time::from_nanos(10));
    assert_eq!(clock.now(), Time::from_nanos(10));

    clock.advance(Duration::from_nanos(7));
    assert_eq!(clock.now(), Time::from_nanos(17));

    clock.set(Time::from_nanos(3));
    assert_eq!(clock.now(), Time::from_nanos(3));
}

#[test]
fn default_clock_starts_at_zero() {
    assert_eq!(ManualClock::default().now(), Time::ZERO);
}
