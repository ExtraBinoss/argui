use argui_animation::{CubicBezier, Easing, EasingError, LinearStop, StepPosition, Steps};

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1.0e-4, "{actual} != {expected}");
}

#[test]
fn linear_custom_and_bezier_easing_sample_bounded_input() {
    close(Easing::Linear.sample(-1.0), 0.0);
    close(Easing::Linear.sample(2.0), 1.0);
    close(Easing::custom(|value| value * value).sample(0.5), 0.25);

    let linear_bezier = CubicBezier::new(0.0, 0.0, 1.0, 1.0).unwrap();
    close(linear_bezier.sample(0.35), 0.35);
    close(Easing::CubicBezier(linear_bezier).sample(0.7), 0.7);
    assert_eq!(format!("{:?}", Easing::custom(|value| value)), "Custom(..)");
}

#[test]
fn invalid_bezier_coordinates_are_rejected() {
    assert_eq!(
        CubicBezier::new(-0.1, 0.0, 1.0, 1.0),
        Err(EasingError::InvalidBezier)
    );
    assert_eq!(
        CubicBezier::new(0.0, f32::NAN, 1.0, 1.0),
        Err(EasingError::InvalidBezier)
    );
    assert!(EasingError::InvalidBezier.to_string().contains("Bezier"));
}

#[test]
fn every_step_position_has_explicit_boundary_behavior() {
    close(
        Steps::new(4, StepPosition::JumpStart).unwrap().sample(0.0),
        0.25,
    );
    close(
        Steps::new(4, StepPosition::JumpEnd).unwrap().sample(0.1),
        0.0,
    );
    close(
        Steps::new(4, StepPosition::JumpNone).unwrap().sample(0.5),
        2.0 / 3.0,
    );
    close(
        Steps::new(4, StepPosition::JumpBoth).unwrap().sample(0.5),
        3.0 / 5.0,
    );
    assert_eq!(
        Steps::new(0, StepPosition::JumpEnd),
        Err(EasingError::InvalidSteps)
    );
    assert_eq!(
        Steps::new(1, StepPosition::JumpNone),
        Err(EasingError::InvalidSteps)
    );
}

#[test]
fn piecewise_linear_easing_interpolates_and_supports_hard_stops() {
    let easing = Easing::piecewise_linear(vec![
        LinearStop::new(0.0, 0.0),
        LinearStop::new(0.5, 0.25),
        LinearStop::new(0.5, 0.75),
        LinearStop::new(1.0, 1.0),
    ])
    .unwrap();
    close(easing.sample(0.25), 0.125);
    close(easing.sample(0.5), 0.75);
    close(easing.sample(1.0), 1.0);

    for invalid in [
        vec![LinearStop::new(0.0, 0.0)],
        vec![LinearStop::new(0.1, 0.0), LinearStop::new(1.0, 1.0)],
        vec![LinearStop::new(0.0, 0.0), LinearStop::new(0.9, f32::NAN)],
        vec![
            LinearStop::new(0.0, 0.0),
            LinearStop::new(0.8, 1.0),
            LinearStop::new(0.7, 1.0),
            LinearStop::new(1.0, 1.0),
        ],
    ] {
        assert!(matches!(
            Easing::piecewise_linear(invalid),
            Err(EasingError::InvalidStops)
        ));
    }
}
