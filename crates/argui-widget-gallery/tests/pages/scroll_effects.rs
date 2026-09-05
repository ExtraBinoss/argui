use argui::{
    core::Point,
    runtime::Entity,
    ui::{ClickEvent, Element, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn find<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    if element.key.as_deref() == Some(key) {
        Some(element)
    } else {
        element.children.iter().find_map(|child| find(child, key))
    }
}

fn dispatch(app: &Entity<WidgetGallery>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(node, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

#[test]
fn scroll_demo_cycles_presets_parameters_and_virtual_windows() {
    let app = Entity::new(WidgetGallery::default());
    let click = || UiEventKind::Click(ClickEvent::accessibility());
    dispatch(&app, "nav::effects", click());
    for expected in [
        "argui.scroll.edge-fade",
        "argui.scroll.edge-shadow",
        "gallery.scroll.progress-tint",
    ] {
        let root = app.render();
        let list = find(&root, "scroll-demo-list").unwrap();
        assert!(list.children.len() < 40);
        let argui::paint::Filter::Effect(effect) =
            &list.scroll.as_ref().unwrap().effects[0].layer.filters[0]
        else {
            panic!()
        };
        assert_eq!(effect.id.0, expected);
        dispatch(&app, "scroll-mode", click());
    }
    for _ in 0..7 {
        dispatch(&app, "scroll-width", click());
    }
    for _ in 0..5 {
        dispatch(&app, "scroll-intensity", click());
    }
    for offset in [1.0, 2.0, 2400.0, 319_700.0, 0.0] {
        dispatch(
            &app,
            "scroll-demo-list",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 1.0),
                offset: Point::new(0.0, offset),
            },
        );
        assert!(
            find(&app.render(), "scroll-demo-list")
                .unwrap()
                .children
                .len()
                < 40
        );
    }
    assert!(find(&app.render(), "scroll-demo-nested").is_some());
    assert!(find(&app.render(), "scroll-horizontal").is_some());
}
