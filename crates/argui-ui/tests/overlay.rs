use argui_core::{Point, Rect, Size};
use argui_ui::{
    AnchorWidth, CollisionPolicy, FloatingPlacement, Placement, ViewportAlign, ViewportPlacement,
    WritingDirection,
};

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}

#[test]
fn placement_flips_and_shifts_inside_the_viewport() {
    let viewport = rect(0.0, 0.0, 400.0, 300.0);
    let anchor = rect(160.0, 250.0, 80.0, 30.0);
    let placed = FloatingPlacement::new(Placement::Bottom)
        .offset(10.0)
        .viewport_padding(12.0)
        .place(
            viewport,
            anchor,
            Size::new(220.0, 140.0),
            WritingDirection::Ltr,
        );

    assert_eq!(placed.placement, Placement::Top);
    assert_eq!(placed.bounds, rect(90.0, 100.0, 220.0, 140.0));
    assert!(placed.available_size.height >= placed.bounds.size.height);
}

#[test]
fn oversized_overlays_report_constraints_and_anchor_width() {
    let viewport = rect(20.0, 10.0, 300.0, 200.0);
    let anchor = rect(145.0, 85.0, 40.0, 30.0);
    let placed = FloatingPlacement::new(Placement::RightEnd)
        .anchor_width(AnchorWidth::AtLeastAnchor)
        .offset(5.0)
        .viewport_padding(10.0)
        .place(
            viewport,
            anchor,
            Size::new(800.0, 600.0),
            WritingDirection::Ltr,
        );

    assert!(placed.constrained_width);
    assert!(placed.constrained_height);
    assert!(placed.bounds.origin.x >= 30.0);
    assert!(placed.bounds.origin.y >= 20.0);
    assert_eq!(placed.inset_from(viewport).left, argui_ui::length(170.0));
}

#[test]
fn start_and_end_follow_writing_direction() {
    let placement = FloatingPlacement::new(Placement::BottomStart).collision(CollisionPolicy::NONE);
    let viewport = rect(0.0, 0.0, 500.0, 500.0);
    let anchor = rect(200.0, 100.0, 100.0, 20.0);
    let ltr = placement.place(
        viewport,
        anchor,
        Size::new(40.0, 30.0),
        WritingDirection::Ltr,
    );
    let rtl = placement.place(
        viewport,
        anchor,
        Size::new(40.0, 30.0),
        WritingDirection::Rtl,
    );

    assert_eq!(ltr.bounds.origin.x, 200.0);
    assert_eq!(rtl.bounds.origin.x, 260.0);
}

#[test]
fn all_placements_produce_finite_bounds() {
    let placements = [
        Placement::TopStart,
        Placement::Top,
        Placement::TopEnd,
        Placement::BottomStart,
        Placement::Bottom,
        Placement::BottomEnd,
        Placement::LeftStart,
        Placement::Left,
        Placement::LeftEnd,
        Placement::RightStart,
        Placement::Right,
        Placement::RightEnd,
    ];
    for placement in placements {
        let placed = FloatingPlacement::new(placement).place(
            rect(0.0, 0.0, 600.0, 400.0),
            rect(250.0, 180.0, 100.0, 40.0),
            Size::new(80.0, 60.0),
            WritingDirection::Ltr,
        );
        assert!(placed.bounds.origin.x.is_finite());
        assert!(placed.bounds.origin.y.is_finite());
        assert!(placed.bounds.size.width > 0.0);
        assert!(placed.bounds.size.height > 0.0);
    }
}

#[test]
fn tiny_and_non_finite_geometry_is_normalized() {
    let placed = FloatingPlacement::default().place(
        rect(f32::NAN, 0.0, -10.0, f32::INFINITY),
        rect(20.0, 110.0, 80.0, 30.0),
        Size::new(400.0, 430.0),
        WritingDirection::Ltr,
    );

    assert_eq!(placed.bounds, rect(0.0, 0.0, 0.0, 0.0));
    assert_eq!(placed.available_size, Size::new(0.0, 0.0));
}

#[test]
fn viewport_placement_fills_or_aligns_within_margin() {
    let viewport = rect(10.0, 20.0, 300.0, 200.0);
    assert_eq!(
        ViewportPlacement::fill()
            .margin(5.0)
            .place(viewport, Size::default()),
        rect(15.0, 25.0, 290.0, 190.0)
    );
    assert_eq!(
        ViewportPlacement::centered()
            .align(ViewportAlign::End, ViewportAlign::Start)
            .margin(10.0)
            .place(viewport, Size::new(80.0, 40.0)),
        rect(220.0, 30.0, 80.0, 40.0)
    );
}

#[test]
fn collision_policy_can_preserve_overflow_and_cross_offset() {
    let placed = FloatingPlacement::new(Placement::BottomEnd)
        .offset(4.0)
        .cross_offset(3.0)
        .viewport_padding(0.0)
        .collision(CollisionPolicy::NONE)
        .place(
            rect(0.0, 0.0, 100.0, 80.0),
            rect(70.0, 60.0, 20.0, 10.0),
            Size::new(120.0, 50.0),
            WritingDirection::Ltr,
        );

    assert_eq!(placed.placement, Placement::BottomEnd);
    assert_eq!(placed.bounds, rect(-27.0, 74.0, 120.0, 50.0));
    assert!(!placed.constrained_width);
    assert!(!placed.constrained_height);
}

#[test]
fn anchor_width_supports_minimum_and_exact_matching() {
    let viewport = rect(0.0, 0.0, 500.0, 300.0);
    let anchor = rect(100.0, 80.0, 160.0, 30.0);
    let minimum = FloatingPlacement::default()
        .anchor_width(AnchorWidth::AtLeastAnchor)
        .place(
            viewport,
            anchor,
            Size::new(80.0, 40.0),
            WritingDirection::Ltr,
        );
    let exact = FloatingPlacement::default()
        .anchor_width(AnchorWidth::MatchAnchor)
        .place(
            viewport,
            anchor,
            Size::new(240.0, 40.0),
            WritingDirection::Ltr,
        );

    assert_eq!(minimum.bounds.size.width, 160.0);
    assert_eq!(exact.bounds.size.width, 160.0);
}

#[test]
fn flip_uses_a_perpendicular_side_when_both_vertical_sides_are_too_short() {
    let placed = FloatingPlacement::new(Placement::BottomStart)
        .viewport_padding(0.0)
        .place(
            rect(0.0, 0.0, 500.0, 300.0),
            rect(200.0, 130.0, 40.0, 40.0),
            Size::new(100.0, 280.0),
            WritingDirection::Ltr,
        );

    assert_eq!(placed.placement, Placement::RightStart);
}

#[test]
fn viewport_start_center_and_end_are_exact() {
    let viewport = rect(10.0, 20.0, 300.0, 200.0);
    let desired = Size::new(80.0, 40.0);
    let start = ViewportPlacement::centered()
        .align(ViewportAlign::Start, ViewportAlign::End)
        .margin(10.0)
        .place(viewport, desired);
    let center = ViewportPlacement::centered()
        .margin(10.0)
        .place(viewport, desired);

    assert_eq!(start, rect(20.0, 170.0, 80.0, 40.0));
    assert_eq!(center, rect(120.0, 100.0, 80.0, 40.0));
}

#[path = "overlay/portal.rs"]
mod portal;
