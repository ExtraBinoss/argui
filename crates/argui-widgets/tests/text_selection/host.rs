use super::*;

pub(super) fn finish_exit(app: &Entity<argui_widgets::SelectionHost<Editor>>) {
    let root = app.render();
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
    });
}

#[test]
fn menu_is_readable_on_zero_elapsed_frame_and_finishes_without_pointer_events() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
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
    let root = app.render();
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
        });
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
    });
    assert!(!has_key(&app.render(), "argui::selection-menu"));
    assert!(!app.read(Render::wants_animation_frame));
}

#[test]
fn context_menu_survives_passive_selection_refresh_then_closes_on_drag() {
    let app = Entity::new(
        argui_widgets::SelectionHost::new(Editor).backdrop_filter(argui_paint::Filter::Blur(3.0)),
    );
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
    });
    let root = app.render();
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
    });
    assert!(!has_key(&app.render(), "argui::selection-menu"));
}

#[test]
fn reopen_during_exit_restores_interaction_and_full_opacity() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
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
        });
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
    let root = app.render();
    assert!(
        !find_key(&root, "argui::selection-menu")
            .unwrap()
            .semantic_hidden
    );
    assert!(app.read(Render::wants_animation_frame));
}
