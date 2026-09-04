use argui_core::{Affine2D, Point, PointerEvent, PointerPhase, Rect, ScrollDelta, Size};
use argui_paint::{ClipChain, ClipRegion};
use argui_ui::{
    CursorIcon, DismissPolicy, Element, EventListenerOptions, EventOwnerId, EventPhase, EventType,
    FloatingPlacement, GestureSet, HitRegion, HitShape, Placement, Sides, UiEvent, UiEventKind,
    UiTree, WindowLayer,
};

fn event_tree() -> UiTree {
    UiTree::new(
        Element::column([Element::container([Element::text("target")
            .keyed("target")
            .listen(
                EventType::Click,
                EventListenerOptions::default().capture(true),
            )
            .listen(EventType::Click, EventListenerOptions::default())])
        .keyed("parent")
        .listen(
            EventType::Click,
            EventListenerOptions::default().capture(true),
        )
        .listen(EventType::Click, EventListenerOptions::default())])
        .keyed("root")
        .listen(
            EventType::Click,
            EventListenerOptions::default().capture(true),
        )
        .listen(EventType::Click, EventListenerOptions::default()),
    )
}

#[test]
fn capture_target_and_bubble_preserve_dom_target_semantics() {
    let mut tree = event_tree();
    let target = tree.node_id_at(2).unwrap();
    let events = tree.event_deliveries(target, UiEventKind::Clicked);

    assert_eq!(
        events
            .iter()
            .map(|event| (event.phase(), event.current_key().unwrap()))
            .collect::<Vec<_>>(),
        [
            (EventPhase::Capture, "root"),
            (EventPhase::Capture, "parent"),
            (EventPhase::Target, "target"),
            (EventPhase::Target, "target"),
            (EventPhase::Bubble, "parent"),
            (EventPhase::Bubble, "root"),
        ]
    );
    assert!(events.iter().all(|event| {
        event.target == target
            && event.target_key() == Some("target")
            && event.event_type() == EventType::Click
            && event.bubbles()
            && event.cancelable()
    }));
}

#[test]
fn propagation_stops_after_the_current_target_and_immediate_stops_on_it() {
    let mut tree = event_tree();
    let target = tree.node_id_at(2).unwrap();
    let events = tree.event_deliveries(target, UiEventKind::Clicked);
    events[2].stop_propagation();
    assert!(events[3].should_dispatch());
    assert!(!events[4].should_dispatch());

    let events = tree.event_deliveries(target, UiEventKind::Clicked);
    events[2].stop_immediate_propagation();
    assert!(!events[3].should_dispatch());
    assert!(events[2].propagation_stopped());
}

#[test]
fn passive_and_non_cancelable_events_cannot_prevent_defaults() {
    let root = Element::container([])
        .listen(
            EventType::Wheel,
            EventListenerOptions::default().passive(true),
        )
        .listen(EventType::PointerEnter, EventListenerOptions::default());
    let mut tree = UiTree::new(root);
    let target = tree.node_id_at(0).unwrap();

    let wheel = tree.event_deliveries(
        target,
        UiEventKind::Wheel {
            delta: ScrollDelta::Pixels(Point::new(0.0, 8.0)),
            position: Point::default(),
        },
    );
    assert!(!wheel[0].prevent_default());
    assert!(!wheel[0].default_prevented());

    let enter = tree.event_deliveries(target, UiEventKind::PointerEntered);
    assert!(!enter[0].cancelable());
    assert!(!enter[0].prevent_default());
}

#[test]
fn once_is_consumed_only_when_the_listener_is_actually_dispatched() {
    let root = Element::container([Element::text("target")
        .listen(EventType::Click, EventListenerOptions::default().once(true))])
    .listen(
        EventType::Click,
        EventListenerOptions::default().capture(true),
    );
    let mut tree = UiTree::new(root);
    let target = tree.node_id_at(1).unwrap();

    let first = tree.event_deliveries(target, UiEventKind::Clicked);
    assert!(first[0].should_dispatch());
    first[0].stop_propagation();
    assert!(!first[1].should_dispatch());

    let second = tree.event_deliveries(target, UiEventKind::Clicked);
    assert!(second[1].should_dispatch());
    assert!(!second[1].should_dispatch());
    assert_eq!(tree.event_deliveries(target, UiEventKind::Clicked).len(), 1);
}

#[test]
fn event_metadata_matches_dom_delivery_rules() {
    let hit_tested = [
        EventType::PointerEnter,
        EventType::PointerLeave,
        EventType::PointerMove,
        EventType::PointerDown,
        EventType::PointerUp,
        EventType::Click,
        EventType::ContextMenu,
        EventType::Wheel,
        EventType::Scroll,
    ];
    let direct = [
        EventType::PointerOutside,
        EventType::GotPointerCapture,
        EventType::LostPointerCapture,
        EventType::Key,
        EventType::Focus,
        EventType::Blur,
        EventType::Input,
        EventType::Submit,
        EventType::Gesture,
        EventType::SemanticAction,
        EventType::SelectionChange,
    ];
    assert!(hit_tested.into_iter().all(EventType::requires_hit_test));
    assert!(direct.into_iter().all(|event| !event.requires_hit_test()));

    let mut tree = UiTree::new(Element::text("target"));
    let target = tree.node_id_at(0).unwrap();
    let enter = UiEvent::new(target, None, UiEventKind::PointerEntered);
    assert!(!enter.bubbles());
    assert!(!enter.cancelable());
    let click = UiEvent::new(target, None, UiEventKind::Clicked);
    assert!(click.bubbles());
    assert!(click.cancelable());
    assert!(click.prevent_default());
    assert!(click.default_prevented());

    let implicit = tree.event_deliveries(target, UiEventKind::Clicked);
    assert_eq!(implicit.len(), 1);
    assert_eq!(implicit[0].phase(), EventPhase::Target);
}

