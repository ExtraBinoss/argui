#[path = "event/wire.rs"]
mod wire_tests;

use argui_core::{
    Affine2D, Key, KeyInput, KeyState, Point, PointerEvent, PointerId, PointerPhase, Rect,
    ScrollDelta, Size,
};
use argui_paint::{ClipChain, ClipRegion};
use argui_ui::{
    CursorIcon, DismissPolicy, Element, EventHandlerId, EventListener, EventListenerOptions,
    EventOwnerId, EventPhase, EventType, FloatingPlacement, GestureDelivery, GestureEvent,
    GestureKind, GesturePhase, GestureSet, HitRegion, HitShape, Placement, SemanticAction, Sides,
    UiEvent, UiEventKind, UiTree, WindowLayer,
};
use std::sync::atomic::{AtomicU32, Ordering};

#[path = "event/listener/color.rs"]
mod color;
#[path = "event/listener.rs"]
mod filter;

static NEXT_LISTENER: AtomicU32 = AtomicU32::new(0);

trait TestListen {
    fn listen(self, event: EventType, options: EventListenerOptions) -> Self;
}

impl TestListen for Element {
    fn listen(self, event: EventType, options: EventListenerOptions) -> Self {
        let slot = NEXT_LISTENER.fetch_add(1, Ordering::Relaxed);
        let mut listener = EventListener::new(event, EventHandlerId::new(EventOwnerId(1), slot));
        listener.options = options;
        self.on(listener)
    }
}

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
    let events = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );

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
    let events = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    events[2].stop_propagation();
    assert!(events[3].should_dispatch());
    assert!(!events[4].should_dispatch());

    let events = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
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

    let enter = tree.event_deliveries(
        target,
        UiEventKind::Pointer(PointerEvent::mouse(PointerPhase::Entered, Point::default())),
    );
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

    let first = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert!(first[0].should_dispatch());
    first[0].stop_propagation();
    assert!(!first[1].should_dispatch());

    let second = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert!(second[1].should_dispatch());
    assert!(!second[1].should_dispatch());
    assert_eq!(
        tree.event_deliveries(
            target,
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )
        .len(),
        1
    );
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
        EventType::TextEdit,
        EventType::Submit,
        EventType::Gesture,
        EventType::SemanticAction,
        EventType::SelectionChange,
    ];
    assert!(hit_tested.into_iter().all(EventType::requires_hit_test));
    assert!(direct.into_iter().all(|event| !event.requires_hit_test()));

    let mut tree = UiTree::new(Element::text("target"));
    let target = tree.node_id_at(0).unwrap();
    let enter = UiEvent::new(
        target,
        None,
        UiEventKind::Pointer(PointerEvent::mouse(PointerPhase::Entered, Point::default())),
    );
    assert!(!enter.bubbles());
    assert!(!enter.cancelable());
    let click = UiEvent::new(
        target,
        None,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert!(click.bubbles());
    assert!(click.cancelable());
    assert!(click.prevent_default());
    assert!(click.default_prevented());

    assert!(
        tree.event_deliveries(
            target,
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )
        .is_empty()
    );
}

#[test]
fn every_event_kind_maps_to_its_dom_metadata() {
    let target = UiTree::new(Element::container([])).node_id_at(0).unwrap();
    let pointer = |phase| PointerEvent::mouse(phase, Point::new(2.0, 3.0));
    let key = KeyInput {
        key: Key::Enter,
        state: KeyState::Pressed,
        modifiers: argui_core::Modifiers::default(),
        repeat: false,
        text: None,
    };
    let gesture = GestureEvent {
        target,
        pointer: PointerId::MOUSE,
        phase: GesturePhase::Changed,
        kind: GestureKind::Pan {
            position: Point::default(),
            delta: Point::default(),
            total: Point::default(),
            velocity: Point::default(),
        },
        delivery: GestureDelivery::Immediate,
    };
    let cases = [
        (
            UiEventKind::Pointer(pointer(PointerPhase::Entered)),
            EventType::PointerEnter,
        ),
        (
            UiEventKind::Pointer(pointer(PointerPhase::Moved)),
            EventType::PointerMove,
        ),
        (
            UiEventKind::Pointer(pointer(PointerPhase::Pressed)),
            EventType::PointerDown,
        ),
        (
            UiEventKind::Pointer(pointer(PointerPhase::Released)),
            EventType::PointerUp,
        ),
        (
            UiEventKind::Pointer(pointer(PointerPhase::Left)),
            EventType::PointerLeave,
        ),
        (
            UiEventKind::Pointer(pointer(PointerPhase::Cancelled)),
            EventType::PointerCancel,
        ),
        (
            UiEventKind::PointerOutside(pointer(PointerPhase::Pressed)),
            EventType::PointerOutside,
        ),
        (
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
            EventType::Click,
        ),
        (
            UiEventKind::ContextMenu {
                position: Point::default(),
                capabilities: argui_ui::SelectionCapabilities::default(),
            },
            EventType::ContextMenu,
        ),
        (
            UiEventKind::GotPointerCapture(PointerId::MOUSE),
            EventType::GotPointerCapture,
        ),
        (
            UiEventKind::LostPointerCapture(PointerId::MOUSE),
            EventType::LostPointerCapture,
        ),
        (UiEventKind::KeyInput(key), EventType::Key),
        (
            UiEventKind::Wheel {
                delta: ScrollDelta::Pixels(Point::new(0.0, 1.0)),
                position: Point::default(),
            },
            EventType::Wheel,
        ),
        (
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 1.0),
                offset: Point::new(0.0, 2.0),
            },
            EventType::Scroll,
        ),
        (UiEventKind::Focused, EventType::Focus),
        (UiEventKind::Blurred, EventType::Blur),
        (UiEventKind::TextChanged("edit".into()), EventType::Input),
        (
            UiEventKind::TextEdited(argui_ui::TextEdit::new(1..2, "x")),
            EventType::TextEdit,
        ),
        (UiEventKind::Submitted("done".into()), EventType::Submit),
        (UiEventKind::Gesture(gesture), EventType::Gesture),
        (
            UiEventKind::SemanticAction {
                action: SemanticAction::Click,
                value: None,
            },
            EventType::SemanticAction,
        ),
        (
            UiEventKind::DocumentSelectionChanged {
                text: None,
                bounds: None,
                touch: false,
                dragging: false,
            },
            EventType::SelectionChange,
        ),
    ];

    for (kind, expected) in cases {
        assert_eq!(kind.event_type(), expected);
        let event = UiEvent::new(target, None, kind);
        assert_eq!(event.event_type(), expected);
    }
}

