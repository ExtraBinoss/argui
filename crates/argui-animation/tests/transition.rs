use argui_animation::{Duration, Motion, SpringConfig, Time, Transition, Tween};

#[test]
fn tween_transition_retargets_the_existing_motion() {
    let motion = Motion::new(0.0_f32);
    Transition::tween(Tween::new(Duration::from_millis(100))).retarget(&motion, 1.0);
    motion.advance(Time::from_nanos(1)).unwrap();
    motion.advance(Time::from_nanos(50_000_001)).unwrap();
    assert!((motion.value() - 0.5).abs() < 0.001);
}

#[test]
fn spring_transition_validates_before_it_can_be_stored() {
    let invalid = SpringConfig {
        mass: 0.0,
        ..SpringConfig::default()
    };
    assert!(Transition::try_spring(invalid).is_err());

    let motion = Motion::new(0.0_f32);
    Transition::spring().retarget(&motion, 1.0);
    motion.advance_spring(Time::from_nanos(1));
    assert!(motion.is_active());
}
