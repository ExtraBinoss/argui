use super::*;
use argui::{
    core::Key,
    ui::{CheckedState, Role},
};

#[test]
fn menu_pages_share_keyboard_navigation_indicators_and_controlled_selection() {
    for (page, key) in [
        ("menu", "menu"),
        ("context-menu", "menu"),
        ("menubar", "second-menu"),
    ] {
        let app = Entity::new(WidgetGallery::default());
        click(&app, &format!("nav::{page}"));
        keyboard(
            &app,
            key,
            if page == "context-menu" {
                Key::ContextMenu
            } else {
                Key::ArrowDown
            },
        );
        let check = format!("{key}::item::check");
        let root = app.render();
        assert_eq!(
            keyed(&root, &check)
                .unwrap()
                .semantics
                .as_ref()
                .unwrap()
                .state
                .checked,
            Some(CheckedState::Checked)
        );
        assert!(matches!(
            keyed(&root, &check).unwrap().children[0].kind,
            argui::ui::ElementKind::Vector { .. }
        ));
        click(&app, &check);
        let root = app.render();
        assert_eq!(
            keyed(&root, &check)
                .unwrap()
                .semantics
                .as_ref()
                .unwrap()
                .state
                .checked,
            Some(CheckedState::Unchecked)
        );
        assert!(!contains_text(&root, "Design review · Draft"));
        keyboard(&app, &check, Key::Character("l".into()));
        keyboard(&app, &format!("{key}::item::options"), Key::ArrowRight);
        let radio = format!("{key}::item::second");
        click(&app, &radio);
        let root = app.render();
        let row = keyed(&root, &radio).unwrap();
        assert_eq!(row.semantics.as_ref().unwrap().role, Role::MenuItemRadio);
        assert!(matches!(
            row.children[0].kind,
            argui::ui::ElementKind::Vector { .. }
        ));
        keyboard(&app, &radio, Key::Escape);
        assert!(keyed(&app.render(), &radio).is_none());
        keyboard(&app, &check, Key::Tab);
        assert!(keyed(&app.render(), &check).is_none());
    }
}

#[test]
fn keyboard_dismissal_cancels_a_pending_hover_and_hover_opens_when_due() {
    use argui::{
        core::{Point, PointerEvent, PointerPhase},
        runtime::tasks::TaskRuntime,
    };
    use std::{sync::mpsc, time::Duration};
    let app = Entity::new(WidgetGallery::default());
    let (sender, wake) = mpsc::channel();
    let runtime = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    app.set_task_runtime(runtime.clone());
    let drain = || {
        while runtime.pending() > 0 {
            runtime.drain();
            if runtime.pending() > 0 {
                wake.recv_timeout(Duration::from_secs(5)).unwrap();
            }
        }
    };
    click(&app, "nav::menu");
    click(&app, "menu");
    dispatch(
        &app,
        "menu::item::options",
        UiEventKind::Pointer(PointerEvent::mouse(PointerPhase::Entered, Point::default())),
    );
    keyboard(&app, "menu::item::options", Key::Escape);
    drain();
    assert!(keyed(&app.render(), "menu::item::first").is_none());
    click(&app, "menu");
    dispatch(
        &app,
        "menu::item::options",
        UiEventKind::Pointer(PointerEvent::mouse(PointerPhase::Entered, Point::default())),
    );
    drain();
    assert!(keyed(&app.render(), "menu::item::first").is_some());
    click(&app, "nav::menubar");
    keyboard(&app, "first-menu", Key::ArrowRight);
    assert!(keyed(&app.render(), "second-menu::item::check").is_none());
    keyboard(&app, "second-menu", Key::ArrowDown);
    assert!(keyed(&app.render(), "second-menu::item::check").is_some());
}

