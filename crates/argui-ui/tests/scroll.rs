use argui_animation::{Duration, Motion, MotionState, Time, Tween};
use argui_core::{Affine2D, Point, Rect, ScrollDelta, Size};
use argui_paint::{ClipChain, ClipRegion};
use argui_ui::{
    Color, Element, QuadStyle, ScrollChaining, ScrollConfig, ScrollPolarity, ScrollRegion,
    ScrollbarRegion, ScrollbarStyle, TreeUpdate, UiEventKind, UiTree, property,
};

fn region(node: argui_ui::NodeId, config: ScrollConfig, max_y: f32) -> ScrollRegion {
    let bounds = Rect::new(Point::default(), Size::new(200.0, 200.0));
    ScrollRegion {
        node,
        bounds,
        clip: bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        max_offset: Point::new(0.0, max_y),
        config,
        scrollbar: None,
    }
}

#[test]
fn scroll_polarity_is_explicit_and_offsets_are_clamped() {
    let mut tree = UiTree::new(
        Element::container([])
            .keyed("scroll")
            .scrollable(ScrollConfig::default()),
    );
    let node = tree.node_id_at(0).unwrap();
    let normal = [region(node, ScrollConfig::default(), 100.0)];

    let update = tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Lines(Point::new(0.0, -1.0)),
        &normal,
    );
    assert!(update.scroll_changed);
    assert!(matches!(
        update.events[0].kind,
        UiEventKind::Scrolled { offset, .. } if offset == Point::new(0.0, 40.0)
    ));

    let inverted = [region(
        node,
        ScrollConfig::default().polarity(ScrollPolarity::Inverted),
        100.0,
    )];
    tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -200.0)),
        &inverted,
    );
    assert_eq!(tree.scroll_offset(node), Point::new(0.0, 0.0));
}

#[test]
fn exhausted_nested_scrolls_chain_to_their_parent() {
    let mut tree = UiTree::new(Element::column([
        Element::container([]).keyed("parent"),
        Element::container([]).keyed("child"),
    ]));
    let parent = tree.node_id_at(1).unwrap();
    let child = tree.node_id_at(2).unwrap();
    let regions = [
        region(parent, ScrollConfig::default(), 100.0),
        region(child, ScrollConfig::default(), 0.0),
    ];

    let update = tree.scroll(
        Point::new(20.0, 20.0),
        ScrollDelta::Lines(Point::new(0.0, -1.0)),
        &regions,
    );
    assert_eq!(update.events[0].target, parent);
}

#[test]
fn contained_nested_scroll_consumes_the_gesture_at_its_edge() {
    let mut tree = UiTree::new(Element::column([
        Element::container([]).keyed("parent"),
        Element::container([]).keyed("child"),
    ]));
    let parent = tree.node_id_at(1).unwrap();
    let child = tree.node_id_at(2).unwrap();
    let regions = [
        region(parent, ScrollConfig::default(), 100.0),
        region(
            child,
            ScrollConfig::default().chaining(ScrollChaining::Contain),
            0.0,
        ),
    ];

    let update = tree.scroll(
        Point::new(20.0, 20.0),
        ScrollDelta::Lines(Point::new(0.0, -1.0)),
        &regions,
    );
    assert!(update.events.is_empty());
    assert_eq!(tree.scroll_offset(parent), Point::default());
}

#[test]
fn disabled_scroll_regions_do_not_capture_input() {
    let mut tree = UiTree::new(Element::container([]).keyed("scroll"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node, ScrollConfig::default().enabled(false), 100.0)];
    let update = tree.scroll(
        Point::new(20.0, 20.0),
        ScrollDelta::Lines(Point::new(0.0, -1.0)),
        &regions,
    );
    assert!(!update.scroll_changed);
    assert_eq!(tree.scroll_offset(node), Point::default());
}

