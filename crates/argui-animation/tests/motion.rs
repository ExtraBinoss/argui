use argui_animation::{MotionValue, PhysicsError};
use argui_core::{Color, Point, Rect, Size, Transform2D};

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

    let color = Color::linear_rgba(0.4, 0.3, 0.2, 0.1);
    assert_eq!(color.subtract(color), Color::TRANSPARENT);
    assert_eq!(color.add(color), Color::linear_rgba(0.8, 0.6, 0.4, 0.2));
    assert_eq!(Color::linear_rgba(1.0, 0.0, 0.0, 0.0).magnitude(), 1.0);
}

#[test]
fn transforms_are_spring_values_with_component_velocity() {
    let start = Transform2D::IDENTITY.translate(2.0, 3.0).rotate(0.2);
    let target = Transform2D::IDENTITY.scale(2.0, 0.5).skew(0.1, 0.0);
    let delta = start.subtract(target);
    assert_eq!(target.add(delta), start);
    assert!(delta.magnitude() > 1.0);
    assert_eq!(Transform2D::zero().scale, Point::default());
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
