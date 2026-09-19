use std::time::Duration;

use argui_core::{Point, PointerButton, PointerEvent, PointerId, PointerKind, PointerPhase};
use argui_ui::{
    Element, EventHandlerId, EventListener, EventOwnerId, EventType, GestureArena, GestureCapture,
    GestureDelivery, GestureKind, GesturePhase, GestureSet, HitRegion, HitShape, Interaction,
    PanAxis, PanGesture, PinchGesture, RotationGesture, TapGesture, UiEventKind, UiTree,
};

fn target() -> argui_ui::NodeId {
    UiTree::new(Element::container([])).node_id_at(0).unwrap()
}

fn touch(id: u64, phase: PointerPhase, x: f32, y: f32, millis: u64) -> PointerEvent {
    PointerEvent {
        id: PointerId::new(id),
        kind: PointerKind::Touch,
        phase,
        position: Point::new(x, y),
        button: Some(PointerButton::Primary),
        buttons: u16::from(!matches!(
            phase,
            PointerPhase::Released | PointerPhase::Cancelled
        )),
        pressure: Some(0.5),
        primary: id == 1,
        modifiers: argui_core::Modifiers::default(),
        timestamp: Duration::from_millis(millis),
    }
}

#[test]
fn gesture_configuration_composes_without_hidden_defaults() {
    let pan = PanGesture::default()
        .axis(PanAxis::Horizontal)
        .threshold(3.0)
        .capture(GestureCapture::OnPress)
        .delivery(GestureDelivery::FrameCoalesced);
    let pinch = PinchGesture::default().delivery(GestureDelivery::FrameCoalesced);
    let rotation = RotationGesture::default().delivery(GestureDelivery::FrameCoalesced);
    let gestures = GestureSet::default()
        .tap(TapGesture::default())
        .pan(pan)
        .pinch(pinch)
        .rotation(rotation);

    assert_eq!(gestures.pan, Some(pan));
    assert_eq!(gestures.pinch, Some(pinch));
    assert_eq!(gestures.rotation, Some(rotation));
    assert!(gestures.captures_on_press());
    assert!(!gestures.is_empty());
    assert!(GestureSet::EMPTY.is_empty());
    assert!(!GestureSet::EMPTY.captures_on_press());
}

#[test]
fn on_press_capture_is_observable_until_the_contact_ends() {
    let gestures = GestureSet::EMPTY.pan(
        PanGesture::default()
            .immediate()
            .capture(GestureCapture::OnPress),
    );
    let mut tree =
        UiTree::new(Element::container([]).interaction(Interaction::default().gestures(gestures)));
    let node = tree.node_ids()[0];
    let region = HitRegion {
        node,
        bounds: argui_core::Rect::new(Point::default(), argui_core::Size::new(100.0, 100.0)),
        transform: argui_core::Affine2D::IDENTITY,
        clips: argui_paint::ClipChain::default(),
        shape: HitShape::Bounds,
        slop: argui_ui::Sides::default(),
        enabled: true,
        focus_policy: argui_ui::FocusPolicy::None,
        cursor: argui_ui::CursorIcon::Auto,
        gestures,
        window_drag: None,
    };
    let pointer = PointerId::new(1);
    tree.pointer_event(
        touch(1, PointerPhase::Pressed, 10.0, 10.0, 0),
        std::slice::from_ref(&region),
    );
    assert!(tree.pointer_captured(pointer));
    tree.pointer_event(
        touch(1, PointerPhase::Released, 10.0, 10.0, 10),
        std::slice::from_ref(&region),
    );
    assert!(!tree.pointer_captured(pointer));
}

#[test]
fn pan_axes_filter_delta_total_and_velocity() {
    let node = target();
    for (axis, expected) in [
        (PanAxis::Horizontal, Point::new(12.0, 0.0)),
        (PanAxis::Vertical, Point::new(0.0, 9.0)),
        (PanAxis::Both, Point::new(12.0, 9.0)),
    ] {
        let mut arena = GestureArena::default();
        arena.update(
            touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
            Some((
                node,
                GestureSet::EMPTY.pan(PanGesture::default().axis(axis).immediate()),
            )),
        );
        let events = arena.update(touch(1, PointerPhase::Moved, 12.0, 9.0, 10), None);
        let GestureKind::Pan {
            delta,
            total,
            velocity,
            ..
        } = events[0].kind
        else {
            panic!("expected pan");
        };
        assert_eq!(delta, expected);
        assert_eq!(total, expected);
        assert_eq!(velocity.x == 0.0, expected.x == 0.0);
        assert_eq!(velocity.y == 0.0, expected.y == 0.0);
    }
}

