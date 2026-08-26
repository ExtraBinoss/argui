use argui_animation::{DecayConfig, Duration, Inertia, InertiaConfig, InertiaState, PhysicsError};

#[test]
fn bounded_inertia_preserves_velocity_into_its_bounce() {
    let config = InertiaConfig {
        bounds: Some((0.0, 100.0)),
        decay: DecayConfig {
            rate: 1.0,
            rest_speed: 0.001,
        },
        ..InertiaConfig::default()
    };
    let mut inertia = Inertia::new(90.0, 200.0, config).unwrap();
    assert_eq!(inertia.state(), InertiaState::Decaying);
    inertia.advance(Duration::from_millis(100));
    assert_eq!(inertia.state(), InertiaState::Bouncing);
    assert!(inertia.velocity() > 0.0);

    for _ in 0..1_000 {
        inertia.advance(Duration::from_millis(16));
        if !inertia.is_active() {
            break;
        }
    }
    assert_eq!(inertia.state(), InertiaState::Settled);
    assert_eq!(inertia.value(), 100.0);
    assert_eq!(inertia.velocity(), 0.0);
}

#[test]
fn out_of_bounds_values_bounce_immediately_and_launch_can_reuse_state() {
    let config = InertiaConfig {
        bounds: Some((0.0, 10.0)),
        ..InertiaConfig::default()
    };
    let mut inertia = Inertia::new(-2.0, 0.0, config).unwrap();
    assert_eq!(inertia.state(), InertiaState::Bouncing);
    inertia.launch(5.0, 20.0);
    assert_eq!(inertia.state(), InertiaState::Decaying);
    assert_eq!(inertia.value(), 5.0);
}

#[test]
fn unbounded_inertia_settles_as_plain_decay() {
    let mut inertia = Inertia::new(0.0, 4.0, InertiaConfig::default()).unwrap();
    for _ in 0..1_000 {
        inertia.advance(Duration::from_millis(16));
        if !inertia.is_active() {
            break;
        }
    }
    assert_eq!(inertia.state(), InertiaState::Settled);
    assert!(inertia.value() > 0.0);
}

#[test]
fn inertia_rejects_invalid_bounds() {
    assert_eq!(
        Inertia::new(
            0.0,
            1.0,
            InertiaConfig {
                bounds: Some((2.0, 1.0)),
                ..InertiaConfig::default()
            }
        ),
        Err(PhysicsError::InvalidBounds)
    );
}
