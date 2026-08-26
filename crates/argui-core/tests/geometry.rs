use argui_core::{Point, Rect, Size};

#[test]
fn rect_contains_its_bounds_but_not_external_points() {
    let rect = Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 50.0));

    assert!(rect.contains(Point::new(10.0, 20.0)));
    assert!(rect.contains(Point::new(110.0, 70.0)));
    assert!(!rect.contains(Point::new(9.0, 30.0)));
    assert!(!rect.contains(Point::new(111.0, 30.0)));
    assert!(!rect.contains(Point::new(20.0, 19.0)));
    assert!(!rect.contains(Point::new(20.0, 71.0)));
}
