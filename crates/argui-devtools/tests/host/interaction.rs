use super::*;

fn key_input(key: argui_core::Key) -> UiEventKind {
    UiEventKind::KeyInput(argui_core::KeyInput {
        key,
        state: argui_core::KeyState::Pressed,
        modifiers: Default::default(),
        repeat: false,
        text: None,
    })
}

fn click_count(count: u8) -> UiEventKind {
    UiEventKind::Click(argui_ui::ClickEvent::pointer(
        PointerEvent::mouse(PointerPhase::Released, Point::default()),
        count,
    ))
}

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

#[test]
fn tree_actions_cover_collapse_expand_parent_and_sibling_navigation() {
    let mut host = populated_host();
    assert_eq!(
        host.update(&event("__devtools-node-1", click_count(2))),
        ViewUpdate::Rebuild
    );
    assert!(!contains_text(&host.view(), "button  #save"));

    assert_eq!(
        host.update(&event(
            "__devtools-node-1",
            key_input(argui_core::Key::ArrowRight)
        )),
        ViewUpdate::Rebuild
    );
    assert!(contains_text(&host.view(), "button  #save"));
    assert_eq!(
        host.update(&event(
            "__devtools-node-1",
            key_input(argui_core::Key::ArrowRight)
        )),
        ViewUpdate::Rebuild
    );
    assert_eq!(host.inspector().selected(), Some(InspectNodeId(2)));
    assert!(matches!(
        host.take_focus_request(),
        Some(argui_ui::FocusRequest::Focus(argui_ui::FocusTarget::Key(key)))
            if key == "__devtools-node-2"
    ));
    assert_eq!(
        host.update(&event(
            "__devtools-node-2",
            key_input(argui_core::Key::ArrowLeft)
        )),
        ViewUpdate::Rebuild
    );
    assert_eq!(host.inspector().selected(), Some(InspectNodeId(1)));
    for navigation in [argui_core::Key::ArrowDown, argui_core::Key::End] {
        assert_eq!(
            host.update(&event("__devtools-node-1", key_input(navigation))),
            ViewUpdate::Rebuild
        );
        assert_eq!(host.inspector().selected(), Some(InspectNodeId(2)));
    }
    assert_eq!(
        host.update(&event(
            "__devtools-node-2",
            key_input(argui_core::Key::Home)
        )),
        ViewUpdate::Rebuild
    );
    assert_eq!(host.inspector().selected(), Some(InspectNodeId(1)));
}

#[test]
fn docking_select_and_picker_escape_cover_open_close_and_disabled_options() {
    let mut host = populated_host();
    assert_eq!(
        host.update(&event("__devtools-dock", click_count(1))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.update(&event("__devtools-dock::option::1", click_count(1))),
        ViewUpdate::Rebuild
    );
    assert!(contains_key(&host.view(), "__devtools-surface"));
    assert_eq!(
        host.update(&event(
            "__devtools-splitter",
            key_input(argui_core::Key::End),
        )),
        ViewUpdate::Rebuild
    );

    assert_eq!(
        host.update(&event("__devtools-dock", click_count(1))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.update(&event(
            "__devtools-dock::list",
            UiEventKind::PointerOutside(PointerEvent::mouse(
                PointerPhase::Moved,
                Point::new(3.0, 3.0),
            )),
        )),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.update(&event("__devtools-dock::option::2", click_count(1))),
        ViewUpdate::None
    );

    host.update(&event("__devtools-picker", click_count(1)));
    assert_eq!(
        host.update(&event(
            "__devtools-picker",
            key_input(argui_core::Key::Escape),
        )),
        ViewUpdate::Rebuild
    );
    assert!(!contains_key(&host.view(), "__devtools-picker-surface"));
}

#[test]
fn splitter_keyboard_reset_and_gpu_scroll_rebuild_only_on_range_changes() {
    let mut host = populated_host();
    assert_eq!(
        host.update(&event(
            "__devtools-splitter",
            key_input(argui_core::Key::End)
        )),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.update(&event("__devtools-splitter", click_count(2))),
        ViewUpdate::Rebuild
    );

    host.inspector().record_ui(FrameRecord {
        gpu: Some(argui_inspect::GpuFrameRecord {
            total: std::time::Duration::from_millis(8),
            passes: (0..40)
                .map(|index| argui_inspect::GpuPassRecord {
                    label: format!("pass-{index}"),
                    duration: std::time::Duration::from_micros(100),
                    ..argui_inspect::GpuPassRecord::default()
                })
                .collect(),
            ..argui_inspect::GpuFrameRecord::default()
        }),
        ..FrameRecord::default()
    });
    host.update(&event("__devtools-profiling", click_count(1)));
    assert_eq!(
        host.update(&event("__devtools-frame-0", click_count(1))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.update(&event("__devtools-frame-99", click_count(1))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.update(&event(
            "__devtools-gpu-passes",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 1000.0),
                offset: Point::new(0.0, 1000.0),
            },
        )),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.update(&event(
            "__devtools-gpu-passes",
            UiEventKind::Scrolled {
                delta: Point::default(),
                offset: Point::new(0.0, 1000.0),
            },
        )),
        ViewUpdate::None
    );
}

#[test]
fn dock_and_profiler_animation_paths_distinguish_pause_and_unmount() {
    let mut host = populated_host();
    host.inspector().record_ui(FrameRecord::default());
    host.update(&event("__devtools-profiling", click_count(1)));
    assert_eq!(
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        }),
        ViewUpdate::None
    );
    host.update(&event("__devtools-frame-0", click_count(1)));
    assert_eq!(
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(100),
        }),
        ViewUpdate::None
    );
    host.update(&event("__devtools-pause", click_count(1)));
    assert!(host.inspector().paused());
    assert!(!host.wants_animation_frame());
    assert_eq!(
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(100),
        }),
        ViewUpdate::None
    );

    host.update(&event("__devtools-dock", click_count(1)));
    assert!(host.wants_animation_frame());
    assert_eq!(
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        }),
        ViewUpdate::Paint
    );
    host.update(&event("__devtools-dock", click_count(1)));
    assert_eq!(
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(100),
        }),
        ViewUpdate::Rebuild
    );
}

