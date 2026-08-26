use argui_animation::{Easing, Keyframe, Keyframes, StepPosition, Steps, TimingError};

#[test]
fn keyframes_interpolate_each_segment_with_its_own_easing() {
    let frames = Keyframes::new(vec![
        Keyframe::new(0.0, 0.0_f32).easing(Easing::custom(|value| value * value)),
        Keyframe::new(0.5, 10.0)
            .easing(Easing::Steps(Steps::new(2, StepPosition::JumpEnd).unwrap())),
        Keyframe::new(1.0, 20.0),
    ])
    .unwrap();

    assert_eq!(frames.sample(-1.0), 0.0);
    assert_eq!(frames.sample(0.25), 2.5);
    assert_eq!(frames.sample(0.6), 10.0);
    assert_eq!(frames.sample(1.0), 20.0);
    assert_eq!(frames.as_slice().len(), 3);
}

#[test]
fn holds_and_duplicate_offsets_create_discontinuities() {
    let held = Keyframes::new(vec![
        Keyframe::new(0.0, 2.0_f32).hold(),
        Keyframe::new(0.5, 8.0),
        Keyframe::new(1.0, 10.0),
    ])
    .unwrap();
    assert_eq!(held.sample(0.49), 2.0);

    let duplicate = Keyframes::new(vec![
        Keyframe::new(0.0, 0.0_f32),
        Keyframe::new(0.5, 4.0),
        Keyframe::new(0.5, 7.0),
        Keyframe::new(1.0, 9.0),
    ])
    .unwrap();
    assert_eq!(duplicate.sample(0.5), 7.0);
}

#[test]
fn malformed_keyframe_sequences_are_rejected() {
    let invalid = [
        vec![Keyframe::new(0.0, 0.0_f32)],
        vec![Keyframe::new(0.1, 0.0), Keyframe::new(1.0, 1.0)],
        vec![Keyframe::new(0.0, 0.0), Keyframe::new(0.9, 1.0)],
        vec![
            Keyframe::new(0.0, 0.0),
            Keyframe::new(f32::NAN, 0.5),
            Keyframe::new(1.0, 1.0),
        ],
        vec![
            Keyframe::new(0.0, 0.0),
            Keyframe::new(0.8, 0.5),
            Keyframe::new(0.4, 0.7),
            Keyframe::new(1.0, 1.0),
        ],
    ];
    for frames in invalid {
        assert!(matches!(
            Keyframes::new(frames),
            Err(TimingError::InvalidKeyframes)
        ));
    }
}