#[test]
fn short_stationary_contacts_resolve_as_taps() {
    let node = target();
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 10.0, 20.0, 0),
        Some((node, GestureSet::EMPTY.tap(TapGesture::default()))),
    );
    let events = arena.update(touch(1, PointerPhase::Released, 12.0, 21.0, 120), None);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].phase, GesturePhase::Ended);
    assert!(matches!(events[0].kind, GestureKind::Tap { .. }));
}

#[test]
fn pan_only_gestures_start_immediately_and_report_velocity() {
    let node = target();
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((
            node,
            GestureSet::EMPTY.pan(PanGesture::default().immediate()),
        )),
    );
    let started = arena.update(touch(1, PointerPhase::Moved, 4.0, 0.0, 10), None);
    let ended = arena.update(touch(1, PointerPhase::Released, 20.0, 0.0, 30), None);

    assert_eq!(started[0].phase, GesturePhase::Changed);
    let GestureKind::Pan {
        total, velocity, ..
    } = started[0].kind
    else {
        panic!("expected pan");
    };
    assert_eq!(total, Point::new(4.0, 0.0));
    assert!(velocity.x > 0.0);
    assert_eq!(ended[0].phase, GesturePhase::Ended);
}

#[test]
fn immediate_pan_begins_at_the_press_position() {
    let node = target();
    let mut arena = GestureArena::default();
    let started = arena.update(
        touch(1, PointerPhase::Pressed, 42.0, 18.0, 0),
        Some((
            node,
            GestureSet::EMPTY.pan(PanGesture::default().immediate()),
        )),
    );

    assert_eq!(started.len(), 1);
    assert_eq!(started[0].phase, GesturePhase::Started);
    let GestureKind::Pan {
        position, total, ..
    } = started[0].kind
    else {
        panic!("expected an immediate pan");
    };
    assert_eq!(position, Point::new(42.0, 18.0));
    assert_eq!(total, Point::default());

    let ended = arena.update(touch(1, PointerPhase::Released, 60.0, 18.0, 20), None);
    let GestureKind::Pan { position, .. } = ended[0].kind else {
        panic!("expected a completed pan");
    };
    assert_eq!(position, Point::new(60.0, 18.0));
}

#[test]
fn tap_and_pan_gestures_keep_a_drag_threshold() {
    let node = target();
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((
            node,
            GestureSet::EMPTY
                .tap(TapGesture::default())
                .pan(PanGesture::default()),
        )),
    );
    assert!(
        arena
            .update(touch(1, PointerPhase::Moved, 4.0, 0.0, 10), None)
            .is_empty()
    );
    let started = arena.update(touch(1, PointerPhase::Moved, 12.0, 0.0, 20), None);
    assert_eq!(started[0].phase, GesturePhase::Started);
    assert!(matches!(started[0].kind, GestureKind::Pan { .. }));
}

#[test]
fn two_contacts_can_recognize_pinch_and_rotation_together() {
    let node = target();
    let gestures = GestureSet::EMPTY
        .pinch(PinchGesture::default())
        .rotation(RotationGesture::default());
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, gestures)),
    );
    arena.update(
        touch(2, PointerPhase::Pressed, 10.0, 0.0, 0),
        Some((node, gestures)),
    );
    let events = arena.update(touch(2, PointerPhase::Moved, 12.0, 4.0, 16), None);

    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind, GestureKind::Pinch { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind, GestureKind::Rotation { .. }))
    );
    let cancelled = arena.cancel_all();
    assert!(
        cancelled
            .iter()
            .all(|event| event.phase == GesturePhase::Cancelled)
    );
}

#[test]
fn unrelated_and_cancelled_contacts_produce_no_tap() {
    let node = target();
    let mut arena = GestureArena::default();
    assert!(
        arena
            .update(touch(1, PointerPhase::Entered, 0.0, 0.0, 0), None)
            .is_empty()
    );
    assert!(
        arena
            .update(touch(9, PointerPhase::Moved, 4.0, 0.0, 1), None)
            .is_empty()
    );
    assert!(
        arena
            .update(touch(9, PointerPhase::Released, 4.0, 0.0, 2), None)
            .is_empty()
    );
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::EMPTY.tap(TapGesture::default()))),
    );
    assert!(
        arena
            .update(touch(1, PointerPhase::Cancelled, 0.0, 0.0, 20), None)
            .is_empty()
    );
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::EMPTY.tap(TapGesture::default()))),
    );
    assert!(
        arena
            .update(touch(1, PointerPhase::Released, 0.0, 0.0, 600), None)
            .is_empty()
    );
    assert!(
        arena
            .update(touch(2, PointerPhase::Pressed, 0.0, 0.0, 0), None)
            .is_empty()
    );
}