#[test]
fn submenu_geometry_keeps_diagonal_hover_open_and_cancels_on_entry() {
    use argui::{
        core::{Point, PointerEvent, PointerPhase, Rect, Size},
        runtime::{LayoutBounds, LayoutSnapshot, tasks::TaskRuntime},
    };
    use std::{sync::mpsc, time::Duration};
    let entity = Entity::new(WidgetGallery::default());
    let (sender, wake) = mpsc::channel();
    let runtime = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    entity.set_task_runtime(runtime.clone());
    let app = entity.mount().unwrap();
    let dispatch = |key: &str, kind| {
        let mut tree = UiTree::new(app.render(Default::default()).unwrap());
        let id = tree
            .node_ids()
            .iter()
            .copied()
            .find(|id| tree.key(*id) == Some(key))
            .unwrap();
        for event in tree.event_deliveries(id, kind) {
            if event.should_dispatch() {
                app.dispatch_event(&event).unwrap();
            }
        }
    };
    let click = |key: &str| {
        dispatch(
            key,
            UiEventKind::Click(argui::ui::ClickEvent::accessibility()),
        )
    };
    click("nav::menu");
    click("menu");
    click("menu::item::options");
    let mut tree = UiTree::new(app.render(Default::default()).unwrap());
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut TextEngine::new(), Size::new(1200.0, 900.0))
        .unwrap();
    let snapshot = LayoutSnapshot {
        viewport: output.viewport,
        nodes: output
            .nodes
            .iter()
            .map(|node| LayoutBounds {
                node: node.node,
                key: tree.key(node.node).map(str::to_owned),
                retained_identity: None,
                bounds: node.bounds,
            })
            .collect(),
    };
    app.layout_changed(&snapshot).unwrap();
    let Rect { origin, size } = snapshot.bounds("menu::item::options::content").unwrap();
    let start = Point::new(origin.x - 40.0, origin.y + size.height * 0.5);
    dispatch(
        "menu::item::check",
        UiEventKind::Pointer(PointerEvent::mouse(PointerPhase::Entered, start)),
    );
    dispatch(
        "menu::item::check",
        UiEventKind::Pointer(PointerEvent::mouse(
            PointerPhase::Moved,
            Point::new(origin.x - 10.0, start.y),
        )),
    );
    dispatch(
        "menu::item::first",
        UiEventKind::Pointer(PointerEvent::mouse(
            PointerPhase::Moved,
            Point::new(origin.x + 10.0, start.y),
        )),
    );
    while runtime.pending() > 0 {
        runtime.drain();
        if runtime.pending() > 0 {
            wake.recv_timeout(Duration::from_secs(5)).unwrap();
        }
    }
    assert!(
        keyed(
            &app.render(Default::default()).unwrap(),
            "menu::item::first"
        )
        .is_some()
    );
}

#[test]
fn file_and_view_menus_have_distinct_actions_that_update_the_notebook() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::menubar");
    assert!(contains_text(&app.render(), "File"));
    assert!(contains_text(&app.render(), "View"));
    for (action, message, count) in [
        ("new", "Created note 2.", 2),
        ("duplicate", "Duplicated note 2 into note 3.", 3),
        ("reset", "Notebook reset to one note.", 1),
    ] {
        click(&app, "first-menu");
        assert!(keyed(&app.render(), "first-menu::item::check").is_none());
        let mut tree = UiTree::new(app.render());
        let key = format!("first-menu::item::{action}");
        let node = *tree
            .node_ids()
            .iter()
            .find(|node| tree.key(**node) == Some(&key))
            .unwrap();
        for event in tree.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility())) {
            if !event.should_dispatch() {
                continue;
            }
            if let UiEventKind::Action(invocation) = event.kind {
                for delivery in tree.invoke_action(invocation).events {
                    if delivery.should_dispatch() {
                        app.dispatch_event(&delivery);
                    }
                }
            } else {
                app.dispatch_event(&event);
            }
        }
        let root = app.render();
        assert!(contains_text(&root, message));
        assert_eq!(
            keyed(&root, "notebook-preview").unwrap().children.len(),
            count
        );
    }
    keyboard(&app, "first-menu", Key::ArrowDown);
    keyboard(&app, "first-menu", Key::ArrowRight);
    assert!(keyed(&app.render(), "second-menu::item::check").is_some());
    assert!(keyed(&app.render(), "second-menu::item::new").is_none());
}
