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
