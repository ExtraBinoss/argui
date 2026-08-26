use argui_animation::{Duration, PhysicsError, Spring, SpringConfig};

fn close(actual: f32, expected: f32, tolerance: f32) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

#[test]
fn analytical_spring_is_independent_from_frame_partitioning() {
    let config = SpringConfig {
        damping: 12.0,
        ..SpringConfig::default()
    };
    let mut single = Spring::new(0.0_f32, 1.0, 0.0, config).unwrap();
    let mut partitioned = single;
    single.advance(Duration::from_secs(1));
    for _ in 0..100 {
        partitioned.advance(Duration::from_millis(10));
    }
    close(single.value(), partitioned.value(), 1.0e-5);
    close(single.velocity(), partitioned.velocity(), 1.0e-4);
}

#[test]
fn critical_and_overdamped_regimes_are_stable_after_long_frames() {
    let stiffness: f64 = 100.0;
    let critical = SpringConfig {
        stiffness,
        damping: 2.0 * stiffness.sqrt(),
        rest_speed: 0.0,
        rest_delta: 0.0,
        ..SpringConfig::default()
    };
    let mut spring = Spring::new(0.0_f64, 1.0, 0.0, critical).unwrap();
    spring.advance(Duration::from_millis(250));
    assert!(spring.value().is_finite());
    assert!(spring.value() > 0.0 && spring.value() < 1.0);

    let mut overdamped = Spring::new(
        0.0_f64,
        1.0,
        0.0,
        SpringConfig {
            damping: 40.0,
            ..critical
        },
    )
    .unwrap();
    overdamped.advance(Duration::from_secs(10));
    assert!(overdamped.value().is_finite());
}

#[test]
fn retargeting_preserves_velocity_and_settles_exactly() {
    let mut spring = Spring::new(0.0_f32, 1.0, 3.0, SpringConfig::default()).unwrap();
    spring.advance(Duration::from_millis(30));
    let velocity = spring.velocity();
    spring.retarget(-1.0);
    assert_eq!(spring.velocity(), velocity);
    assert_eq!(spring.target(), -1.0);
    spring.set_velocity(2.0);
    assert_eq!(spring.velocity(), 2.0);

    for _ in 0..1_000 {
        spring.advance(Duration::from_millis(16));
        if !spring.is_active() {
            break;
        }
    }
    assert!(!spring.is_active());
    assert_eq!(spring.value(), -1.0);
    assert_eq!(spring.velocity(), 0.0);
    assert!(!spring.advance(Duration::from_millis(16)));
}

#[test]
fn spring_configuration_rejects_non_physical_values() {
    for (config, expected) in [
        (
            SpringConfig {
                mass: 0.0,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidMass,
        ),
        (
            SpringConfig {
                mass: f64::NAN,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidMass,
        ),
        (
            SpringConfig {
                stiffness: f64::NAN,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidStiffness,
        ),
        (
            SpringConfig {
                stiffness: 0.0,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidStiffness,
        ),
        (
            SpringConfig {
                damping: -1.0,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidDamping,
        ),
        (
            SpringConfig {
                damping: f64::NAN,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidDamping,
        ),
        (
            SpringConfig {
                rest_speed: -1.0,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidRestThreshold,
        ),
        (
            SpringConfig {
                rest_speed: f64::NAN,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidRestThreshold,
        ),
        (
            SpringConfig {
                rest_delta: -1.0,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidRestThreshold,
        ),
        (
            SpringConfig {
                rest_delta: f64::NAN,
                ..SpringConfig::default()
            },
            PhysicsError::InvalidRestThreshold,
        ),
    ] {
        assert_eq!(Spring::new(0.0_f32, 1.0, 0.0, config), Err(expected));
    }
    let settled = Spring::new(1.0_f32, 1.0, 0.0, SpringConfig::default()).unwrap();
    assert!(!settled.is_active());

    let mut velocity_only = Spring::new(1.0_f32, 1.0, 2.0, SpringConfig::default()).unwrap();
    assert!(velocity_only.is_active());
    assert!(!velocity_only.advance(Duration::ZERO));
    velocity_only.retarget(1.0);
    velocity_only.set_velocity(0.0);
    assert!(!velocity_only.is_active());
}
