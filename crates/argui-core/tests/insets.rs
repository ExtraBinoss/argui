use argui_core::{Insets, Point, Rect, Size};

#[test]
fn insets_store_each_edge_and_sum_the_axes() {
    let insets = Insets::new(11.0, 13.0, 17.0, 19.0);

    assert_eq!(insets.top, 11.0);
    assert_eq!(insets.right, 13.0);
    assert_eq!(insets.bottom, 17.0);
    assert_eq!(insets.left, 19.0);
    assert_eq!(insets.horizontal(), 32.0);
    assert_eq!(insets.vertical(), 28.0);
}

#[test]
fn physical_rect_conversion_accounts_for_origin_and_scale() {
    let window = Rect::new(Point::new(100.0, 200.0), Size::new(400.0, 800.0));
    let safe = Rect::new(Point::new(120.0, 230.0), Size::new(340.0, 730.0));

    assert_eq!(
        Insets::from_physical_rects(window, safe, 2.0),
        Insets::new(15.0, 20.0, 20.0, 10.0)
    );
}

#[test]
fn physical_rect_conversion_clips_safe_rects_and_handles_invalid_scale() {
    let window = Rect::new(Point::default(), Size::new(400.0, 800.0));
    let outside = Rect::new(Point::new(-20.0, -30.0), Size::new(500.0, 900.0));
    assert_eq!(
        Insets::from_physical_rects(window, outside, f32::NAN),
        Insets::ZERO
    );

    let disjoint = Rect::new(Point::new(450.0, 900.0), Size::new(10.0, 10.0));
    assert_eq!(
        Insets::from_physical_rects(window, disjoint, 1.0),
        Insets::new(800.0, 0.0, 0.0, 400.0)
    );
}

#[test]
fn try_physical_rect_conversion_rejects_empty_or_disjoint_areas() {
    let window = Rect::new(Point::default(), Size::new(400.0, 800.0));
    let empty = Rect::new(Point::default(), Size::default());
    let outside = Rect::new(Point::new(500.0, 900.0), Size::new(20.0, 30.0));

    assert_eq!(Insets::try_from_physical_rects(window, empty, 1.0), None);
    assert_eq!(Insets::try_from_physical_rects(empty, window, 1.0), None);
    assert_eq!(Insets::try_from_physical_rects(window, outside, 1.0), None);
}

#[test]
fn zero_is_the_default_inset_value() {
    assert_eq!(Insets::default(), Insets::ZERO);
}