#[test]
fn active_pans_survive_pointer_leave_and_cancel_only_explicitly() {
    let node = target();
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::EMPTY.pan(PanGesture::default()))),
    );
    let started = arena.update(touch(1, PointerPhase::Moved, 9.0, 0.0, 0), None);
    let GestureKind::Pan { velocity, .. } = started[0].kind else {
        panic!("expected pan");
    };
    assert_eq!(velocity, Point::default());
    for step in 1..=8 {
        let events = arena.update(
            touch(1, PointerPhase::Moved, 9.0 + step as f32, 0.0, step * 40),
            None,
        );
        assert_eq!(events[0].phase, GesturePhase::Changed);
    }
    assert!(
        arena
            .update(touch(1, PointerPhase::Left, 17.0, 0.0, 340), None)
            .is_empty()
    );
    let resumed = arena.update(touch(1, PointerPhase::Moved, 18.0, 0.0, 360), None);
    assert_eq!(resumed[0].phase, GesturePhase::Changed);
    let ended = arena.update(touch(1, PointerPhase::Released, 18.0, 0.0, 380), None);
    assert_eq!(ended[0].phase, GesturePhase::Ended);

    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 400),
        Some((node, GestureSet::EMPTY.pan(PanGesture::default()))),
    );
    arena.update(touch(1, PointerPhase::Moved, 9.0, 0.0, 410), None);
    let cancelled = arena.update(touch(1, PointerPhase::Cancelled, 9.0, 0.0, 420), None);
    assert_eq!(cancelled[0].phase, GesturePhase::Cancelled);
}

#[test]
fn paired_gestures_emit_changed_and_ended_phases() {
    let node = target();
    let gestures = GestureSet::EMPTY
        .tap(TapGesture::default())
        .pan(PanGesture::default())
        .pinch(PinchGesture::default())
        .rotation(RotationGesture::default());
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, gestures)),
    );
    arena.update(
        touch(2, PointerPhase::Pressed, 10.0, 0.0, 0),
        Some((node, gestures)),
    );
    let started = arena.update(touch(2, PointerPhase::Moved, 13.0, 4.0, 16), None);
    assert!(
        started
            .iter()
            .all(|event| event.phase == GesturePhase::Started)
    );
    let changed = arena.update(touch(2, PointerPhase::Moved, 16.0, 7.0, 32), None);
    assert!(
        changed
            .iter()
            .all(|event| event.phase == GesturePhase::Changed)
    );
    let ended = arena.update(touch(2, PointerPhase::Released, 16.0, 7.0, 48), None);
    assert_eq!(ended.len(), 2);
    assert!(ended.iter().all(|event| event.phase == GesturePhase::Ended));
}

#[test]
fn rotation_wraps_across_the_pi_boundary() {
    let node = target();
    let gestures = GestureSet::EMPTY.rotation(RotationGesture::default());
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, gestures)),
    );
    arena.update(
        touch(2, PointerPhase::Pressed, -10.0, 0.1, 0),
        Some((node, gestures)),
    );
    let events = arena.update(touch(2, PointerPhase::Moved, -10.0, -1.0, 16), None);
    let GestureKind::Rotation { radians } = events[0].kind else {
        panic!("expected rotation");
    };
    assert!(radians.abs() < 0.2);

    let mut opposite = GestureArena::default();
    opposite.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, gestures)),
    );
    opposite.update(
        touch(2, PointerPhase::Pressed, -10.0, -0.1, 0),
        Some((node, gestures)),
    );
    assert!(
        opposite
            .update(touch(2, PointerPhase::Moved, -10.0, -0.2, 8), None)
            .is_empty()
    );
    let events = opposite.update(touch(2, PointerPhase::Moved, -10.0, 1.0, 16), None);
    let GestureKind::Rotation { radians } = events[0].kind else {
        panic!("expected opposite rotation");
    };
    assert!(radians.abs() < 0.2);
}

