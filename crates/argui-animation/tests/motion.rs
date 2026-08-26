use argui_animation::{MotionValue, PhysicsError};
use argui_core::{Color, Point, Rect, Size};

#[test]
fn motion_values_provide_vector_arithmetic_and_magnitude() {
    assert_eq!(f32::zero(), 0.0);
    assert_eq!(3.0_f32.subtract(1.0), 2.0);
    assert_eq!(2.0_f64.add(3.0), 5.0);
    assert_eq!(2.0_f64.scale(1.5), 3.0);
    assert_eq!(Point::new(3.0, 4.0).magnitude(), 5.0);
    assert_eq!(Size::new(3.0, 4.0).magnitude(), 5.0);

    let rect = Rect::new(Point::new(1.0, 2.0), Size::new(3.0, 4.0));
    assert_eq!(rect.subtract(rect), Rect::default());
    assert_eq!(rect.scale(2.0).origin, Point::new(2.0, 4.0));

    let color = Color::rgba(0.4, 0.3, 0.2, 0.1);
    assert_eq!(color.subtract(color), Color::TRANSPARENT);
    assert_eq!(color.add(color), Color::rgba(0.8, 0.6, 0.4, 0.2));
    assert_eq!(Color::rgba(1.0, 0.0, 0.0, 0.0).magnitude(), 1.0);
}

#[test]
fn every_physics_error_has_an_actionable_message() {
    for error in [
        PhysicsError::InvalidMass,
        PhysicsError::InvalidStiffness,
        PhysicsError::InvalidDamping,
        PhysicsError::InvalidRestThreshold,
        PhysicsError::InvalidDecay,
        PhysicsError::InvalidBounds,
    ] {
        assert!(!error.to_string().is_empty());
    }
}