fn dispatch_model_event(
    model: &mut argui_runtime::SingleWindowModel<DevtoolsHost<App>>,
    tree: &mut UiTree,
    key: &str,
    kind: UiEventKind,
) {
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .expect("event target must exist");
    for delivery in tree.event_deliveries(target, kind) {
        if delivery.should_dispatch() {
            argui_runtime::AppModel::update(
                model,
                &argui_runtime::AppEvent::Ui {
                    window: argui_platform::WindowKey::main(),
                    event: delivery,
                },
            );
        }
    }
}

#[test]
fn render_listener_forwards_clipboard_focus_and_scroll_effects() {
    let mut model = argui_runtime::SingleWindowModel::new(populated_host());
    let window = argui_platform::WindowKey::main();
    let mut tree = UiTree::new(
        argui_runtime::AppModel::view(&model, &window, argui_runtime::WindowEnvironment::default())
            .expect("single window view"),
    );

    dispatch_model_event(
        &mut model,
        &mut tree,
        "__devtools-profiling",
        click_count(1),
    );
    tree.update(
        argui_runtime::AppModel::view(&model, &window, argui_runtime::WindowEnvironment::default())
            .unwrap(),
    );
    dispatch_model_event(&mut model, &mut tree, "__devtools-copy", click_count(1));
    assert!(matches!(
        argui_runtime::AppModel::take_clipboard_request(&mut model, &window),
        Some(argui_ui::ClipboardRequest::Write(trace)) if trace.contains("argui-gpu-trace-v2")
    ));

    dispatch_model_event(&mut model, &mut tree, "__devtools-elements", click_count(1));
    tree.update(
        argui_runtime::AppModel::view(&model, &window, argui_runtime::WindowEnvironment::default())
            .unwrap(),
    );
    dispatch_model_event(
        &mut model,
        &mut tree,
        "__devtools-node-1",
        key_input(argui_core::Key::ArrowRight),
    );
    assert!(matches!(
        argui_runtime::AppModel::take_focus_request(&mut model, &window),
        Some(argui_ui::FocusRequest::Focus(argui_ui::FocusTarget::Key(key)))
            if key == "__devtools-node-2"
    ));
    assert!(matches!(
        argui_runtime::AppModel::take_scroll_request(&mut model, &window),
        Some(argui_ui::ScrollRequest {
            target: argui_ui::ScrollTarget::Offset { .. },
            ..
        })
    ));

    let reduced = argui_runtime::AppModel::view(
        &model,
        &window,
        argui_runtime::WindowEnvironment {
            reduced_motion: true,
            ..argui_runtime::WindowEnvironment::default()
        },
    )
    .unwrap();
    assert!(contains_key(&reduced, "__devtools-surface"));
    let node = UiTree::new(Element::container([])).node_id_at(0).unwrap();
    let update = argui_runtime::AppModel::layout_changed(
        &mut model,
        &window,
        &LayoutSnapshot {
            viewport: Rect::new(Point::default(), Size::new(900.0, 700.0)),
            nodes: vec![
                LayoutBounds {
                    node,
                    key: Some("__devtools-frames".into()),
                    bounds: Rect::new(Point::default(), Size::new(900.0, 180.0)),
                },
                LayoutBounds {
                    node,
                    key: Some("__devtools-tree".into()),
                    bounds: Rect::new(Point::default(), Size::new(300.0, 240.0)),
                },
                LayoutBounds {
                    node,
                    key: Some("__devtools-app-root".into()),
                    bounds: Rect::new(Point::new(2.0, 3.0), Size::new(800.0, 500.0)),
                },
            ],
        },
    );
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
}
