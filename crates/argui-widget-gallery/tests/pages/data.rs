use argui::{
    core::{Key, KeyInput, KeyState, Modifiers, Point, Size},
    layout::LayoutEngine,
    runtime::Entity,
    text::TextEngine,
    ui::{ClickEvent, Element, Role, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn dispatch(app: &Entity<WidgetGallery>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}
fn click(app: &Entity<WidgetGallery>, key: &str) {
    dispatch(app, key, UiEventKind::Click(ClickEvent::accessibility()));
}
fn find<'a>(root: &'a Element, key: &str) -> Option<&'a Element> {
    if root.key.as_deref() == Some(key) {
        Some(root)
    } else {
        root.children.iter().find_map(|child| find(child, key))
    }
}

#[test]
fn data_pages_select_navigate_and_virtualize_after_layout_measurement() {
    let app = Entity::new(WidgetGallery::default());
    for (page, role) in [
        ("list", Role::ListBox),
        ("vlist", Role::ListBox),
        ("table", Role::Grid),
    ] {
        click(&app, &format!("nav::{page}"));
        let root = app.render();
        assert_eq!(
            find(&root, "data")
                .unwrap()
                .semantics
                .as_ref()
                .unwrap()
                .role,
            role
        );
        pointer_click(&app, "data::row::1");
        assert!(
            find(&app.render(), "data::row::1")
                .unwrap()
                .semantics
                .as_ref()
                .unwrap()
                .state
                .selected
        );
        dispatch(
            &app,
            "data::row::1",
            UiEventKind::KeyInput(KeyInput {
                key: Key::End,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            }),
        );
        let last = if page == "vlist" { 9999 } else { 7 };
        let root = app.render();
        assert!(
            find(&root, &format!("data::row::{last}"))
                .unwrap()
                .semantics
                .as_ref()
                .unwrap()
                .state
                .selected
        );
        let mut tree = UiTree::new(root);
        let layout = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(900.0, 700.0))
            .unwrap();
        assert!(layout.viewport.size.width.is_finite());
        if page == "vlist" {
            assert!(find(tree.root(), "data::row::0").is_none());
            dispatch(
                &app,
                "data",
                UiEventKind::Scrolled {
                    delta: Point::default(),
                    offset: Point::new(0.0, 400.0),
                },
            );
            assert!(find(&app.render(), "data::row::10").is_some());
        }
    }
    let root = app.render();
    for removed in [
        "shared-mailbox",
        "draft-studio",
        "form",
        "settings",
        "composition",
    ] {
        assert!(find(&root, &format!("nav::{removed}")).is_none());
    }
    assert!(find(&root, "nav::webview").is_some());
}

fn pointer_click(app: &Entity<WidgetGallery>, key: &str) {
    use argui::core::{PointerButton, PointerEvent, PointerPhase};
    let mut tree = UiTree::new(app.render());
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut TextEngine::new(), Size::new(900.0, 700.0))
        .unwrap();
    let region = output
        .hit_regions
        .iter()
        .find(|region| tree.key(region.node) == Some(key))
        .unwrap();
    let point = Point::new(
        region.bounds.origin.x + region.bounds.size.width * 0.5,
        region.bounds.origin.y + region.bounds.size.height * 0.5,
    );
    for (phase, buttons) in [(PointerPhase::Pressed, 1), (PointerPhase::Released, 0)] {
        let update = tree.pointer_event(
            PointerEvent {
                button: Some(PointerButton::Primary),
                buttons,
                ..PointerEvent::mouse(phase, point)
            },
            &output.hit_regions,
        );
        for event in update.events {
            if event.should_dispatch() {
                app.dispatch_event(&event);
            }
        }
    }
}

