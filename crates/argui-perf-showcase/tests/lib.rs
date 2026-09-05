use argui::core::Point;
use argui::runtime::Entity;
use argui::ui::{ClickEvent, UiEventKind, UiTree};
use argui_perf_showcase::PerfShowcase;

fn has_key(element: &argui::ui::Element, key: &str) -> bool {
    element.key.as_deref() == Some(key) || element.children.iter().any(|child| has_key(child, key))
}

fn dispatch(app: &Entity<PerfShowcase>, key: &str, kind: UiEventKind) {
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

fn has_text(element: &argui::ui::Element, value: &str) -> bool {
    if let argui::ui::ElementKind::Text { content, .. } = &element.kind
        && content.as_str().contains(value)
    {
        return true;
    }
    element.children.iter().any(|child| has_text(child, value))
}

fn find_key<'a>(element: &'a argui::ui::Element, key: &str) -> Option<&'a argui::ui::Element> {
    if element.key.as_deref() == Some(key) {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| find_key(child, key))
}

#[test]
fn performance_showcase_keeps_both_million_row_labs_in_the_public_tree() {
    let app = Entity::new(PerfShowcase::default());
    let root = app.render();
    assert!(has_key(&root, "perf-million-fixed"));
    assert!(has_key(&root, "perf-million-variable"));
    assert!(has_key(&root, "perf-counter"));
}

#[test]
fn public_events_update_the_counter_and_keep_virtual_rows_bounded() {
    let app = Entity::new(PerfShowcase::default());
    let _ = app.render();
    dispatch(
        &app,
        "perf-counter",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    dispatch(
        &app,
        "perf-million-fixed",
        UiEventKind::Scrolled {
            delta: Point::new(0.0, 20_000.0),
            offset: Point::new(0.0, 20_000.0),
        },
    );
    let root = app.render();
    assert!(has_text(&root, "value=1"));
    assert!(has_key(&root, "perf-million-variable"));
    let fixed = find_key(&root, "perf-million-fixed").unwrap();
    assert!(fixed.children.len() < 64);
}