#[test]
fn multiple_listeners_keep_distinct_handler_identities() {
    let options = EventListenerOptions::default();
    let root = Element::container([Element::text("child")])
        .listen(EventType::Click, options)
        .listen(EventType::Click, options);
    assert_eq!(root.event_listeners.len(), 2);
    let mut tree = UiTree::new(root);
    let target = tree.node_ids()[0];
    let deliveries = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert_eq!(deliveries.len(), 2);
    assert_ne!(
        deliveries[0].current_handler(),
        deliveries[1].current_handler()
    );
}

#[test]
fn target_only_listeners_ignore_descendant_bubbling_without_affecting_other_listeners() {
    let options = EventListenerOptions::default();
    let root = Element::container([Element::text("child").keyed("child")])
        .keyed("root")
        .listen(EventType::Click, options.target_only(true))
        .listen(EventType::Click, options);
    let mut tree = UiTree::new(root);
    let child = tree.node_id_at(1).unwrap();
    let descendant = tree.event_deliveries(
        child,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert_eq!(descendant.len(), 1);
    assert_eq!(descendant[0].current_key(), Some("root"));
    assert_eq!(descendant[0].phase(), EventPhase::Bubble);

    let root = tree.node_id_at(0).unwrap();
    let direct = tree.event_deliveries(
        root,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert_eq!(direct.len(), 2);
    assert!(
        direct
            .iter()
            .all(|event| event.phase() == EventPhase::Target)
    );
}

#[test]
fn top_light_dismiss_portal_receives_pointer_outside() {
    let mut tree = UiTree::new(Element::container([
        Element::container([]).keyed("outside"),
        Element::container([Element::container([]).keyed("inside")])
            .keyed("portal")
            .portal(WindowLayer::Popover)
            .portal_dismiss(DismissPolicy::OutsidePointer)
            .listen(EventType::PointerOutside, EventListenerOptions::default()),
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
        focus_policy: argui_ui::FocusPolicy::None,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    };
    let outside = region(tree.node_id_at(1).unwrap());
    let update = tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(5.0, 5.0)),
        &[outside],
    );

    assert!(update.events.iter().any(|event| {
        event.target == tree.node_id_at(2).unwrap()
            && matches!(event.kind, UiEventKind::PointerOutside(_))
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
            .any(|event| matches!(event.kind, UiEventKind::PointerOutside(_)))
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
            .portal_dismiss(DismissPolicy::OutsidePointer)
            .listen(EventType::PointerOutside, EventListenerOptions::default()),
        Element::container([])
            .keyed("upper")
            .portal(WindowLayer::Popover)
            .portal_dismiss(DismissPolicy::OutsidePointer)
            .listen(EventType::PointerOutside, EventListenerOptions::default()),
    ]));
    let update = tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(50.0, 50.0)),
        &[],
    );
    let outside = update
        .events
        .iter()
        .filter(|event| matches!(event.kind, UiEventKind::PointerOutside(_)))
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
        focus_policy: argui_ui::FocusPolicy::None,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
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
            .all(|event| !matches!(event.kind, UiEventKind::PointerOutside(_)))
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
    let delivery = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert!(delivery[0].should_dispatch());

    tree.update(Element::container(
        [Element::text("target").keyed("stable")],
    ));
    assert!(
        tree.event_deliveries(
            target,
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )
        .is_empty()
    );
    tree.update(Element::container([once()]));
    let delivery = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert!(delivery[0].should_dispatch());

    tree.update(Element::container([]));
    assert!(
        tree.event_deliveries(
            target,
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )
        .is_empty()
    );
}

#[test]
fn stopping_propagation_is_idempotent_and_preserves_the_first_boundary() {
    let mut tree = event_tree();
    let target = tree.node_id_at(2).unwrap();
    let events = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    events[2].stop_propagation();
    events[4].stop_propagation();
    assert!(events[3].should_dispatch());
    assert!(!events[4].should_dispatch());
}

#[test]
fn public_event_delivery_preserves_capture_target_and_bubble_order() {
    let listener = |slot, capture| {
        EventListener::new(EventType::Click, EventHandlerId::new(EventOwnerId(1), slot))
            .capture(capture)
    };
    let root =
        Element::container([Element::text("target").on(listener(1, false))]).on(listener(2, true));
    let mut tree = UiTree::new(root);
    let target = tree.node_id_at(1).unwrap();
    let deliveries = tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );

    assert_eq!(deliveries.len(), 2);
    assert_eq!(deliveries[0].event_type(), EventType::Click);
    assert_eq!(deliveries[1].event_type(), EventType::Click);
}
