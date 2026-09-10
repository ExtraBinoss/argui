use super::*;

#[test]
fn selection_toolbar_dismisses_on_outside_pointer_without_consuming_it() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor))
        .mount()
        .unwrap();
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: false,
        },
    );
    let mut tree = UiTree::new(app.render(WindowEnvironment::default()).unwrap());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("argui::selection-menu"))
        .unwrap();
    let events = tree.event_deliveries(
        target,
        UiEventKind::PointerOutside(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Pressed,
            Point::new(0.0, 0.0),
        )),
    );
    for event in &events {
        if event.should_dispatch() {
            app.dispatch_event(event).unwrap();
        }
        assert!(!event.default_prevented());
    }
    host::finish_exit(&app);
    assert!(!has_key(
        &app.render(WindowEnvironment::default()).unwrap(),
        "argui::selection-menu"
    ));
}

pub(super) fn finish_exit(app: &Mount<argui_widgets::SelectionHost<Editor>>) {
    let root = app.render(WindowEnvironment::default()).unwrap();
    let menu = find_key(&root, "argui::selection-menu").unwrap();
    assert!(menu.semantic_hidden);
    assert_eq!(menu.hit_test.pointer_events, argui_ui::PointerEvents::None);
    app.update(|host, cx| {
        host.animation_frame(
            argui_animation::Frame {
                now: argui_animation::Time::ZERO,
                elapsed: argui_animation::Duration::from_millis(100),
            },
            cx,
        )
    })
    .unwrap();
}

#[test]
fn menu_is_readable_on_zero_elapsed_frame_and_finishes_without_pointer_events() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor))
        .mount()
        .unwrap();
    dispatch(
        &app,
        UiEventKind::ContextMenu {
            position: Point::new(60.0, 60.0),
            capabilities: SelectionCapabilities {
                copy: true,
                select_all: true,
                ..Default::default()
            },
        },
    );
    let root = app.render(WindowEnvironment::default()).unwrap();
    let toolbar = find_key(&root, "argui::selection-menu").unwrap();
    assert!(matches!(
        toolbar.layer.as_ref().unwrap().backdrop_filters.as_slice(),
        [argui_paint::Filter::Blur(_)]
    ));
    let tree = UiTree::new(root.clone());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == toolbar.key.as_deref())
        .unwrap();
    // Keep the original tree: bound paint values must advance without reconciliation.
    for millis in [0, 16, 16, 16, 16, 16, 16, 16, 16, 16] {
        app.update(|host, cx| {
            host.animation_frame(
                argui_animation::Frame {
                    now: argui_animation::Time::ZERO,
                    elapsed: argui_animation::Duration::from_millis(millis),
                },
                cx,
            )
        })
        .unwrap();
        assert_eq!(
            tree.resolved_layer(node, toolbar, toolbar.layer.as_ref().unwrap())
                .opacity,
            1.0
        );
    }
    assert!(!app.read(Render::wants_animation_frame));
    assert_eq!(
        tree.resolved_transform(node, toolbar),
        argui_core::Transform2D::IDENTITY
    );
    dispatch_key(
        &app,
        "argui::selection-menu",
        UiEventKind::PointerOutside(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Pressed,
            Point::default(),
        )),
    );
    app.update(|host, cx| {
        host.animation_frame(
            argui_animation::Frame {
                now: argui_animation::Time::ZERO,
                elapsed: argui_animation::Duration::from_millis(100),
            },
            cx,
        )
    })
    .unwrap();
    assert!(!has_key(
        &app.render(WindowEnvironment::default()).unwrap(),
        "argui::selection-menu"
    ));
    assert!(!app.read(Render::wants_animation_frame));
}

#[test]
fn context_menu_survives_passive_selection_refresh_then_closes_on_drag() {
    let app = Entity::new(
        argui_widgets::SelectionHost::new(Editor).backdrop_filter(argui_paint::Filter::Blur(3.0)),
    )
    .mount()
    .unwrap();
    dispatch(
        &app,
        UiEventKind::ContextMenu {
            position: Point::new(20.0, 30.0),
            capabilities: SelectionCapabilities {
                copy: true,
                select_all: true,
                ..Default::default()
            },
        },
    );
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::default()),
            touch: false,
            dragging: false,
        },
    );
    app.update(|host, cx| {
        host.animation_frame(
            argui_animation::Frame {
                now: argui_animation::Time::from_nanos(140_000_000),
                elapsed: argui_animation::Duration::from_millis(140),
            },
            cx,
        )
    })
    .unwrap();
    let root = app.render(WindowEnvironment::default()).unwrap();
    let toolbar = find_key(&root, "argui::selection-menu")
        .expect("passive selection refresh must not dismiss menu");
    let tree = UiTree::new(root.clone());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == toolbar.key.as_deref())
        .unwrap();
    assert_eq!(
        tree.resolved_layer(node, toolbar, toolbar.layer.as_ref().unwrap())
            .opacity,
        1.0
    );
    assert!(!app.read(Render::wants_animation_frame));
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("new selection".into()),
            bounds: Some(Rect::default()),
            touch: false,
            dragging: true,
        },
    );
    app.update(|host, cx| {
        host.animation_frame(
            argui_animation::Frame {
                now: argui_animation::Time::from_nanos(240_000_000),
                elapsed: argui_animation::Duration::from_millis(100),
            },
            cx,
        )
    })
    .unwrap();
    assert!(!has_key(
        &app.render(WindowEnvironment::default()).unwrap(),
        "argui::selection-menu"
    ));
}

