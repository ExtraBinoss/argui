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
        focusable: false,
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
