use argui_animation::{Duration, Motion, MotionState, Time, Tween};
use argui_core::{Affine2D, Point, Rect, ScrollDelta, Size};
use argui_paint::{ClipChain, ClipRegion};
use argui_ui::{
    Axes, Color, CursorIcon, Element, GestureSet, HitRegion, Overflow, QuadStyle, ScrollChaining,
    ScrollConfig, ScrollPolarity, ScrollRegion, ScrollbarPartStyle, ScrollbarRegion,
    ScrollbarStyle, StateName, StateScopeId, StateSelector, StylePatch, StyleTransition,
    Transition, TreeUpdate, UiEventKind, UiTree, VisualState, property, scrollbar_at,
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
        interaction_order: 0,
    }
}

fn hit_region(node: argui_ui::NodeId, bounds: Rect) -> HitRegion {
    HitRegion {
        node,
        bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled: true,
        focusable: false,
        cursor: CursorIcon::Default,
        gestures: GestureSet::NONE,
        window_drag: None,
    }
}

#[test]
fn scroll_polarity_is_explicit_and_offsets_are_clamped() {
    let mut tree = UiTree::new(
        Element::container([])
            .keyed("scroll")
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scroll_config(ScrollConfig::default()),
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
        ScrollbarPartStyle::new(QuadStyle::solid(Color::srgb(0.0, 0.0, 0.0))),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
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
    assert!(tree.scrollbar_released().is_some());
    assert!(tree.scrollbar_released().is_none());
}

#[test]
fn scrollbar_parts_reuse_retained_state_transitions() {
    let base = Color::srgb(0.1, 0.2, 0.3);
    let hovered = Color::srgb(0.7, 0.8, 0.9);
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(base))
            .when(
                VisualState::Hovered,
                StylePatch::new()
                    .set(property::BackgroundColor, hovered)
                    .set(property::CornerRadii, [5.0; 4]),
            )
            .when(
                VisualState::Pressed,
                StylePatch::new().set(property::Opacity, 0.7),
            )
            .transition(StyleTransition::new(Transition::tween(Tween::new(
                Duration::from_millis(100),
            )))),
    );
    let config = ScrollConfig::default().scrollbar(style.clone());
    let mut tree = UiTree::new(
        Element::container([])
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scroll_config(config.clone()),
    );
    let node = tree.node_id_at(0).unwrap();
    let mut scroll = region(node, config.clone(), 1_000.0);
    scroll.scrollbar = Some(ScrollbarRegion {
        track: scroll.bounds,
        thumb: Rect::new(Point::default(), Size::new(200.0, 40.0)),
        style,
    });

    let regions = [scroll];
    let point = Point::new(20.0, 20.0);
    let update = tree.scrollbar_pointer_moved(Some(point), &regions);
    assert!(update.paint_changed);
    assert!(tree.wants_animation_frame());
    assert_eq!(tree.set_reduced_motion(true), TreeUpdate::Paint);
    let resolved = tree.resolved_scroll_config(node, &config);
    let thumb = resolved.scrollbar.unwrap().thumb.base;
    assert_eq!(thumb.background, Some(argui_ui::Fill::Solid(hovered)));
    assert_eq!(thumb.radii.as_array(), [5.0; 4]);
    tree.scrollbar_pressed(point, &regions).unwrap();
    assert_eq!(
        tree.resolved_scroll_config(node, &config)
            .scrollbar
            .unwrap()
            .thumb
            .base
            .radii
            .as_array(),
        [5.0; 4]
    );
    assert!(tree.scrollbar_pointer_moved(None, &[]).paint_changed);
}

#[test]
fn scrollbar_parts_share_named_and_scoped_state_resolution() {
    let scope = StateScopeId::new("scroll-host");
    let active = StateName::new("active");
    let selected = Color::srgb(0.8, 0.3, 0.2);
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE))
            .when(active, StylePatch::new().set(property::Opacity, 0.6))
            .when(
                StateSelector::scope(scope, active),
                StylePatch::new().set(property::BackgroundColor, selected),
            ),
    );
    let config = ScrollConfig::default().scrollbar(style);
    let element = Element::container([])
        .state_scope(scope)
        .active_state(active, true)
        .scroll_config(config.clone());
    let tree = UiTree::new(element);
    let thumb = tree
        .resolved_scroll_config(tree.node_ids()[0], &config)
        .scrollbar
        .unwrap()
        .thumb
        .base;
    assert_eq!(thumb.opacity, 0.6);
    assert_eq!(thumb.background, Some(argui_ui::Fill::Solid(selected)));
}

#[test]
fn scrollbar_hit_testing_follows_paint_order() {
    let tree = UiTree::new(Element::column([
        Element::container([]),
        Element::container([]),
    ]));
    let scroll_node = tree.node_id_at(1).unwrap();
    let overlay_node = tree.node_id_at(2).unwrap();
    let mut scroll = region(scroll_node, ScrollConfig::default(), 100.0);
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::default()),
    );
    scroll.scrollbar = Some(ScrollbarRegion {
        track: scroll.bounds,
        thumb: scroll.bounds,
        style,
    });
    scroll.interaction_order = 1;
    let point = Point::new(20.0, 20.0);
    let scroll_hit = hit_region(scroll_node, scroll.bounds);

    assert_eq!(
        scrollbar_at(
            point,
            std::slice::from_ref(&scroll),
            std::slice::from_ref(&scroll_hit),
        )
        .map(|region| region.node),
        Some(scroll_node)
    );
    assert!(
        scrollbar_at(
            point,
            std::slice::from_ref(&scroll),
            &[scroll_hit, hit_region(overlay_node, scroll.bounds)],
        )
        .is_none()
    );
}

#[test]
fn scrollbar_ignores_track_outside_the_effective_clip() {
    let mut tree = UiTree::new(Element::container([]));
    let node = tree.node_id_at(0).unwrap();
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::default()),
    );
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
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::default()),
    );
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
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scroll_config(ScrollConfig::default())
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
