use super::*;

#[test]
fn settings_control_click_count_time_and_distance() {
    let mut tree = UiTree::new(interactive("target"));
    tree.set_pointer_settings(
        argui_core::PointerSettings::default()
            .multi_click(std::time::Duration::from_millis(100), 2.0),
    );
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];
    let click = |tree: &mut UiTree, point: Point, millis: u64| {
        let event = |phase| PointerEvent {
            button: Some(PointerButton::Primary),
            timestamp: std::time::Duration::from_millis(millis),
            ..PointerEvent::mouse(phase, point)
        };
        tree.pointer_event(event(PointerPhase::Pressed), &regions);
        tree.pointer_event(event(PointerPhase::Released), &regions)
            .events
            .into_iter()
            .find_map(|event| match event.kind {
                UiEventKind::Click(click) => Some(click.count),
                _ => None,
            })
            .unwrap()
    };

    assert_eq!(click(&mut tree, Point::new(30.0, 20.0), 0), 1);
    assert_eq!(click(&mut tree, Point::new(31.0, 20.0), 50), 2);
    assert_eq!(click(&mut tree, Point::new(35.0, 20.0), 80), 1);
    assert_eq!(click(&mut tree, Point::new(35.0, 20.0), 250), 1);
}

#[test]
fn public_routing_reports_enter_and_leave_for_hit_regions() {
    let mut tree = UiTree::new(listeners(Element::container([])));
    let node = tree.node_id_at(0).unwrap();
    let region = HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(20.0, 20.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: HitTestStyle::default().slop,
        enabled: true,
        focus_policy: argui_ui::FocusPolicy::None,
        cursor: CursorIcon::Auto,
        gestures: argui_ui::GestureSet::EMPTY,
        window_drag: None,
    };

    let entered = tree.pointer_moved(Point::new(5.0, 5.0), std::slice::from_ref(&region));
    assert!(entered.events.iter().any(|event| {
        matches!(
            &event.kind,
            UiEventKind::Pointer(PointerEvent {
                phase: PointerPhase::Entered,
                ..
            })
        )
    }));
    let left = tree.pointer_moved(Point::new(30.0, 30.0), &[region]);
    assert!(left.events.iter().any(|event| {
        matches!(
            &event.kind,
            UiEventKind::Pointer(PointerEvent {
                phase: PointerPhase::Left,
                ..
            })
        )
    }));
}

#[test]
fn captured_cursor_ignores_missing_or_disabled_regions() {
    let mut tree = UiTree::new(interactive("cursor"));
    let node = tree.node_id_at(0).unwrap();
    let mut region = region(node);
    region.cursor = CursorIcon::Grab;

    assert_eq!(
        tree.captured_cursor(PointerId::MOUSE, &[region.clone()]),
        None
    );
    tree.capture_pointer(PointerId::MOUSE, node);
    assert_eq!(
        tree.captured_cursor(PointerId::MOUSE, &[region.clone()]),
        Some(CursorIcon::Grab)
    );
    region.enabled = false;
    assert_eq!(tree.captured_cursor(PointerId::MOUSE, &[region]), None);
}

#[test]
fn replacing_capture_releases_the_previous_target_and_same_capture_is_idempotent() {
    let mut tree = UiTree::new(Element::row([interactive("first"), interactive("second")]));
    let first = tree.node_id_at(1).unwrap();
    let second = tree.node_id_at(2).unwrap();
    let pointer = PointerId::new(19);

    assert_eq!(tree.capture_pointer(pointer, first).events.len(), 1);
    assert!(tree.capture_pointer(pointer, first).events.is_empty());
    let replaced = tree.capture_pointer(pointer, second);
    assert_eq!(replaced.events.len(), 2);
    assert_eq!(
        replaced.events[0].kind,
        UiEventKind::LostPointerCapture(pointer)
    );
    assert_eq!(
        replaced.events[1].kind,
        UiEventKind::GotPointerCapture(pointer)
    );
}

#[test]
fn secondary_touches_do_not_replace_primary_interaction_capture() {
    let mut tree = UiTree::new(interactive("touch"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];
    let touch = |id, phase, primary| PointerEvent {
        id: PointerId::new(id),
        kind: PointerKind::Touch,
        phase,
        position: Point::new(30.0, 20.0),
        button: Some(PointerButton::Primary),
        buttons: 1,
        pressure: None,
        primary,
        modifiers: argui_core::Modifiers::default(),
        timestamp: std::time::Duration::ZERO,
    };

    let pressed = tree.pointer_event(touch(1, PointerPhase::Pressed, true), &regions);
    assert!(pressed.events.iter().any(|event| matches!(
        event.kind,
        UiEventKind::Pointer(PointerEvent {
            phase: PointerPhase::Pressed,
            ..
        })
    )));
    assert!(
        tree.pointer_event(touch(2, PointerPhase::Pressed, false), &regions)
            .events
            .is_empty()
    );
    assert!(tree.visual_states(node).contains(VisualState::Hovered));
    assert!(tree.visual_states(node).contains(VisualState::Pressed));
    let cancelled = tree.pointer_event(touch(1, PointerPhase::Cancelled, true), &regions);
    assert!(cancelled.events.iter().any(|event| matches!(
        event.kind,
        UiEventKind::Pointer(PointerEvent {
            phase: PointerPhase::Cancelled,
            ..
        })
    )));
    assert!(
        cancelled
            .events
            .iter()
            .all(|event| event.kind != UiEventKind::Click(argui_ui::ClickEvent::accessibility()))
    );
    assert_eq!(tree.visual_states(node), VisualStates::NONE);
    assert!(
        tree.pointer_event(touch(1, PointerPhase::Cancelled, true), &regions)
            .events
            .is_empty()
    );

    tree.pointer_event(touch(3, PointerPhase::Pressed, true), &regions);
    let released = tree.pointer_event(touch(3, PointerPhase::Released, true), &regions);
    assert!(
        released
            .events
            .iter()
            .any(|event| matches!(event.kind, UiEventKind::Click(_)))
    );
    assert!(released.events.iter().any(|event| matches!(
        event.kind,
        UiEventKind::Pointer(PointerEvent {
            phase: PointerPhase::Left,
            kind: PointerKind::Touch,
            ..
        })
    )));
    assert_eq!(tree.visual_states(node), VisualStates::NONE);
}
