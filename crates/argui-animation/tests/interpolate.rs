use argui_animation::Interpolate;
use argui_core::{Color, ColorInterpolation, Point, Rect, Size};

#[test]
fn scalar_interpolation_supports_endpoints_and_extrapolation() {
    assert_eq!(2.0_f32.interpolate(6.0, 0.0), 2.0);
    assert_eq!(2.0_f32.interpolate(6.0, 1.0), 6.0);
    assert_eq!(2.0_f32.interpolate(6.0, 1.5), 8.0);
    assert_eq!(2.0_f64.interpolate(6.0, 0.25), 3.0);
}

#[test]
fn core_visual_types_use_perceptual_color_interpolation() {
    let from = Color::srgba(0.0, 0.2, 0.4, 0.6);
    let target = Color::srgba(1.0, 0.6, 0.8, 1.0);
    let color = from.interpolate(target, 0.5);
    assert_eq!(color, from.mix(target, 0.5, ColorInterpolation::Oklab));
    assert!((color.to_linear_rgba()[3] - 0.8).abs() < f32::EPSILON * 2.0);
    assert_eq!(
        Point::new(0.0, 10.0).interpolate(Point::new(10.0, 30.0), 0.5),
        Point::new(5.0, 20.0)
    );
    assert_eq!(
        Size::new(10.0, 20.0).interpolate(Size::new(30.0, 60.0), 0.25),
        Size::new(15.0, 30.0)
    );
    assert_eq!(
        Rect::new(Point::new(0.0, 10.0), Size::new(20.0, 30.0)).interpolate(
            Rect::new(Point::new(10.0, 30.0), Size::new(40.0, 70.0)),
            0.5,
        ),
        Rect::new(Point::new(5.0, 20.0), Size::new(30.0, 50.0))
    );
}