#[test]
fn reopen_during_exit_restores_interaction_and_full_opacity() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor))
        .mount()
        .unwrap();
    for _ in 0..3 {
        dispatch(
            &app,
            UiEventKind::ContextMenu {
                position: Point::default(),
                capabilities: SelectionCapabilities {
                    copy: true,
                    ..Default::default()
                },
            },
        );
        app.update(|host, cx| {
            host.animation_frame(
                argui_animation::Frame {
                    now: argui_animation::Time::ZERO,
                    elapsed: argui_animation::Duration::from_millis(30),
                },
                cx,
            )
        })
        .unwrap();
        dispatch_key(
            &app,
            "argui::selection-menu",
            UiEventKind::PointerOutside(argui_core::PointerEvent::mouse(
                argui_core::PointerPhase::Pressed,
                Point::default(),
            )),
        );
    }
    dispatch(
        &app,
        UiEventKind::ContextMenu {
            position: Point::default(),
            capabilities: SelectionCapabilities {
                copy: true,
                ..Default::default()
            },
        },
    );
    let root = app.render(WindowEnvironment::default()).unwrap();
    assert!(
        !find_key(&root, "argui::selection-menu")
            .unwrap()
            .semantic_hidden
    );
    assert!(app.read(Render::wants_animation_frame));
}

#[test]
fn selection_host_reduced_motion_mounts_immediately_and_global_command_uses_no_target() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor))
        .mount()
        .unwrap();
    let environment = WindowEnvironment {
        reduced_motion: true,
        ..WindowEnvironment::default()
    };
    let _ = app.render(environment).unwrap();
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: false,
        },
    );
    let rendered = app.render(environment).unwrap();
    assert!(has_key(&rendered, "argui::selection-menu"));
    assert!(!app.read(Render::wants_animation_frame));

    dispatch_key_in(
        &app,
        "argui::selection-menu::copy",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        environment,
    );
    assert!(!has_key(
        &app.render(environment).unwrap(),
        "argui::selection-menu"
    ));
}

#[test]
fn selection_host_prevents_pointer_default_for_command_press_and_keeps_menu_open() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor))
        .mount()
        .unwrap();
    dispatch(
        &app,
        UiEventKind::ContextMenu {
            position: Point::new(20.0, 24.0),
            capabilities: SelectionCapabilities {
                copy: true,
                ..SelectionCapabilities::default()
            },
        },
    );
    let mut tree = UiTree::new(app.render(WindowEnvironment::default()).unwrap());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("argui::selection-menu::copy"))
        .unwrap();
    let events = tree.event_deliveries(
        target,
        UiEventKind::Pointer(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Pressed,
            Point::new(22.0, 25.0),
        )),
    );
    assert!(!events.is_empty());
    for event in &events {
        if event.should_dispatch() {
            app.dispatch_event(event).unwrap();
        }
    }
    assert!(events.iter().all(argui_ui::UiEvent::default_prevented));
    assert!(has_key(
        &app.render(WindowEnvironment::default()).unwrap(),
        "argui::selection-menu::copy"
    ));
}

struct ContextTarget;
impl Render for ContextTarget {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        Element::column([Element::text("Target").keyed("context-target")]).on(cx.listener(
            argui_ui::EventType::ContextMenu,
            |_, event, _| {
                assert!(event.prevent_default());
            },
        ))
    }
}

#[test]
fn handled_context_menu_does_not_also_open_the_selection_toolbar() {
    let app = Entity::new(argui_widgets::SelectionHost::new(ContextTarget))
        .mount()
        .unwrap();
    dispatch_key(
        &app,
        "context-target",
        UiEventKind::ContextMenu {
            position: Point::new(20.0, 24.0),
            capabilities: SelectionCapabilities {
                copy: true,
                select_all: true,
                ..Default::default()
            },
        },
    );
    assert!(!has_key(
        &app.render(WindowEnvironment::default()).unwrap(),
        "argui::selection-menu"
    ));
}
