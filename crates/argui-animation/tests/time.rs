use argui_animation::{Duration, Time};

#[test]
fn duration_constructors_and_arithmetic_are_exact() {
    assert_eq!(Duration::from_micros(2).as_nanos(), 2_000);
    assert_eq!(Duration::from_millis(3).as_nanos(), 3_000_000);
    assert_eq!(Duration::from_secs(4).as_nanos(), 4_000_000_000);
    assert_eq!(Duration::from_millis(500).as_secs_f64(), 0.5);

    let mut duration = Duration::from_millis(8);
    duration += Duration::from_millis(5);
    assert_eq!(duration, Duration::from_millis(13));
    assert_eq!(duration - Duration::from_millis(20), Duration::ZERO);
}

#[test]
fn duration_arithmetic_saturates() {
    assert_eq!(
        (Duration::from_nanos(u64::MAX) + Duration::from_nanos(1)).as_nanos(),
        u64::MAX
    );
    assert_eq!(Duration::from_micros(u64::MAX).as_nanos(), u64::MAX);
    assert_eq!(Duration::from_millis(u64::MAX).as_nanos(), u64::MAX);
    assert_eq!(Duration::from_secs(u64::MAX).as_nanos(), u64::MAX);
}

#[test]
fn time_is_monotonic_and_saturating() {
    let start = Time::from_nanos(40);
    let end = start + Duration::from_nanos(2);
    assert_eq!(end.as_nanos(), 42);
    assert_eq!(end - start, Duration::from_nanos(2));
    assert_eq!(start.duration_since(end), Duration::ZERO);
    assert_eq!(
        (Time::from_nanos(u64::MAX) + Duration::from_nanos(1)).as_nanos(),
        u64::MAX
    );
}

#[test]
fn standard_duration_conversion_is_bounded() {
    let standard = std::time::Duration::from_millis(17);
    let animation = Duration::from(standard);
    assert_eq!(std::time::Duration::from(animation), standard);

    let oversized = std::time::Duration::new(u64::MAX, 999_999_999);
    assert_eq!(Duration::from(oversized).as_nanos(), u64::MAX);
}
