use super::*;

/// Creates one mouse or primary touch sample for a press lifecycle.
///
/// * `pointer` — identifier of the mouse or touch contact.
/// * `kind` — pointer device kind.
/// * `phase` — transition delivered to the tree.
///
/// Returns a sample inside the shared test hit region.
fn contact(pointer: PointerId, kind: PointerKind, phase: PointerPhase) -> PointerEvent {
    let point = Point::new(30.0, 20.0);
    PointerEvent {
        id: pointer,
        kind,
        button: Some(PointerButton::Primary),
        buttons: u16::from(phase == PointerPhase::Pressed),
        ..PointerEvent::mouse(phase, point)
    }
}

#[test]
fn accepted_and_prevented_presses_control_mouse_and_touch_capture() {
    for (kind, pointer) in [
        (PointerKind::Mouse, PointerId::MOUSE),
        (PointerKind::Touch, PointerId::new(71)),
    ] {
        let element = listeners(
            Element::container([]).interaction(Interaction::default().capture_on_press(true)),
        );
        let mut tree = UiTree::new(element);
        let node = tree.node_id_at(0).unwrap();
        let regions = [region(node)];

        let accepted = tree.pointer_event(contact(pointer, kind, PointerPhase::Pressed), &regions);
        let down = accepted.events.iter().find(|delivery| {
            matches!(
                delivery.kind,
                UiEventKind::Pointer(PointerEvent {
                    phase: PointerPhase::Pressed,
                    ..
                })
            )
        });
        assert!(!tree.pointer_captured(pointer));
        let capture = tree.pointer_press_default(pointer, down, &regions);
        assert!(
            capture
                .events
                .iter()
                .any(|delivery| { delivery.kind == UiEventKind::GotPointerCapture(pointer) })
        );
        assert!(tree.pointer_captured(pointer));
        let released = tree.pointer_event(contact(pointer, kind, PointerPhase::Released), &regions);
        assert!(
            released
                .events
                .iter()
                .any(|delivery| { delivery.kind == UiEventKind::LostPointerCapture(pointer) })
        );

        let prevented = tree.pointer_event(contact(pointer, kind, PointerPhase::Pressed), &regions);
        let down = prevented.events.iter().find(|delivery| {
            matches!(
                delivery.kind,
                UiEventKind::Pointer(PointerEvent {
                    phase: PointerPhase::Pressed,
                    ..
                })
            )
        });
        let down = down.expect("interactive target delivers PointerDown");
        assert!(down.prevent_default());
        assert!(
            tree.pointer_press_default(pointer, Some(down), &regions)
                .is_empty()
        );
        assert!(!tree.pointer_captured(pointer));
        tree.pointer_event(contact(pointer, kind, PointerPhase::Released), &regions);
    }
}

#[test]
fn capture_default_works_without_pointer_down_listeners() {
    let element = Element::container([]).interaction(Interaction::default().capture_on_press(true));
    let mut tree = UiTree::new(element);
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];
    tree.pointer_event(
        contact(PointerId::MOUSE, PointerKind::Mouse, PointerPhase::Pressed),
        &regions,
    );

    let capture = tree.pointer_press_default(PointerId::MOUSE, None, &regions);

    assert!(capture.events.is_empty());
    assert!(tree.pointer_captured(PointerId::MOUSE));
}

#[test]
fn secondary_button_delivers_pointer_phases_without_primary_click() {
    let mut tree = UiTree::new(listeners(
        Element::container([]).interaction(Interaction::default()),
    ));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];
    for phase in [PointerPhase::Pressed, PointerPhase::Released] {
        let event = PointerEvent {
            button: Some(PointerButton::Secondary),
            buttons: u16::from(phase == PointerPhase::Pressed) * 2,
            ..PointerEvent::mouse(phase, Point::new(30.0, 20.0))
        };
        let update = tree.pointer_event(event, &regions);
        assert!(update.events.iter().any(|delivery| {
            matches!(&delivery.kind, UiEventKind::Pointer(sample)
                if sample.phase == phase && sample.button == Some(PointerButton::Secondary))
        }));
        assert!(
            !update
                .events
                .iter()
                .any(|delivery| matches!(delivery.kind, UiEventKind::Click(_)))
        );
    }
    assert!(!tree.pointer_captured(PointerId::MOUSE));
    let mut disabled = [region(node)];
    disabled[0].enabled = false;
    let down = PointerEvent {
        button: Some(PointerButton::Secondary),
        buttons: 2,
        ..PointerEvent::mouse(PointerPhase::Pressed, Point::new(30.0, 20.0))
    };
    assert!(
        !tree
            .pointer_event(down, &disabled)
            .events
            .iter()
            .any(|delivery| {
                matches!(
                    delivery.kind,
                    UiEventKind::Pointer(PointerEvent {
                        phase: PointerPhase::Pressed,
                        ..
                    })
                )
            })
    );
}
