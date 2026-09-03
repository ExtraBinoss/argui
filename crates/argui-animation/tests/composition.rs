use argui_animation::{Composition, Contribution, compose};

#[test]
fn contributions_use_stable_priority_then_insertion_order() {
    let mut contributions = [
        Contribution {
            value: 3.0_f32,
            composition: Composition::Add,
            priority: 2,
            order: 1,
            completed_iterations: 0,
        },
        Contribution::replace(10.0, 1, 8),
        Contribution {
            value: 2.0,
            composition: Composition::Accumulate,
            priority: 2,
            order: 0,
            completed_iterations: 2,
        },
    ];
    assert_eq!(compose(1.0, &mut contributions), 19.0);
    assert_eq!(contributions[0].composition, Composition::Replace);
}

#[test]
fn geometry_and_color_support_additive_composition() {
    use argui_animation::Compose;
    use argui_core::{Color, Point, Rect, Size};

    assert_eq!(
        Point::new(1.0, 2.0).add(Point::new(3.0, 4.0)),
        Point::new(4.0, 6.0)
    );
    assert_eq!(Size::new(2.0, 3.0).scale(2.0), Size::new(4.0, 6.0));
    assert_eq!(
        Rect::new(Point::new(1.0, 2.0), Size::new(3.0, 4.0))
            .add(Rect::new(Point::new(2.0, 3.0), Size::new(4.0, 5.0))),
        Rect::new(Point::new(3.0, 5.0), Size::new(7.0, 9.0))
    );
    assert_eq!(
        Color::linear_rgba(0.1, 0.2, 0.3, 0.4).scale(2.0),
        Color::linear_rgba(0.2, 0.4, 0.6, 0.8)
    );
    assert_eq!(2.0_f64.scale(2.5), 5.0);
}
