use argui_core::{Affine2D, Point, PointerId, Rect, Size};
use argui_paint::ClipChain;
use argui_ui::{Element, ScrollAxes, ScrollConfig, ScrollRegion, UiTree};

#[path = "../../src/app/touch_scroll.rs"]
mod implementation;

use implementation::TouchScrollGesture;

fn region(axes: ScrollAxes) -> ScrollRegion {
    let tree = UiTree::new(Element::container([]));
    ScrollRegion {
        node: tree.node_ids()[0],
        bounds: Rect::new(Point::default(), Size::new(200.0, 200.0)),
        clip: Rect::new(Point::default(), Size::new(200.0, 200.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        max_offset: Point::new(400.0, 400.0),
        config: ScrollConfig::default().axes(axes),
        scrollbar: None,
        interaction_order: 0,
    }
}

#[test]
fn stationary_touch_stays_a_tap_until_slop_is_exceeded() {
    let pointer = PointerId::new(1);
    let mut gesture = TouchScrollGesture::default();
    gesture.begin(pointer, Point::new(40.0, 40.0));
    assert!(
        gesture
            .moved(
                pointer,
                Point::new(45.0, 49.0),
                10.0,
                &[region(ScrollAxes::Vertical)]
            )
            .is_none()
    );
    assert!(gesture.end(pointer, false).is_none());
}

#[test]
fn touch_scroll_locks_to_intended_axis_and_removes_the_slop_jump() {
    let pointer = PointerId::new(2);
    let mut gesture = TouchScrollGesture::default();
    gesture.begin(pointer, Point::new(50.0, 50.0));
    let started = gesture
        .moved(
            pointer,
            Point::new(66.0, 54.0),
            10.0,
            &[region(ScrollAxes::Vertical), region(ScrollAxes::Horizontal)],
        )
        .unwrap();
    assert_eq!(started.phase, winit::event::TouchPhase::Started);
    assert_eq!(started.delta, Point::new(6.0, 0.0));

    let moved = gesture
        .moved(pointer, Point::new(70.0, 80.0), 10.0, &[])
        .unwrap();
    assert_eq!(moved.delta, Point::new(4.0, 0.0));
    assert_eq!(
        gesture.end(pointer, false).unwrap().phase,
        winit::event::TouchPhase::Ended
    );
}

#[test]
fn vertical_intent_does_not_hijack_a_horizontal_only_table_viewport() {
    let pointer = PointerId::new(3);
    let mut gesture = TouchScrollGesture::default();
    gesture.begin(pointer, Point::new(20.0, 20.0));
    assert!(
        gesture
            .moved(
                pointer,
                Point::new(22.0, 60.0),
                10.0,
                &[region(ScrollAxes::Horizontal)],
            )
            .is_none()
    );
    gesture.cancel();
    assert!(gesture.end(pointer, false).is_none());
}
