use super::*;

#[test]
fn layout_reaches_only_the_child_presentation_of_the_target_devtools_mount() {
    use argui_core::ColorScheme;
    use argui_runtime::WindowEnvironment;
    use std::cell::RefCell;
    struct LayoutApp(Rc<RefCell<Vec<(ColorScheme, Rect)>>>);
    impl Render for LayoutApp {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            Element::text("layout probe")
        }
        fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
            self.0
                .borrow_mut()
                .push((cx.environment().color_scheme, layout.viewport));
        }
    }
    let deliveries = Rc::new(RefCell::new(Vec::new()));
    let model = Entity::new(DevtoolsHost::new(LayoutApp(deliveries.clone())));
    let dark = model.mount().unwrap();
    let light = model.mount().unwrap();
    dark.render(WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..Default::default()
    })
    .unwrap();
    light.render(WindowEnvironment::default()).unwrap();
    let tree = UiTree::new(Element::container([]));
    let bounds = Rect::new(Point::new(10.0, 20.0), Size::new(400.0, 300.0));
    let snapshot = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(800.0, 600.0)),
        nodes: vec![LayoutBounds {
            node: tree.node_id_at(0).unwrap(),
            key: Some("__devtools-app-root".into()),
            retained_identity: None,
            bounds,
        }],
    };
    dark.layout_changed(&snapshot).unwrap();
    assert_eq!(*deliveries.borrow(), vec![(ColorScheme::Dark, bounds)]);
    light.layout_changed(&snapshot).unwrap();
    assert_eq!(
        *deliveries.borrow(),
        vec![(ColorScheme::Dark, bounds), (ColorScheme::Light, bounds)]
    );
    dark.close();
    assert!(dark.layout_changed(&snapshot).is_err());
    assert_eq!(deliveries.borrow().len(), 2);
}

pub(super) fn key_input(key: argui_core::Key) -> UiEventKind {
    UiEventKind::KeyInput(argui_core::KeyInput {
        key,
        state: argui_core::KeyState::Pressed,
        modifiers: Default::default(),
        repeat: false,
        text: None,
    })
}

pub(super) fn click_count(count: u8) -> UiEventKind {
    UiEventKind::Click(argui_ui::ClickEvent::pointer(
        PointerEvent::mouse(PointerPhase::Released, Point::default()),
        count,
    ))
}

#[test]
fn picker_surface_handles_stable_motion_leave_empty_click_and_unrelated_events() {
    let host = Entity::new(populated_host()).mount().unwrap();
    change_tools(&host, |tools| {
        tools.inspect_layout(&LayoutSnapshot {
            viewport: Rect::new(Point::default(), Size::new(900.0, 700.0)),
            nodes: vec![],
        })
    });
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-picker",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    let moved = event(
        "__devtools-picker-surface",
        pointer(PointerPhase::Moved, Point::new(40.0, 40.0)),
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&moved)),
        ViewUpdate::Paint
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&moved)),
        ViewUpdate::None
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-picker-surface",
            pointer(PointerPhase::Left, Point::default()),
        ))),
        ViewUpdate::Paint
    );
    assert_eq!(
        change_tools(&host, |tools| tools
            .update(&event("__devtools-picker-surface", UiEventKind::Focused,))),
        ViewUpdate::None
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-picker-surface",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))),
        ViewUpdate::Rebuild
    );
    assert_eq!(host.read(|tools| tools.inspector()).selected(), None);
}

#[test]
fn malformed_and_unselected_style_edits_are_safe_noops() {
    let host = Entity::new(populated_host()).mount().unwrap();
    for key in [
        "__devtools-value",
        "__devtools-value-2",
        "__devtools-value-2-no-such-property-0",
        "__devtools-value-2-background-nope",
    ] {
        assert_eq!(
            change_tools(&host, |tools| tools
                .update(&event(key, UiEventKind::TextChanged("4".into())))),
            ViewUpdate::None
        );
    }
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-value-2-background-0",
            UiEventKind::TextChanged("4".into()),
        ))),
        ViewUpdate::None
    );
    host.read(|tools| tools.inspector())
        .select(Some(InspectNodeId(2)));
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-value-2-background-0",
            UiEventKind::TextChanged("not-a-number".into()),
        ))),
        ViewUpdate::Rebuild
    );
}

#[test]
fn every_remaining_click_route_is_deterministic() {
    let host = Entity::new(populated_host()).mount().unwrap();
    for key in [
        "__devtools-elements",
        "__devtools-refresh",
        "__devtools-node-invalid",
    ] {
        assert_eq!(
            change_tools(&host, |tools| tools.update(&event(
                key,
                UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
            ))),
            ViewUpdate::Rebuild
        );
    }
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-style-not-a-property",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))),
        ViewUpdate::None
    );
    assert!(!host.read(|tools| tools.wants_animation_frame()));
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-unknown",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))),
        ViewUpdate::None
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "application-key",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))),
        ViewUpdate::None
    );
}

