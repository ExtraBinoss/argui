use std::time::Duration;

use argui_core::{Affine2D, Point, Rect, ScrollDelta, Size};
use argui_paint::ClipChain;
use argui_ui::{Element, NodeId, ScrollConfig, ScrollGesture, ScrollRegion, UiTree};

fn region(node: NodeId, y: f32) -> ScrollRegion {
    let bounds = Rect::new(Point::new(0.0, y), Size::new(200.0, 200.0));
    ScrollRegion {
        node,
        bounds,
        clip: bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        max_offset: Point::new(0.0, 500.0),
        config: ScrollConfig::default(),
        scrollbar: None,
        interaction_order: 0,
    }
}

#[test]
fn moving_a_list_under_the_pointer_does_not_steal_the_page_gesture() {
    let mut tree = UiTree::new(Element::column([Element::column([])]));
    let parent = tree.node_id_at(0).unwrap();
    let child = tree.node_id_at(1).unwrap();
    let mut regions = [region(parent, 0.0), region(child, 100.0)];
    let pointer = Point::new(10.0, 10.0);
    let mut gesture = ScrollGesture::default();
    let delta = ScrollDelta::Pixels(Point::new(0.0, -30.0));
    for tick in 0..10 {
        let target = gesture.target(pointer, Duration::from_millis(tick * 16), false, &regions);
        assert_eq!(target, Some(parent));
        tree.scroll_from(target.unwrap(), pointer, delta, &regions);
        regions[1] = region(child, 0.0);
    }
    assert_eq!(tree.scroll_offset(parent).y, 300.0);
    assert_eq!(tree.scroll_offset(child).y, 0.0);
    let target = gesture.target(pointer, Duration::from_millis(400), false, &regions);
    assert_eq!(target, Some(child));
    tree.scroll_from(target.unwrap(), pointer, delta, &regions);
    assert_eq!(tree.scroll_offset(child).y, 30.0);
    assert_eq!(tree.scroll_offset(parent).y, 300.0);
}

#[test]
fn latched_scroll_chains_only_to_ancestors_even_outside_moving_bounds() {
    let mut tree = UiTree::new(Element::column([Element::column([]), Element::column([])]));
    let parent = tree.node_id_at(0).unwrap();
    let child = tree.node_id_at(1).unwrap();
    let sibling = tree.node_id_at(2).unwrap();
    let regions = [
        region(parent, 0.0),
        region(child, 0.0),
        region(sibling, 0.0),
    ];
    tree.set_scroll_offset(child, Point::new(0.0, 490.0));
    tree.scroll_from(
        child,
        Point::new(900.0, 900.0),
        ScrollDelta::Pixels(Point::new(0.0, -30.0)),
        &regions,
    );
    assert_eq!(tree.scroll_offset(child).y, 500.0);
    assert_eq!(tree.scroll_offset(parent).y, 20.0);
    assert_eq!(tree.scroll_offset(sibling).y, 0.0);
    let mut disabled = regions.clone();
    disabled[1].config.enabled = false;
    for candidates in [&disabled[..], &regions[..1]] {
        assert!(
            !tree
                .scroll_from(
                    child,
                    Point::default(),
                    ScrollDelta::Lines(Point::new(0.0, -1.0)),
                    candidates
                )
                .scroll_changed
        );
        assert_eq!(tree.scroll_offset(parent).y, 20.0);
    }
}

#[test]
fn explicit_start_timeout_and_removed_targets_are_safe() {
    let tree = UiTree::new(Element::column([Element::column([])]));
    let parent = tree.node_id_at(0).unwrap();
    let child = tree.node_id_at(1).unwrap();
    let mut regions = [region(parent, 0.0), region(child, 100.0)];
    let pointer = Point::new(10.0, 10.0);
    let mut gesture = ScrollGesture::default();
    let mut sample = |ms, start, regions: &[ScrollRegion]| {
        gesture.target(pointer, Duration::from_millis(ms), start, regions)
    };
    assert_eq!(sample(0, false, &regions), Some(parent));
    regions[1] = region(child, 0.0);
    assert_eq!(sample(1, true, &regions), Some(child));
    regions[1].config.enabled = false;
    assert_eq!(sample(2, false, &regions), None);
    assert_eq!(sample(3, false, &regions), None);
    assert_eq!(sample(183, false, &regions), Some(parent));
    assert_eq!(sample(184, false, &[]), None);
    assert_eq!(sample(185, false, &regions), None);
    assert_eq!(sample(0, false, &regions), Some(parent));
    assert_eq!(sample(500, false, &[]), None);
}
