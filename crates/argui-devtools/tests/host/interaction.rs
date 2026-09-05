use super::*;

#[test]
fn picker_surface_handles_stable_motion_leave_empty_click_and_unrelated_events() {
    let mut host = populated_host();
    host.layout_changed(&LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(900.0, 700.0)),
        nodes: vec![],
    });
    host.update(&event(
        "__devtools-picker",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    ));
    let moved = event(
        "__devtools-picker-surface",
        pointer(PointerPhase::Moved, Point::new(40.0, 40.0)),
    );
    assert_eq!(host.update(&moved), ViewUpdate::Paint);
    assert_eq!(host.update(&moved), ViewUpdate::None);
    assert_eq!(
        host.update(&event(
            "__devtools-picker-surface",
            pointer(PointerPhase::Left, Point::default()),
        )),
        ViewUpdate::Paint
    );
    assert_eq!(
        host.update(&event("__devtools-picker-surface", UiEventKind::Focused,)),
        ViewUpdate::None
    );
    assert_eq!(
        host.update(&event(
            "__devtools-picker-surface",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )),
        ViewUpdate::Rebuild
    );
    assert_eq!(host.inspector().selected(), None);
}

#[test]
fn malformed_and_unselected_style_edits_are_safe_noops() {
    let mut host = populated_host();
    for key in [
        "__devtools-value",
        "__devtools-value-2",
        "__devtools-value-2-no-such-property-0",
        "__devtools-value-2-background-nope",
    ] {
        assert_eq!(
            host.update(&event(key, UiEventKind::TextChanged("4".into()))),
            ViewUpdate::None
        );
    }
    assert_eq!(
        host.update(&event(
            "__devtools-value-2-background-0",
            UiEventKind::TextChanged("4".into()),
        )),
        ViewUpdate::None
    );
    host.inspector().select(Some(InspectNodeId(2)));
    assert_eq!(
        host.update(&event(
            "__devtools-value-2-background-0",
            UiEventKind::TextChanged("not-a-number".into()),
        )),
        ViewUpdate::None
    );
}

#[test]
fn every_remaining_click_route_is_deterministic() {
    let mut host = populated_host();
    for key in [
        "__devtools-elements",
        "__devtools-refresh",
        "__devtools-node-invalid",
    ] {
        assert_eq!(
            host.update(&event(
                key,
                UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
            )),
            ViewUpdate::Rebuild
        );
    }
    assert_eq!(
        host.update(&event(
            "__devtools-style-not-a-property",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )),
        ViewUpdate::None
    );
    assert!(!host.wants_animation_frame());
    assert_eq!(
        host.update(&event(
            "__devtools-unknown",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )),
        ViewUpdate::None
    );
    assert_eq!(
        host.update(&event(
            "application-key",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )),
        ViewUpdate::None
    );
}

#[test]
fn large_virtual_scrolls_rebuild_only_when_the_visible_window_changes() {
    let mut host = populated_host();
    let mut snapshot = host.inspector().tree();
    snapshot.revision += 1;
    let template = snapshot.nodes[1].clone();
    for index in 3..600 {
        let mut node = template.clone();
        node.id = InspectNodeId(index);
        node.key = Some(format!("node-{index}"));
        snapshot.nodes.push(node);
    }
    host.inspector().publish_tree(snapshot);
    assert_eq!(
        host.update(&event(
            "__devtools-tree",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 1_500.0),
                offset: Point::new(0.0, 1_500.0),
            },
        )),
        ViewUpdate::Rebuild
    );

    for _ in 0..100 {
        host.inspector().record_ui(FrameRecord::default());
    }
    assert_eq!(
        host.update(&event(
            "__devtools-refresh",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.update(&event(
            "__devtools-frames",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 1_500.0),
                offset: Point::new(0.0, 1_500.0),
            },
        )),
        ViewUpdate::Rebuild
    );
}
