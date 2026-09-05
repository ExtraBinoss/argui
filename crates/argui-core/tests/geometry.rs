use argui_core::{Affine2D, Point, Rect, Size, Transform2D, TransformOrigin};

#[test]
fn rect_contains_its_bounds_but_not_external_points() {
    let rect = Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 50.0));

    assert!(rect.contains(Point::new(10.0, 20.0)));
    assert!(rect.contains(Point::new(110.0, 70.0)));
    assert!(!rect.contains(Point::new(9.0, 30.0)));
    assert!(!rect.contains(Point::new(111.0, 30.0)));
    assert!(!rect.contains(Point::new(20.0, 19.0)));
    assert!(!rect.contains(Point::new(20.0, 71.0)));

    let overlap = rect
        .intersection(Rect::new(Point::new(100.0, 60.0), Size::new(30.0, 30.0)))
        .unwrap();
    assert_eq!(
        overlap,
        Rect::new(Point::new(100.0, 60.0), Size::new(10.0, 10.0))
    );
    assert!(
        rect.intersection(Rect::new(Point::new(200.0, 200.0), Size::new(10.0, 10.0)))
            .is_none()
    );
}

#[test]
fn affine_transforms_round_trip_points_and_reject_singular_matrices() {
    let translated = Affine2D::translation(4.0, -3.0);
    let point = Point::new(2.0, 5.0);
    let transformed = translated.transform_point(point);

    assert_eq!(transformed, Point::new(6.0, 2.0));
    assert_eq!(
        translated.inverse().unwrap().transform_point(transformed),
        point
    );
    assert!(
        Affine2D {
            matrix: [1.0, 2.0, 2.0, 4.0],
            translation: Point::default(),
        }
        .inverse()
        .is_none()
    );

    let bounds = Rect::new(Point::new(2.0, 4.0), Size::new(10.0, 6.0));
    assert_eq!(bounds.corners()[2], Point::new(12.0, 10.0));
    assert_eq!(
        Transform2D::IDENTITY
            .translate(3.0, 5.0)
            .affine(bounds, TransformOrigin::TOP_LEFT)
            .transform_point(Point::new(2.0, 4.0)),
        Point::new(5.0, 9.0)
    );
}
