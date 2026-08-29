use std::time::Duration;

use argui_core::{Point, PointerButton, PointerEvent, PointerId, PointerKind, PointerPhase};
use argui_ui::{Element, GestureArena, GestureKind, GesturePhase, GestureSet, UiTree};

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
        timestamp: Duration::from_millis(millis),
    }
}

#[test]
fn short_stationary_contacts_resolve_as_taps() {
    let node = target();
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 10.0, 20.0, 0),
        Some((node, GestureSet::NONE.tap())),
    );
    let events = arena.update(touch(1, PointerPhase::Released, 12.0, 21.0, 120), None);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].phase, GesturePhase::Ended);
    assert!(matches!(events[0].kind, GestureKind::Tap { .. }));
}

#[test]
fn pans_start_after_the_threshold_and_report_velocity() {
    let node = target();
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::NONE.pan())),
    );
    assert!(
        arena
            .update(touch(1, PointerPhase::Moved, 4.0, 0.0, 10), None)
            .is_empty()
    );
    let started = arena.update(touch(1, PointerPhase::Moved, 12.0, 0.0, 20), None);
    let ended = arena.update(touch(1, PointerPhase::Released, 20.0, 0.0, 30), None);

    assert_eq!(started[0].phase, GesturePhase::Started);
    let GestureKind::Pan {
        total, velocity, ..
    } = started[0].kind
    else {
        panic!("expected pan");
    };
    assert_eq!(total, Point::new(12.0, 0.0));
    assert!(velocity.x > 0.0);
    assert_eq!(ended[0].phase, GesturePhase::Ended);
}

#[test]
fn two_contacts_can_recognize_pinch_and_rotation_together() {
    let node = target();
    let gestures = GestureSet::NONE.pinch().rotation();
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
        Some((node, GestureSet::NONE.tap())),
    );
    assert!(
        arena
            .update(touch(1, PointerPhase::Cancelled, 0.0, 0.0, 20), None)
            .is_empty()
    );
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::NONE.tap())),
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
fn active_pans_change_cancel_and_keep_bounded_velocity_history() {
    let node = target();
    let mut arena = GestureArena::default();
    arena.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::NONE.pan())),
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
    let cancelled = arena.update(touch(1, PointerPhase::Left, 17.0, 0.0, 340), None);
    assert_eq!(cancelled[0].phase, GesturePhase::Cancelled);
}

#[test]
fn paired_gestures_emit_changed_and_ended_phases() {
    let node = target();
    let gestures = GestureSet::ALL;
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
    let gestures = GestureSet::NONE.rotation();
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
        Some((node, GestureSet::NONE.pan())),
    );
    cancelled_pan.update(touch(1, PointerPhase::Moved, 9.0, 0.0, 1), None);
    assert_eq!(cancelled_pan.cancel_all()[0].phase, GesturePhase::Cancelled);

    let mut no_pan = GestureArena::default();
    no_pan.update(
        touch(1, PointerPhase::Pressed, 0.0, 0.0, 0),
        Some((node, GestureSet::NONE.tap())),
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
        Some((node, GestureSet::NONE.pan())),
    );
    assert!(
        no_pan
            .update(touch(2, PointerPhase::Released, 1.0, 0.0, 10), None)
            .is_empty()
    );

    let mut pair = GestureArena::default();
    let pair_gestures = GestureSet::NONE.pinch();
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
        Some((other, GestureSet::NONE.pan())),
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
        Some((node, GestureSet::NONE.pan())),
    );
    for step in 1..=8 {
        dense_history.update(
            touch(1, PointerPhase::Moved, step as f32 * 9.0, 0.0, step),
            None,
        );
    }
}