#[test]
fn arena_edge_paths_keep_contacts_and_thresholds_independent() {
    let tree = UiTree::new(Element::column([Element::container([])]));
    let node = tree.node_id_at(0).unwrap();
    let other = tree.node_id_at(1).unwrap();

    let mut cancelled_pan = GestureArena::default();
    cancelled_pan.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::EMPTY.pan(PanGesture::default()))),
    );
    cancelled_pan.update(touch(1, PointerPhase::Moved, 9.0, 0.0, 1), None);
    assert_eq!(cancelled_pan.cancel_all()[0].phase, GesturePhase::Cancelled);

    let mut no_pan = GestureArena::default();
    no_pan.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::EMPTY.tap(TapGesture::default()))),
    );
    assert!(
        no_pan
            .update(touch(1, PointerPhase::Moved, 20.0, 0.0, 10), None)
            .is_empty()
    );
    assert!(
        no_pan
            .update(touch(1, PointerPhase::Released, 20.0, 0.0, 20), None)
            .is_empty()
    );
    no_pan.update(
        touch(2, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::EMPTY.pan(PanGesture::default()))),
    );
    assert!(
        no_pan
            .update(touch(2, PointerPhase::Released, 1.0, 0.0, 10), None)
            .is_empty()
    );

    let mut pair = GestureArena::default();
    let pair_gestures = GestureSet::EMPTY.pinch(PinchGesture::default());
    pair.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, pair_gestures)),
    );
    pair.update(
        touch(2, PointerPhase::Pressed, 10.0, 0.0, 0),
        Some((node, pair_gestures)),
    );
    pair.update(
        touch(3, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((other, GestureSet::EMPTY.pan(PanGesture::default()))),
    );
    assert_eq!(
        pair.update(touch(3, PointerPhase::Moved, 10.0, 0.0, 8), None)[0].phase,
        GesturePhase::Started
    );
    assert!(
        pair.update(touch(2, PointerPhase::Moved, 10.1, 0.0, 10), None)
            .is_empty()
    );
    pair.update(touch(3, PointerPhase::Released, 10.0, 0.0, 12), None);
    pair.update(touch(2, PointerPhase::Released, 10.1, 0.0, 14), None);

    let mut dense_history = GestureArena::default();
    dense_history.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::EMPTY.pan(PanGesture::default()))),
    );
    for step in 1..=8 {
        dense_history.update(
            touch(1, PointerPhase::Moved, step as f32 * 9.0, 0.0, step),
            None,
        );
    }
}

#[test]
fn frame_coalesced_pan_keeps_the_latest_sample_and_accumulated_delta() {
    let gestures = GestureSet::EMPTY.pan(
        PanGesture::default()
            .immediate()
            .delivery(GestureDelivery::FrameCoalesced),
    );
    let root = Element::container([])
        .interaction(Interaction::default().gestures(gestures))
        .on(EventListener::new(
            EventType::Gesture,
            EventHandlerId::new(EventOwnerId(1), 0),
        ));
    let mut tree = UiTree::new(root);
    let node = tree.node_id_at(0).unwrap();
    let region = HitRegion {
        node,
        bounds: argui_core::Rect::new(Point::default(), argui_core::Size::new(100.0, 100.0)),
        transform: argui_core::Affine2D::IDENTITY,
        clips: argui_paint::ClipChain::default(),
        shape: HitShape::Bounds,
        slop: argui_ui::Sides::default(),
        enabled: true,
        focus_policy: argui_ui::FocusPolicy::None,
        cursor: argui_ui::CursorIcon::Auto,
        gestures,
        window_drag: None,
    };

    let started = tree.pointer_event(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        std::slice::from_ref(&region),
    );
    assert!(started.events.iter().any(|event| matches!(
        event.kind,
        UiEventKind::Gesture(argui_ui::GestureEvent {
            phase: GesturePhase::Started,
            ..
        })
    )));
    let first = tree.pointer_event(
        touch(1, PointerPhase::Moved, 12.0, 0.0, 8),
        std::slice::from_ref(&region),
    );
    let second = tree.pointer_event(
        touch(1, PointerPhase::Moved, 24.0, 0.0, 16),
        std::slice::from_ref(&region),
    );
    assert!(first.events.is_empty() && first.frame_requested);
    assert!(second.events.is_empty() && second.frame_requested);

    let frame = tree.flush_gesture_frame();
    assert_eq!(frame.events.len(), 1);
    assert!(matches!(
        frame.events[0].kind,
        UiEventKind::Gesture(argui_ui::GestureEvent {
            phase: GesturePhase::Changed,
            kind: GestureKind::Pan { delta, total, .. },
            ..
        }) if total == Point::new(24.0, 0.0) && delta == Point::new(24.0, 0.0)
    ));
    assert!(tree.flush_gesture_frame().events.is_empty());
}

#[test]
fn cancelling_an_active_pair_finishes_each_started_gesture_once() {
    let node = target();
    let gestures = GestureSet::EMPTY
        .pinch(PinchGesture::default())
        .rotation(RotationGesture::default());
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, gestures)),
    );
    arena.update(
        touch(2, PointerPhase::Pressed, 10.0, 0.0, 0),
        Some((node, gestures)),
    );
    let changed = arena.update(touch(2, PointerPhase::Moved, 12.0, 1.0, 10), None);
    assert!(changed.iter().any(|event| matches!(
        event.kind,
        GestureKind::Pinch { .. } | GestureKind::Rotation { .. }
    )));

    let cancelled = arena.cancel_all();
    assert!(!cancelled.is_empty());
    assert!(
        cancelled
            .iter()
            .all(|event| event.phase == GesturePhase::Cancelled)
    );
    assert!(arena.cancel_all().is_empty());
}
