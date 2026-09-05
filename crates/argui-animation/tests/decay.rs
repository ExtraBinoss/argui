use argui_animation::{Decay, DecayConfig, Duration, PhysicsError};

#[test]
fn exponential_decay_is_frame_partition_independent() {
    let config = DecayConfig {
        rest_speed: 0.0,
        ..DecayConfig::default()
    };
    let mut single = Decay::new(0.0_f64, 100.0, config).unwrap();
    let mut partitioned = single;
    single.advance(Duration::from_secs(1));
    for _ in 0..100 {
        partitioned.advance(Duration::from_millis(10));
    }
    assert!((single.value() - partitioned.value()).abs() < 1.0e-10);
    assert!((single.velocity() - partitioned.velocity()).abs() < 1.0e-10);
}

#[test]
fn decay_kicks_and_stops_at_its_rest_threshold() {
    let mut decay = Decay::new(4.0_f32, 0.0, DecayConfig::default()).unwrap();
    assert!(!decay.is_active());
    assert!(!decay.advance(Duration::ZERO));
    decay.kick(10.0);
    assert!(decay.is_active());
    assert!(!decay.advance(Duration::ZERO));
    assert_eq!(decay.value(), 4.0);
    assert_eq!(decay.velocity(), 10.0);
    for _ in 0..1_000 {
        decay.advance(Duration::from_millis(16));
        if !decay.is_active() {
            break;
        }
    }
    assert!(!decay.is_active());
    assert_eq!(decay.velocity(), 0.0);
    assert!(decay.value() > 4.0);
}

#[test]
fn decay_configuration_is_validated() {
    assert_eq!(
        Decay::new(
            0.0_f32,
            1.0,
            DecayConfig {
                rate: f64::NAN,
                ..DecayConfig::default()
            }
        ),
        Err(PhysicsError::InvalidDecay)
    );
    assert_eq!(
        Decay::new(
            0.0_f32,
            1.0,
            DecayConfig {
                rate: 0.0,
                ..DecayConfig::default()
            }
        ),
        Err(PhysicsError::InvalidDecay)
    );
    assert_eq!(
        Decay::new(
            0.0_f32,
            1.0,
            DecayConfig {
                rest_speed: f64::NAN,
                ..DecayConfig::default()
            }
        ),
        Err(PhysicsError::InvalidRestThreshold)
    );
    assert_eq!(
        Decay::new(
            0.0_f32,
            1.0,
            DecayConfig {
                rest_speed: -1.0,
                ..DecayConfig::default()
            }
        ),
        Err(PhysicsError::InvalidRestThreshold)
    );
}
