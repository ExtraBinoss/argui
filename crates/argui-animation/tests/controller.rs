use argui_animation::{
    Duration, Easing, Keyframe, Keyframes, Motion, MotionState, MotionTrack, SpringConfig, Time,
    Timeline, Timing, Tween,
};

#[test]
fn tween_retargets_from_the_presented_value() {
    let motion = Motion::new(0.0_f32);
    motion.animate_to(
        10.0,
        Tween::new(Duration::from_millis(100)).easing(Easing::Linear),
    );
    assert!(!MotionTrack::advance(&motion, Time::from_nanos(1)));
    assert!(MotionTrack::advance(&motion, Time::from_nanos(50_000_001)));
    let presented = motion.value();
    assert!((presented - 5.0).abs() < 0.001);

    motion.animate_to(20.0, Tween::new(Duration::from_millis(100)));
    assert!(!MotionTrack::advance(&motion, Time::from_nanos(60_000_000)));
    assert_eq!(motion.value(), presented);
    assert!(MotionTrack::advance(&motion, Time::from_nanos(110_000_000)));
    assert!((motion.value() - 12.5).abs() < 0.001);
}

#[test]
fn playing_a_low_level_timeline_finishes_at_its_terminal_value() {
    let timeline = Timeline::new(
        Keyframes::new([Keyframe::new(0.0, 2.0_f32), Keyframe::new(1.0, 12.0_f32)]).unwrap(),
        Timing::new(Duration::from_millis(100)),
    )
    .unwrap();
    let motion = Motion::new(2.0_f32);
    motion.play(timeline);

    assert!(!MotionTrack::advance(&motion, Time::from_nanos(1)));
    assert!(MotionTrack::advance(&motion, Time::from_nanos(100_000_001)));
    assert_eq!(motion.value(), 12.0);
    assert_eq!(motion.target(), 12.0);
    assert_eq!(motion.state(), MotionState::Finished);
}

#[test]
fn spring_retarget_preserves_velocity() {
    let motion = Motion::new(0.0_f32);
    motion.spring_to(10.0, SpringConfig::default()).unwrap();
    MotionTrack::advance(&motion, Time::from_nanos(1));
    MotionTrack::advance(&motion, Time::from_nanos(16_000_001));
    let velocity = motion.velocity();
    assert!(velocity > 0.0);
    let presented = motion.value();

    motion.spring_to(-4.0, SpringConfig::default()).unwrap();
    assert_eq!(motion.value(), presented);
    assert_eq!(motion.velocity(), velocity);
}

#[test]
fn controls_have_explicit_terminal_values() {
    let motion = Motion::new(2.0_f32);
    motion.animate_to(8.0, Tween::new(Duration::from_secs(1)));
    motion.pause();
    assert_eq!(motion.state(), MotionState::Paused);
    motion.resume();
    assert_eq!(motion.state(), MotionState::Running);
    motion.cancel();
    assert_eq!(motion.state(), MotionState::Canceled);
    assert_eq!(motion.value(), 2.0);

    motion.animate_to(8.0, Tween::new(Duration::from_secs(1)));
    motion.finish();
    assert_eq!(motion.state(), MotionState::Finished);
    assert_eq!(motion.value(), 8.0);
    assert!(!motion.is_active());

    motion.set(3.0);
    assert_eq!(motion.state(), MotionState::Idle);
    assert_eq!(motion.value(), 3.0);
}

#[test]
fn zero_duration_tweens_finish_synchronously() {
    let motion = Motion::new(1.0_f32);
    motion.animate_to(9.0, Tween::new(Duration::ZERO));

    assert_eq!(motion.value(), 9.0);
    assert_eq!(motion.target(), 9.0);
    assert_eq!(motion.state(), MotionState::Finished);
    assert!(!motion.is_active());
}

#[test]
fn delayed_unfilled_timeline_keeps_the_presented_value_until_it_starts() {
    let animation = Timeline::new(
        Keyframes::new([Keyframe::new(0.0, 0.0_f32), Keyframe::new(1.0, 10.0)]).unwrap(),
        Timing::new(Duration::from_millis(100)).delay(Duration::from_millis(100)),
    )
    .unwrap();
    let motion = Motion::new(7.0);
    motion.play(animation);
    assert!(!motion.advance_spring(Time::ZERO));
    assert!(!motion.advance(Time::ZERO).unwrap());
    assert!(!motion.advance(Time::from_nanos(50_000_000)).unwrap());
    assert_eq!(motion.value(), 7.0);
    assert!(motion.advance(Time::from_nanos(150_000_000)).unwrap());
    assert_eq!(motion.value(), 5.0);
}

#[test]
fn spring_reaches_a_terminal_state_without_an_explicit_finish() {
    let motion = Motion::new(0.0_f32);
    motion.spring_to(10.0, SpringConfig::default()).unwrap();
    motion.pause();
    motion.resume();
    assert!(!motion.advance(Time::ZERO).unwrap());
    assert_eq!(motion.value(), 0.0);
    for index in 1..1000 {
        motion.advance_spring(Time::from_nanos(index * 16_000_000));
        if !motion.is_active() {
            break;
        }
    }
    assert_eq!(motion.state(), MotionState::Finished);
    assert_eq!(motion.value(), 10.0);
    assert!(!motion.advance_spring(Time::from_nanos(20_000_000_000)));
}

#[test]
fn identity_debug_and_idle_controls_are_stable() {
    let motion = Motion::new(3.0_f32);
    let clone = motion.clone();
    let other = Motion::new(3.0_f32);

    motion.pause();
    motion.resume();
    assert_eq!(motion.state(), MotionState::Idle);
    assert_eq!(motion, clone);
    assert_ne!(motion, other);
    assert_eq!(motion.identity(), clone.identity());
    assert!(format!("{motion:?}").contains("Motion"));
    assert_eq!(motion.velocity(), 0.0);
    assert!(!motion.advance_spring(Time::ZERO));
}

#[test]
fn timeline_pause_resume_and_restart_preserve_presented_values() {
    let motion = Motion::new(0.0_f32);
    motion.restart(2.0, 12.0, Tween::new(Duration::from_millis(100)));
    MotionTrack::advance(&motion, Time::from_nanos(1));
    MotionTrack::advance(&motion, Time::from_nanos(40_000_001));
    let paused = motion.value();
    motion.pause();
    assert_eq!(motion.state(), MotionState::Paused);
    assert!(!MotionTrack::advance(&motion, Time::from_nanos(80_000_001)));
    assert_eq!(motion.value(), paused);

    motion.resume();
    assert!(!MotionTrack::advance(&motion, Time::from_nanos(90_000_001)));
    assert!(MotionTrack::advance(&motion, Time::from_nanos(150_000_001)));
    assert!(motion.value() > paused);
}

#[test]
fn settled_and_restarted_springs_have_explicit_states() {
    let motion = Motion::new(5.0_f32);
    motion.spring_to(5.0, SpringConfig::default()).unwrap();
    assert_eq!(motion.state(), MotionState::Finished);
    assert!(!motion.advance_spring(Time::ZERO));

    motion
        .restart_spring(0.0, 5.0, SpringConfig::default())
        .unwrap();
    assert_eq!(motion.value(), 0.0);
    assert_eq!(motion.target(), 5.0);
    assert_eq!(motion.state(), MotionState::Running);
}
