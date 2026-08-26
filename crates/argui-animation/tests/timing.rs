use argui_animation::{
    Direction, Duration, FillMode, Iterations, Keyframe, Keyframes, Timeline, Timing, TimingError,
};

fn frames() -> Keyframes<f32> {
    Keyframes::new(vec![Keyframe::new(0.0, 0.0), Keyframe::new(1.0, 1.0)]).unwrap()
}

#[test]
fn timing_builders_preserve_every_option() {
    let timing = Timing::new(Duration::from_millis(200))
        .delay(Duration::from_millis(10))
        .end_delay(Duration::from_millis(20))
        .iterations(Iterations::Finite(2.5))
        .direction(Direction::AlternateReverse)
        .fill(FillMode::Both)
        .playback_rate(1.5);
    let timeline = Timeline::new(frames(), timing).unwrap();
    assert_eq!(timeline.timing(), timing);
}

#[test]
fn invalid_timing_is_rejected_with_actionable_errors() {
    let cases = [
        (Timing::new(Duration::ZERO), TimingError::ZeroDuration),
        (
            Timing::new(Duration::from_millis(1)).iterations(Iterations::Finite(0.0)),
            TimingError::InvalidIterations,
        ),
        (
            Timing::new(Duration::from_millis(1)).iterations(Iterations::Finite(f64::NAN)),
            TimingError::InvalidIterations,
        ),
        (
            Timing::new(Duration::from_millis(1)).playback_rate(0.0),
            TimingError::InvalidPlaybackRate,
        ),
        (
            Timing::new(Duration::from_millis(1)).playback_rate(f64::INFINITY),
            TimingError::InvalidPlaybackRate,
        ),
    ];
    for (timing, expected) in cases {
        let error = Timeline::new(frames(), timing).unwrap_err();
        assert_eq!(error, expected);
        assert!(!error.to_string().is_empty());
    }
}
