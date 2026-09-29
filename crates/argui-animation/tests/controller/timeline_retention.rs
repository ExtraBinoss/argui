use argui_animation::{Duration, Keyframe, Keyframes, Motion, MotionState, Time, Timeline, Timing};

fn entry(target: f32) -> Motion<f32> {
    let motion = Motion::new(0.0);
    motion.play(
        Timeline::new(
            Keyframes::new([Keyframe::new(0.0, 0.0), Keyframe::new(1.0, target)]).unwrap(),
            Timing::new(Duration::from_millis(150)),
        )
        .unwrap(),
    );
    motion
}

#[test]
fn completed_entry_is_retained_without_restarting_or_scheduling_frames() {
    let motion = entry(1.0);
    motion.advance(Time::from_nanos(1)).unwrap();
    motion.advance(Time::from_nanos(150_000_001)).unwrap();
    assert_eq!(motion.state(), MotionState::Finished);
    assert!(motion.retain_timeline(&entry(1.0)));
    assert!(!motion.is_active());
    assert_eq!(motion.value(), 1.0);
    assert!(!motion.retain_timeline(&entry(0.5)));
}

#[test]
fn reduced_motion_finished_entry_and_active_phase_survive_equivalent_updates() {
    let motion = entry(1.0);
    motion.advance(Time::from_nanos(1)).unwrap();
    motion.advance(Time::from_nanos(75_000_001)).unwrap();
    assert!(motion.retain_timeline(&entry(1.0)));
    assert_eq!(motion.value(), 0.5);
    motion.finish();
    assert!(motion.retain_timeline(&entry(1.0)));
    assert_eq!(motion.value(), 1.0);
    assert!(!motion.is_active());
}

#[test]
fn canceled_entry_and_explicit_value_reset_are_not_retained() {
    let motion = entry(1.0);
    motion.cancel();
    assert!(!motion.retain_timeline(&entry(1.0)));
    motion.set(0.7);
    assert!(!motion.retain_timeline(&entry(1.0)));
    let remounted = entry(1.0);
    assert_eq!(remounted.value(), 0.0);
    assert!(remounted.is_active());
}
