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

fn scroll_to(app: &Entity<PerfShowcase>, key: &str, delta_y: f32, offset_y: f32) {
    dispatch(
        app,
        key,
        UiEventKind::Scrolled {
            delta: Point::new(0.0, delta_y),
            offset: Point::new(0.0, offset_y),
        },
    );
}

fn dispatch_retyped_scroll(app: &Entity<PerfShowcase>, key: &str) {
    let mut tree = UiTree::new(app.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    let mut delivery = tree
        .event_deliveries(
            target,
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 500.0),
                offset: Point::new(0.0, 500.0),
            },
        )
        .into_iter()
        .find(|event| event.current_handler().is_some())
        .unwrap();
    // UiEvent.kind is public, so callers can mutate a valid listener delivery.
    delivery.kind = UiEventKind::Focused;
    app.dispatch_event(&delivery);
}

fn has_text(element: &argui::ui::Element, value: &str) -> bool {
    if let argui::ui::ElementKind::Text { content, .. } = &element.kind
        && content.as_str().contains(value)
    {
        return true;
    }
    element.children.iter().any(|child| has_text(child, value))
}

fn text_with_prefix<'a>(element: &'a argui::ui::Element, prefix: &str) -> Option<&'a str> {
    if let argui::ui::ElementKind::Text { content, .. } = &element.kind
        && content.as_str().starts_with(prefix)
    {
        return Some(content.as_str());
    }
    element
        .children
        .iter()
        .find_map(|child| text_with_prefix(child, prefix))
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

fn row_keys(root: &argui::ui::Element, list_key: &str, row_prefix: &str) -> Vec<String> {
    fn collect(element: &argui::ui::Element, prefix: &str, keys: &mut Vec<String>) {
        if let Some(key) = element.key.as_deref()
            && key.starts_with(prefix)
        {
            keys.push(key.to_owned());
        }
        for child in &element.children {
            collect(child, prefix, keys);
        }
    }

    let mut keys = Vec::new();
    collect(find_key(root, list_key).unwrap(), row_prefix, &mut keys);
    keys.sort();
    keys
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

#[test]
fn list_scrolls_only_rebuild_when_the_visible_window_changes() {
    for (key, prefix, sibling, sibling_prefix) in [
        (
            "perf-million-fixed",
            "perf-row-",
            "perf-million-variable",
            "perf-variable-row-",
        ),
        (
            "perf-million-variable",
            "perf-variable-row-",
            "perf-million-fixed",
            "perf-row-",
        ),
    ] {
        let app = Entity::new(PerfShowcase::default());
        let initial = app.render();
        let rows_at_start = row_keys(&initial, key, prefix);
        let sibling_rows = row_keys(&initial, sibling, sibling_prefix);
        assert!(!rows_at_start.is_empty());
        scroll_to(&app, key, 20_000.0, 20_000.0);
        let moved = app.render();
        let rows_at_distance = row_keys(&moved, key, prefix);
        assert_ne!(rows_at_start, rows_at_distance);
        assert_eq!(sibling_rows, row_keys(&moved, sibling, sibling_prefix));
        let hud = text_with_prefix(&moved, "root renders=").unwrap();
        for (delta, offset) in [(0.0, 20_000.0), (1.0, 20_001.0)] {
            scroll_to(&app, key, delta, offset);
            let repeated = app.render();
            assert_eq!(rows_at_distance, row_keys(&repeated, key, prefix));
            assert_eq!(
                hud,
                text_with_prefix(&repeated, "root renders=").unwrap(),
                "scrolling within the same virtual window must keep the cached tree"
            );
        }
    }
}

#[test]
fn scroll_listeners_ignore_malformed_public_deliveries_without_invalidation() {
    let app = Entity::new(PerfShowcase::default());
    let before = app.render();
    let fixed_before = row_keys(&before, "perf-million-fixed", "perf-row-");
    let variable_before = row_keys(&before, "perf-million-variable", "perf-variable-row-");
    let hud_before = text_with_prefix(&before, "root renders=")
        .unwrap()
        .to_owned();

    dispatch_retyped_scroll(&app, "perf-million-fixed");
    dispatch_retyped_scroll(&app, "perf-million-variable");

    let after = app.render();
    assert_eq!(
        fixed_before,
        row_keys(&after, "perf-million-fixed", "perf-row-")
    );
    assert_eq!(
        variable_before,
        row_keys(&after, "perf-million-variable", "perf-variable-row-")
    );
    assert_eq!(
        hud_before,
        text_with_prefix(&after, "root renders=").unwrap(),
        "ignoring a malformed listener delivery must not invalidate the tree"
    );
    assert!(has_text(&after, "value=0"));
}