#[test]
fn docking_select_and_picker_escape_cover_open_close_and_disabled_options() {
    let host = Entity::new(populated_host()).mount().unwrap();
    assert_eq!(
        change_tools(&host, |tools| tools
            .update(&event("__devtools-dock", click_count(1)))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        change_tools(&host, |tools| tools
            .update(&event("__devtools-dock::option::1", click_count(1)))),
        ViewUpdate::Rebuild
    );
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-surface"
    ));
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-splitter",
            key_input(argui_core::Key::End),
        ))),
        ViewUpdate::Rebuild
    );

    assert_eq!(
        change_tools(&host, |tools| tools
            .update(&event("__devtools-dock", click_count(1)))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-dock::list",
            UiEventKind::PointerOutside(PointerEvent::mouse(
                PointerPhase::Moved,
                Point::new(3.0, 3.0),
            )),
        ))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        change_tools(&host, |tools| tools
            .update(&event("__devtools-dock::option::2", click_count(1)))),
        ViewUpdate::None
    );

    change_tools(&host, |tools| {
        tools.update(&event("__devtools-picker", click_count(1)))
    });
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-picker",
            key_input(argui_core::Key::Escape),
        ))),
        ViewUpdate::Rebuild
    );
    assert!(!contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-picker-surface"
    ));
}

#[test]
fn splitter_keyboard_reset_and_gpu_scroll_rebuild_only_on_range_changes() {
    let host = Entity::new(populated_host()).mount().unwrap();
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-splitter",
            key_input(argui_core::Key::End)
        ))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        change_tools(&host, |tools| tools
            .update(&event("__devtools-splitter", click_count(2)))),
        ViewUpdate::Rebuild
    );

    host.read(|tools| tools.inspector()).record_ui(FrameRecord {
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
    change_tools(&host, |tools| {
        tools.update(&event("__devtools-profiling", click_count(1)))
    });
    assert_eq!(
        change_tools(&host, |tools| tools
            .update(&event("__devtools-frame-0", click_count(1)))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        change_tools(&host, |tools| tools
            .update(&event("__devtools-frame-99", click_count(1)))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-gpu-passes",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 1000.0),
                offset: Point::new(0.0, 1000.0),
            },
        ))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-gpu-passes",
            UiEventKind::Scrolled {
                delta: Point::default(),
                offset: Point::new(0.0, 1000.0),
            },
        ))),
        ViewUpdate::None
    );
}

#[test]
fn dock_and_profiler_animation_paths_distinguish_pause_and_unmount() {
    let host = Entity::new(populated_host()).mount().unwrap();
    host.read(|tools| tools.inspector())
        .record_ui(FrameRecord::default());
    change_tools(&host, |tools| {
        tools.update(&event("__devtools-profiling", click_count(1)))
    });
    assert_eq!(
        change_tools(&host, |tools| tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        })),
        ViewUpdate::None
    );
    change_tools(&host, |tools| {
        tools.update(&event("__devtools-frame-0", click_count(1)))
    });
    assert_eq!(
        change_tools(&host, |tools| tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(100),
        })),
        ViewUpdate::None
    );
    change_tools(&host, |tools| {
        tools.update(&event("__devtools-pause", click_count(1)))
    });
    assert!(host.read(|tools| tools.inspector()).paused());
    assert!(!host.read(|tools| tools.wants_animation_frame()));
    assert_eq!(
        change_tools(&host, |tools| tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(100),
        })),
        ViewUpdate::None
    );

    change_tools(&host, |tools| {
        tools.update(&event("__devtools-dock", click_count(1)))
    });
    assert!(host.read(|tools| tools.wants_animation_frame()));
    assert_eq!(
        change_tools(&host, |tools| tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        })),
        ViewUpdate::Paint
    );
    change_tools(&host, |tools| {
        tools.update(&event("__devtools-dock", click_count(1)))
    });
    assert_eq!(
        change_tools(&host, |tools| tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(100),
        })),
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
        Some(argui_ui::ClipboardRequest::Write(trace)) if trace.contains("argui-gpu-trace-v5")
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
                    retained_identity: None,
                    bounds: Rect::new(Point::default(), Size::new(900.0, 180.0)),
                },
                LayoutBounds {
                    node,
                    key: Some("__devtools-tree".into()),
                    retained_identity: None,
                    bounds: Rect::new(Point::default(), Size::new(300.0, 240.0)),
                },
                LayoutBounds {
                    node,
                    key: Some("__devtools-app-root".into()),
                    retained_identity: None,
                    bounds: Rect::new(Point::new(2.0, 3.0), Size::new(800.0, 500.0)),
                },
            ],
        },
    );
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
}
