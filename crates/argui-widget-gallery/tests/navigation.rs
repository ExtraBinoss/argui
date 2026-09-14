use argui::{
    runtime::Entity,
    ui::{ClickEvent, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn contains_text(element: &argui::ui::Element, value: &str) -> bool {
    if let argui::ui::ElementKind::Text { content, .. } = &element.kind
        && content.as_str().contains(value)
    {
        return true;
    }
    element
        .children
        .iter()
        .any(|child| contains_text(child, value))
}

#[test]
fn gallery_navigation_exposes_widget_and_example_sections() {
    let gallery = Entity::new(WidgetGallery::default());
    let root = gallery.render();
    assert!(contains_text(&root, "Widgets"));
    assert!(contains_text(&root, "Examples"));
    assert!(contains_text(&root, "Effects"));
    assert!(contains_text(&root, "Button"));
}

fn click_page(gallery: &Entity<WidgetGallery>, key: &str) {
    let mut tree = UiTree::new(gallery.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .expect("every public page has a navigation key");
    for event in tree.event_deliveries(target, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event);
        }
    }
}

#[test]
fn navigation_rebuilds_widget_and_example_pages_from_public_events() {
    let gallery = Entity::new(WidgetGallery::default());

    for (key, heading) in [
        ("nav::calendar", "Calendar"),
        ("nav::date-picker", "Date picker"),
        ("nav::data-table", "Hours"),
        ("nav::toast", "Show notification"),
        ("nav::file-picker", "Choose export destination"),
        ("nav::input", "Input & Search"),
        ("nav::textarea", "Text area"),
        ("nav::checkbox", "Checkbox"),
        ("nav::switch", "Switch"),
        ("nav::radio-group", "Radio group"),
        ("nav::slider", "Slider"),
        ("nav::tabs", "Tabs"),
        ("nav::select", "Select"),
        ("nav::dialog", "Dialog"),
        ("nav::hot-reload", "State-preserving Rust patches"),
        ("nav::i18n", "Live Fluent catalogs"),
        ("nav::layout", "Web layout"),
        ("nav::motion", "Animation laboratory"),
        ("nav::liquid-glass", "Liquid glass"),
        ("nav::scroll-shadow", "Scroll shadow"),
        ("nav::typography", "Typography & selection"),
        ("nav::async-tasks", "Search 10,000 draft titles"),
        ("nav::editing", "Unicode text"),
    ] {
        click_page(&gallery, key);
        assert!(
            contains_text(&gallery.render(), heading),
            "missing {heading}"
        );
    }
}

#[test]
fn effects_pages_are_separate_and_do_not_compress_the_sidebar() {
    use argui::{core::Size, layout::LayoutEngine, text::TextEngine};
    let gallery = Entity::new(WidgetGallery::default());
    for (page, visible, absent) in [
        (
            "nav::liquid-glass",
            "liquid-glass-demo",
            "scroll-effects-demo",
        ),
        (
            "nav::scroll-shadow",
            "scroll-effects-demo",
            "liquid-glass-demo",
        ),
    ] {
        click_page(&gallery, page);
        let mut tree = UiTree::new(gallery.render());
        let keys: Vec<_> = tree
            .node_ids()
            .iter()
            .filter_map(|id| tree.key(*id))
            .collect();
        assert!(keys.contains(&visible));
        assert!(!keys.contains(&absent));
        assert!(!keys.contains(&"nav::effects"));
        for width in [640.0, 800.0, 1220.0, 1800.0] {
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(width, 900.0))
                .unwrap();
            let bounds = |key| {
                output
                    .nodes
                    .iter()
                    .find(|node| tree.key(node.node) == Some(key))
                    .unwrap()
                    .bounds
            };
            let sidebar = bounds("gallery-sidebar");
            let content = bounds("gallery-content-scroll");
            assert_eq!(sidebar.size.width, 260.0);
            assert_eq!(content.origin.x, sidebar.origin.x + sidebar.size.width);
            assert!((content.origin.x + content.size.width - width).abs() < 0.1);
            let demo = bounds(visible);
            assert!(demo.origin.x >= content.origin.x);
            assert!(demo.origin.x + demo.size.width <= width);
        }
    }
    let mut tree = UiTree::new(gallery.render());
    let search = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("gallery-search"))
        .unwrap();
    for event in tree.event_deliveries(search, UiEventKind::TextChanged("effects".into())) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event);
        }
    }
    let tree = UiTree::new(gallery.render());
    let effects: Vec<_> = tree
        .node_ids()
        .iter()
        .filter_map(|node| tree.key(*node))
        .filter(|key| key.starts_with("nav::"))
        .collect();
    assert_eq!(effects, ["nav::liquid-glass", "nav::scroll-shadow"]);
}