#[test]
fn listeners_are_unique_and_event_owners_are_stable() {
    let options = EventListenerOptions::default();
    let mut root = Element::container([Element::text("child")])
        .listen(EventType::Click, options)
        .listen(EventType::Click, options);
    assert_eq!(root.event_listeners.len(), 1);

    root.assign_event_owner(EventOwnerId(7));
    root.assign_event_owner(EventOwnerId(9));
    assert_eq!(root.event_owner, Some(EventOwnerId(7)));
    assert_eq!(root.children[0].event_owner, Some(EventOwnerId(7)));
}

#[test]
fn top_light_dismiss_portal_receives_pointer_outside() {
    let mut tree = UiTree::new(Element::container([
        Element::container([]).keyed("outside"),
        Element::container([Element::container([]).keyed("inside")])
            .keyed("portal")
            .portal(WindowLayer::Popover)
            .portal_dismiss(DismissPolicy::OutsidePointer),
    ]));
    let bounds = Rect::new(Point::default(), Size::new(20.0, 20.0));
    let region = |node| HitRegion {
        node,
        bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        shape: HitShape::Bounds,
        slop: Sides::default(),
        enabled: true,
        focusable: false,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::NONE,
        window_drag: None,
    };
    let outside = region(tree.node_id_at(1).unwrap());
    let update = tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(5.0, 5.0)),
        &[outside],
    );

    assert!(update.events.iter().any(|event| {
        event.target == tree.node_id_at(2).unwrap()
            && matches!(event.kind, UiEventKind::PointerOutside)
    }));

    let inside = region(tree.node_id_at(3).unwrap());
    let update = tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(5.0, 5.0)),
        &[inside],
    );
    assert!(
        !update
            .events
            .iter()
            .any(|event| matches!(event.kind, UiEventKind::PointerOutside))
    );
}

#[test]
fn manual_portals_are_ignored_and_only_the_top_outside_portal_dismisses() {
    let mut tree = UiTree::new(Element::container([
        Element::container([])
            .keyed("manual")
            .portal(WindowLayer::Popover),
        Element::container([])
            .keyed("lower")
            .portal(WindowLayer::Popover)
            .portal_dismiss(DismissPolicy::OutsidePointer),
        Element::container([])
            .keyed("upper")
            .portal(WindowLayer::Popover)
            .portal_dismiss(DismissPolicy::OutsidePointer),
    ]));
    let update = tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(50.0, 50.0)),
        &[],
    );
    let outside = update
        .events
        .iter()
        .filter(|event| matches!(event.kind, UiEventKind::PointerOutside))
        .collect::<Vec<_>>();

    assert_eq!(outside.len(), 1);
    assert_eq!(outside[0].target, tree.node_id_at(3).unwrap());
}

#[test]
fn anchored_portal_treats_its_trigger_subtree_as_inside() {
    let mut tree = UiTree::new(Element::container([
        Element::container([Element::container([]).keyed("trigger-child")]).keyed("trigger"),
        Element::container([])
            .keyed("popover")
            .anchored_portal(
                WindowLayer::Popover,
                "trigger",
                FloatingPlacement::new(Placement::BottomStart),
            )
            .portal_dismiss(DismissPolicy::OutsidePointer),
    ]));
    let bounds = Rect::new(Point::default(), Size::new(20.0, 20.0));
    let trigger_child = HitRegion {
        node: tree.node_id_at(2).unwrap(),
        bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        shape: HitShape::Bounds,
        slop: Sides::default(),
        enabled: true,
        focusable: false,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::NONE,
        window_drag: None,
    };
    let update = tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(5.0, 5.0)),
        &[trigger_child],
    );

    assert!(
        update
            .events
            .iter()
            .all(|event| !matches!(event.kind, UiEventKind::PointerOutside))
    );
}

#[test]
fn registry_forgets_removed_targets_and_removed_once_listeners() {
    let once = || {
        Element::text("target")
            .keyed("stable")
            .listen(EventType::Click, EventListenerOptions::default().once(true))
    };
    let mut tree = UiTree::new(Element::container([once()]));
    let target = tree.node_id_at(1).unwrap();
    let delivery = tree.event_deliveries(target, UiEventKind::Clicked);
    assert!(delivery[0].should_dispatch());

    tree.update(Element::container(
        [Element::text("target").keyed("stable")],
    ));
    assert_eq!(tree.event_deliveries(target, UiEventKind::Clicked).len(), 1);
    tree.update(Element::container([once()]));
    let delivery = tree.event_deliveries(target, UiEventKind::Clicked);
    assert!(delivery[0].should_dispatch());

    tree.update(Element::container([]));
    assert!(
        tree.event_deliveries(target, UiEventKind::Clicked)
            .is_empty()
    );
}

#[test]
fn stopping_propagation_is_idempotent_and_preserves_the_first_boundary() {
    let mut tree = event_tree();
    let target = tree.node_id_at(2).unwrap();
    let events = tree.event_deliveries(target, UiEventKind::Clicked);
    events[2].stop_propagation();
    events[4].stop_propagation();
    assert!(events[3].should_dispatch());
    assert!(!events[4].should_dispatch());
}
