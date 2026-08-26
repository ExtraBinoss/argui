use argui_core::{Point, Rect, ScrollDelta, Size};
use argui_ui::{
    Color, Element, QuadStyle, ScrollConfig, ScrollPolarity, ScrollRegion, ScrollbarRegion,
    ScrollbarStyle, UiEventKind, UiTree,
};

fn region(node: argui_ui::NodeId, config: ScrollConfig, max_y: f32) -> ScrollRegion {
    let bounds = Rect::new(Point::default(), Size::new(200.0, 200.0));
    ScrollRegion {
        node,
        bounds,
        clip: bounds,
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
fn scrollbar_track_and_thumb_drive_the_retained_offset() {
    let mut tree = UiTree::new(Element::container([]).keyed("scroll"));
    let node = tree.node_id_at(0).unwrap();
    let style = ScrollbarStyle::new(
        QuadStyle::solid(Color::rgb(0.0, 0.0, 0.0)),
        QuadStyle::solid(Color::WHITE),
    );
    let mut scroll = region(node, ScrollConfig::default().scrollbar(style), 1_000.0);
    scroll.scrollbar = Some(ScrollbarRegion {
        track: scroll.bounds,
        thumb: Rect::new(Point::default(), Size::new(200.0, 40.0)),
        style,
    });
    let regions = [scroll];

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
        style,
    });

    assert!(!scroll.scrollbar_contains(Point::new(100.0, 100.0)));
    assert!(
        tree.scrollbar_pressed(Point::new(100.0, 100.0), &[scroll])
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
        style,
    });

    let first = tree
        .scrollbar_pressed(Point::new(100.0, 100.0), &[scroll])
        .unwrap();
    assert!(first.scroll_changed);
    let second = tree
        .scrollbar_dragged(Point::new(100.0, 120.0), &[scroll])
        .unwrap();
    assert!(second.scroll_changed);
    tree.scrollbar_released();

    let mut no_travel = scroll;
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
