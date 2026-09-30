use super::{listens_for_scroll, region};
use argui_core::{Point, Rect, ScrollDelta, Size};
use argui_ui::{
    Color, Element, EventHandlerId, EventListener, EventOwnerId, EventType, QuadStyle, ScrollAxes,
    ScrollConfig, ScrollbarGeometry, ScrollbarPartStyle, ScrollbarRegion, ScrollbarStyle,
    UiEventKind, UiTree,
};

#[test]
fn disabled_native_scrolling_still_delivers_raw_wheel_to_custom_zoom_handlers() {
    let listener = EventListener::new(EventType::Wheel, EventHandlerId::new(EventOwnerId(1), 0));
    let mut tree = UiTree::new(Element::container([Element::container([]).on(listener)]));
    let node = tree.node_id_at(1).unwrap();
    let region = region(node, ScrollConfig::default().enabled(false), 100.);
    for delta in [
        ScrollDelta::Lines(Point::new(0., 1.)),
        ScrollDelta::Pixels(Point::new(2.5, -0.5)),
    ] {
        let update = tree.wheel_event(Point::new(10., 10.), delta, &[region.clone()]);
        assert_eq!(update.events.len(), 1);
        assert!(
            matches!(update.events[0].kind, UiEventKind::Wheel { delta: received, .. } if received == delta)
        );
        assert!(
            !tree
                .scroll(Point::new(10., 10.), delta, &[region.clone()])
                .scroll_changed
        );
    }
}

#[test]
fn nested_both_axis_timeline_keeps_side_wheel_and_vertical_scroll_independent() {
    let mut tree = UiTree::new(listens_for_scroll(Element::container([
        listens_for_scroll(Element::container([])),
    ])));
    let outer = tree.node_id_at(0).unwrap();
    let inner = tree.node_id_at(1).unwrap();
    let outer_region = region(outer, ScrollConfig::default(), 200.);
    let mut inner_region = region(inner, ScrollConfig::default().axes(ScrollAxes::Both), 0.);
    inner_region.max_offset.x = 500.;
    let regions = [outer_region, inner_region];
    let point = Point::new(20., 20.);
    tree.scroll(point, ScrollDelta::Pixels(Point::new(-30., 0.)), &regions);
    assert_eq!(tree.scroll_offset(inner).x, 30.);
    assert_eq!(tree.scroll_offset(outer).y, 0.);
    tree.scroll(point, ScrollDelta::Pixels(Point::new(0., -50.)), &regions);
    assert_eq!(tree.scroll_offset(inner).x, 30.);
    assert_eq!(tree.scroll_offset(outer).y, 50.);
    tree.scroll(
        point,
        ScrollDelta::Pixels(Point::new(-10.5, -5.25)),
        &regions,
    );
    assert_eq!(tree.scroll_offset(inner).x, 40.5);
    assert_eq!(tree.scroll_offset(outer).y, 55.25);
}

#[test]
fn horizontal_scrollbar_thumb_drag_uses_horizontal_geometry() {
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    );
    let config = ScrollConfig::default()
        .axes(argui_ui::ScrollAxes::Horizontal)
        .scrollbar(style.clone());
    let mut tree = UiTree::new(Element::container([]).scroll_config(config.clone()));
    let node = tree.node_id_at(0).unwrap();
    let mut scroll = region(node, config, 0.0);
    scroll.max_offset = Point::new(500.0, 0.0);
    scroll.scrollbar = Some(ScrollbarRegion {
        horizontal: Some(ScrollbarGeometry {
            track: scroll.bounds,
            thumb: Rect::new(Point::default(), Size::new(40.0, 200.0)),
        }),
        vertical: None,
        style,
    });
    let regions = [scroll];
    tree.scrollbar_pressed(Point::new(20.0, 100.0), &regions)
        .unwrap();
    tree.scrollbar_dragged(Point::new(180.0, 100.0), &regions)
        .unwrap();
    assert_eq!(tree.scroll_offset(node), Point::new(500.0, 0.0));
}
