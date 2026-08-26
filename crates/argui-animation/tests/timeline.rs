use argui_animation::{
    Direction, Duration, Easing, FillMode, Iterations, Keyframe, Keyframes, PlaybackState, Time,
    Timeline, Timing, TimingError,
};

fn timeline(timing: Timing) -> Timeline<f32> {
    Timeline::new(
        Keyframes::new(vec![Keyframe::new(0.0, 0.0), Keyframe::new(1.0, 100.0)]).unwrap(),
        timing,
    )
    .unwrap()
}

fn milliseconds(value: u64) -> Time {
    Time::from_nanos(value * 1_000_000)
}

fn close(actual: Option<f32>, expected: f32) {
    assert!((actual.unwrap() - expected).abs() < 1.0e-4);
}

#[test]
fn delay_fill_and_lifecycle_events_are_deterministic() {
    let mut animation = timeline(
        Timing::new(Duration::from_millis(100))
            .delay(Duration::from_millis(50))
            .end_delay(Duration::from_millis(20))
            .fill(FillMode::Both),
    );
    assert_eq!(animation.sample(Time::ZERO).value, None);
    animation.play(Time::ZERO);
    assert!(animation.needs_frame());

    let before = animation.sample(milliseconds(20));
    assert_eq!(before.value, Some(0.0));
    assert!(!before.events.started);
    let middle = animation.sample(milliseconds(100));
    assert_eq!(middle.value, Some(50.0));
    assert!(middle.events.started);
    let after = animation.sample(milliseconds(160));
    assert_eq!(after.value, Some(100.0));
    assert_eq!(after.state, PlaybackState::Running);
    let finished = animation.sample(milliseconds(170));
    assert!(finished.events.finished);
    assert_eq!(finished.state, PlaybackState::Finished);
    assert!(!animation.needs_frame());
}

#[test]
fn alternate_iterations_report_crossed_boundaries() {
    let mut animation = timeline(
        Timing::new(Duration::from_millis(100))
            .iterations(Iterations::Finite(3.0))
            .direction(Direction::Alternate)
            .fill(FillMode::Forwards),
    );
    animation.play(Time::ZERO);
    assert_eq!(animation.sample(milliseconds(25)).value, Some(25.0));
    let second = animation.sample(milliseconds(125));
    assert_eq!(second.value, Some(75.0));
    assert_eq!(second.events.iterations, 1);
    let final_sample = animation.sample(milliseconds(300));
    assert_eq!(final_sample.value, Some(100.0));
    assert_eq!(final_sample.events.iterations, 1);
    assert!(final_sample.events.finished);
}

#[test]
fn pause_resume_seek_rate_reverse_finish_and_cancel_are_explicit() {
    let mut animation = timeline(Timing::new(Duration::from_millis(100)).fill(FillMode::Both));
    animation.play(Time::ZERO);
    animation.pause(milliseconds(20));
    assert_eq!(animation.state(), PlaybackState::Paused);
    assert_eq!(animation.sample(milliseconds(80)).value, Some(20.0));
    animation.resume(milliseconds(80));
    close(animation.sample(milliseconds(90)).value, 30.0);

    animation.seek(Duration::from_millis(50), milliseconds(90));
    animation.set_playback_rate(2.0, milliseconds(90)).unwrap();
    close(animation.sample(milliseconds(100)).value, 70.0);
    assert_eq!(
        animation.set_playback_rate(0.0, milliseconds(100)),
        Err(TimingError::InvalidPlaybackRate)
    );

    animation.reverse(milliseconds(100));
    close(animation.sample(milliseconds(110)).value, 50.0);
    animation.finish();
    let finished = animation.sample(milliseconds(110));
    assert!(finished.events.finished);
    assert_eq!(finished.value, Some(0.0));

    animation.restart(milliseconds(200));
    animation.cancel();
    let canceled = animation.sample(milliseconds(200));
    assert!(canceled.events.canceled);
    assert_eq!(canceled.state, PlaybackState::Canceled);
    assert!(!animation.sample(milliseconds(200)).events.canceled);
}

#[test]
fn reverse_and_fractional_directions_resolve_final_progress() {
    for (direction, expected) in [
        (Direction::Normal, 50.0),
        (Direction::Reverse, 50.0),
        (Direction::Alternate, 50.0),
        (Direction::AlternateReverse, 50.0),
    ] {
        let mut animation = timeline(
            Timing::new(Duration::from_millis(100))
                .iterations(Iterations::Finite(1.5))
                .direction(direction)
                .fill(FillMode::Forwards),
        );
        animation.play(Time::ZERO);
        assert_eq!(animation.sample(milliseconds(150)).value, Some(expected));
    }

    let mut infinite = timeline(
        Timing::new(Duration::from_millis(100))
            .iterations(Iterations::Infinite)
            .direction(Direction::AlternateReverse),
    );
    infinite.play(Time::ZERO);
    assert_eq!(infinite.sample(milliseconds(25)).value, Some(75.0));
    assert!(infinite.needs_frame());
}

#[test]
fn fill_modes_apply_only_outside_the_active_interval() {
    for (fill, before, after) in [
        (FillMode::None, None, None),
        (FillMode::Forwards, None, Some(100.0)),
        (FillMode::Backwards, Some(0.0), None),
        (FillMode::Both, Some(0.0), Some(100.0)),
    ] {
        let mut animation = timeline(
            Timing::new(Duration::from_millis(100))
                .delay(Duration::from_millis(20))
                .end_delay(Duration::from_millis(20))
                .fill(fill),
        );
        animation.play(Time::ZERO);
        assert_eq!(animation.sample(milliseconds(10)).value, before);
        assert_eq!(animation.sample(milliseconds(130)).value, after);
        assert_eq!(animation.sample(milliseconds(140)).value, after);
    }
}

#[test]
fn retargeting_starts_from_the_presented_value_without_a_jump() {
    let mut animation = timeline(Timing::new(Duration::from_millis(100)).fill(FillMode::Both));
    animation.play(Time::ZERO);
    assert_eq!(animation.sample(milliseconds(50)).value, Some(50.0));
    animation
        .retarget(
            200.0,
            Duration::from_millis(100),
            Easing::Linear,
            milliseconds(50),
        )
        .unwrap();
    assert_eq!(animation.sample(milliseconds(50)).value, Some(50.0));
    assert_eq!(animation.sample(milliseconds(100)).value, Some(125.0));
    assert_eq!(animation.sample(milliseconds(150)).value, Some(200.0));
}