#[test]
fn scrollbar_track_and_thumb_drive_the_retained_offset() {
    let mut tree = UiTree::new(Element::container([]).keyed("scroll"));
    let node = tree.node_id_at(0).unwrap();
    let style = ScrollbarStyle::new(
        QuadStyle::solid(Color::rgb(0.0, 0.0, 0.0)),
        QuadStyle::solid(Color::WHITE),
    );
    let mut scroll = region(
        node,
        ScrollConfig::default().scrollbar(style.clone()),
        1_000.0,
    );
    scroll.scrollbar = Some(ScrollbarRegion {
        track: scroll.bounds,
        thumb: Rect::new(Point::default(), Size::new(200.0, 40.0)),
        style: style.clone(),
    });
    let regions = [scroll.clone()];

    assert!(regions[0].scrollbar_contains(Point::new(100.0, 20.0)));
    let pressed = tree
        .scrollbar_pressed(Point::new(100.0, 20.0), &regions)
        .unwrap();
    assert!(!pressed.scroll_changed);
    let dragged = tree
        .scrollbar_dragged(Point::new(100.0, 180.0), &regions)
        .unwrap();
    assert!(dragged.scroll_changed);
    assert_eq!(tree.scroll_offset(node), Point::new(0.0, 1_000.0));
    assert!(tree.scrollbar_released());
    assert!(!tree.scrollbar_released());
}

#[test]
fn scrollbar_ignores_track_outside_the_effective_clip() {
    let mut tree = UiTree::new(Element::container([]));
    let node = tree.node_id_at(0).unwrap();
    let style = ScrollbarStyle::new(QuadStyle::default(), QuadStyle::default());
    let mut scroll = region(node, ScrollConfig::default(), 100.0);
    assert!(!scroll.scrollbar_contains(Point::new(10.0, 10.0)));
    scroll.clip = Rect::new(Point::default(), Size::new(50.0, 50.0));
    scroll.scrollbar = Some(ScrollbarRegion {
        track: scroll.bounds,
        thumb: scroll.bounds,
        style: style.clone(),
    });

    assert!(!scroll.scrollbar_contains(Point::new(100.0, 100.0)));
    assert!(
        tree.scrollbar_pressed(Point::new(100.0, 100.0), &[scroll.clone()])
            .is_none()
    );
    assert!(scroll.contains(Point::new(40.0, 40.0)));
    scroll.clip = Rect::new(Point::new(100.0, 100.0), Size::new(20.0, 20.0));
    assert!(!scroll.contains(Point::new(40.0, 40.0)));
}

#[test]
fn scrollbar_track_clicks_reuse_offsets_and_degenerate_tracks_do_no_work() {
    let mut tree = UiTree::new(Element::container([]).keyed("scroll"));
    let node = tree.node_id_at(0).unwrap();
    let style = ScrollbarStyle::new(QuadStyle::default(), QuadStyle::default());
    let mut scroll = region(node, ScrollConfig::default(), 1_000.0);
    scroll.scrollbar = Some(ScrollbarRegion {
        track: scroll.bounds,
        thumb: Rect::new(Point::default(), Size::new(200.0, 40.0)),
        style: style.clone(),
    });

    let first = tree
        .scrollbar_pressed(Point::new(100.0, 100.0), &[scroll.clone()])
        .unwrap();
    assert!(first.scroll_changed);
    let second = tree
        .scrollbar_dragged(Point::new(100.0, 120.0), &[scroll.clone()])
        .unwrap();
    assert!(second.scroll_changed);
    tree.scrollbar_released();

    let mut no_travel = scroll.clone();
    no_travel.scrollbar = Some(ScrollbarRegion {
        track: no_travel.bounds,
        thumb: no_travel.bounds,
        style,
    });
    assert!(
        !tree
            .scrollbar_pressed(Point::new(20.0, 20.0), &[no_travel])
            .unwrap()
            .scroll_changed
    );
    tree.scrollbar_released();

    let mut no_range = scroll;
    no_range.max_offset.y = 0.0;
    assert!(
        !tree
            .scrollbar_pressed(Point::new(20.0, 20.0), &[no_range])
            .unwrap()
            .scroll_changed
    );
}

#[test]
fn direct_scroll_input_takes_over_from_a_scroll_motion() {
    let offset = Motion::new(Point::default());
    let mut tree = UiTree::new(
        Element::container([])
            .scrollable(ScrollConfig::default())
            .bind(property::Scroll, offset.clone()),
    );
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node, ScrollConfig::default(), 100.0)];
    offset.animate_to(
        Point::new(0.0, 60.0),
        Tween::new(Duration::from_millis(100)),
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::None
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(50_000_001)),
        TreeUpdate::Scroll
    );
    assert!((tree.scroll_offset(node).y - 30.0).abs() < 0.001);

    let update = tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Lines(Point::new(0.0, -1.0)),
        &regions,
    );
    assert!(update.scroll_changed);
    assert_eq!(offset.state(), MotionState::Idle);
    assert_eq!(offset.value(), Point::new(0.0, 40.0));
    assert_eq!(tree.scroll_offset(node), Point::new(0.0, 40.0));
}
