use argui_core::{Point, Rect, Size};
use argui_ui::{OverlayAlign, OverlayPlacement, PlacementSide};

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}

#[test]
fn placement_flips_and_clamps_to_the_viewport() {
    let viewport = rect(0.0, 0.0, 400.0, 300.0);
    let anchor = rect(160.0, 250.0, 80.0, 30.0);
    let placed = OverlayPlacement::new(PlacementSide::Bottom)
        .gap(10.0)
        .margin(12.0)
        .place(viewport, anchor, Size::new(220.0, 140.0));

    assert_eq!(placed.side, PlacementSide::Top);
    assert_eq!(placed.bounds, rect(90.0, 100.0, 220.0, 140.0));
    assert!(placed.max_size.height >= placed.bounds.size.height);
}

#[test]
fn oversized_overlays_report_the_scrollable_maximum() {
    let viewport = rect(20.0, 10.0, 300.0, 200.0);
    let anchor = rect(145.0, 85.0, 40.0, 30.0);
    let placed = OverlayPlacement::new(PlacementSide::Right)
        .align(OverlayAlign::End)
        .gap(5.0)
        .margin(10.0)
        .place(viewport, anchor, Size::new(800.0, 600.0));

    assert!(placed.bounds.size.width <= placed.max_size.width);
    assert!(placed.bounds.size.height <= placed.max_size.height);
    assert!(placed.bounds.origin.x >= 30.0);
    assert!(placed.bounds.origin.y >= 20.0);
    assert_eq!(placed.inset_from(viewport).left, argui_ui::length(170.0));
}

#[test]
fn transient_tiny_viewports_and_external_anchors_never_invert_clamp_bounds() {
    let placed = OverlayPlacement::new(PlacementSide::Bottom)
        .gap(12.0)
        .margin(14.0)
        .place(
            rect(0.0, 0.0, 0.0, 0.0),
            rect(20.0, 110.0, 80.0, 30.0),
            Size::new(400.0, 430.0),
        );

    assert_eq!(placed.bounds, rect(0.0, 0.0, 0.0, 0.0));
    assert_eq!(placed.max_size, Size::new(0.0, 0.0));
}

#[test]
fn subpixel_viewports_tolerate_a_one_ulp_inverted_interval() {
    let viewport = rect(13.641_06, 0.0, 428.0, 500.0);
    let placed = OverlayPlacement::new(PlacementSide::Bottom)
        .gap(12.0)
        .margin(14.0)
        .place(
            viewport,
            rect(120.0, 100.0, 80.0, 30.0),
            Size::new(400.0, 100.0),
        );

    assert!(placed.bounds.origin.x.is_finite());
    assert!(placed.bounds.origin.x >= viewport.origin.x + 14.0);
    assert!(
        placed.bounds.origin.x + placed.bounds.size.width
            <= viewport.origin.x + viewport.size.width - 14.0 + f32::EPSILON * 512.0
    );
}