#[test]
fn scrolling_reuses_mounted_rows_until_the_virtual_window_changes() {
    let app = Entity::new(WidgetGallery::default());
    for (page, key) in [("vlist", "data"), ("data-table", "grid::rows")] {
        click(&app, &format!("nav::{page}"));
        let scroll = |offset| {
            dispatch(
                &app,
                key,
                UiEventKind::Scrolled {
                    delta: Point::default(),
                    offset: Point::new(0.0, offset),
                },
            )
        };
        let before = app.render();
        scroll(1.0);
        let within = app.render();
        assert!(
            find(&before, key)
                .unwrap()
                .ptr_eq(find(&within, key).unwrap())
        );
        scroll(800.0);
        let next = app.render();
        assert!(
            !find(&within, key)
                .unwrap()
                .ptr_eq(find(&next, key).unwrap())
        );

        // Measurements can change spacers even while the visible indices stay the same.
        let mut ui = UiTree::new(next);
        LayoutEngine::new()
            .compute(&mut ui, &mut TextEngine::new(), Size::new(900.0, 700.0))
            .unwrap();
        scroll(800.0);
        let measured = app.render();
        scroll(800.0);
        assert!(
            find(&measured, key)
                .unwrap()
                .ptr_eq(find(&app.render(), key).unwrap())
        );
    }
    click(&app, "nav::table");
    click(&app, "nav::vlist");
    assert!(find(&app.render(), "data::row::0").is_some());
}

#[test]
#[ignore = "Manual CPU profile: run with --cargo-profile release --run-ignored ignored-only"]
fn profile_virtual_scrolling() {
    use argui::ui::TreeUpdate;
    use web_time::Instant;
    const FONT: &[u8] = include_bytes!("../../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
    let frames = std::env::var("ARGUI_PROFILE_FRAMES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(120);
    let app = Entity::new(WidgetGallery::default());
    let mut text = TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
    for (page, key) in [
        ("vlist", "data"),
        ("table", "data"),
        ("data-table", "grid::rows"),
    ] {
        let started = Instant::now();
        click(&app, &format!("nav::{page}"));
        let mut ui = UiTree::new(app.render());
        let mut engine = LayoutEngine::new();
        let viewport = Size::new(1280.0, 900.0);
        let mut layout = engine.compute(&mut ui, &mut text, viewport).unwrap();
        let opened = started.elapsed();
        let started = Instant::now();
        let mut layouts = 0;
        for step in 0..frames {
            let target = ui
                .node_ids()
                .iter()
                .copied()
                .find(|node| ui.key(*node) == Some(key))
                .unwrap();
            let offset = Point::new(0.0, step as f32 * 4.0);
            ui.set_scroll_offset(target, offset);
            for event in ui.event_deliveries(
                target,
                UiEventKind::Scrolled {
                    delta: Point::new(0.0, 4.0),
                    offset,
                },
            ) {
                if event.should_dispatch() {
                    app.dispatch_event(&event);
                }
            }
            match ui.update(app.render()) {
                TreeUpdate::Layout => {
                    layout = engine.compute(&mut ui, &mut text, viewport).unwrap();
                    layouts += 1;
                }
                _ => engine.apply_scroll(&ui, &mut layout).unwrap(),
            }
            ui.pointer_moved(Point::new(380.0, 330.0), &layout.hit_regions);
        }
        eprintln!(
            "{page}: open={:.2}ms, {frames} scroll frames={:.2}ms, layouts={layouts}, nodes={}",
            opened.as_secs_f64() * 1000.0,
            started.elapsed().as_secs_f64() * 1000.0,
            ui.node_ids().len()
        );
        assert!(ui.node_ids().len() < 1000);
    }
}

#[test]
fn data_page_text_is_painted_on_the_first_layout_after_navigation() {
    use argui::paint::DisplayCommand;
    let app = Entity::new(WidgetGallery::default());
    let mut ui = UiTree::new(app.render());
    ui.set_reduced_motion(true);
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    for (page, size) in [
        ("table", Size::new(900.0, 700.0)),
        ("data-table", Size::new(900.0, 700.0)),
        ("vlist", Size::new(900.0, 700.0)),
        ("table", Size::new(600.0, 450.0)),
        ("data-table", Size::new(600.0, 450.0)),
        ("table", Size::new(1100.0, 900.0)),
    ] {
        click(&app, &format!("nav::{page}"));
        ui.update(app.render());
        let warm = engine.compute(&mut ui, &mut text, size).unwrap();
        let cold = LayoutEngine::new()
            .compute(&mut ui, &mut text, size)
            .unwrap();
        let commands = |layout: &argui::layout::LayoutOutput| {
            layout
                .display_list
                .commands()
                .iter()
                .filter(|c| matches!(c, DisplayCommand::Text { .. }))
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            commands(&warm),
            commands(&cold),
            "{page}: retained text paint differs from a fresh layout"
        );
        assert_eq!(warm.text, cold.text, "{page}: stale text geometry");
    }
}
