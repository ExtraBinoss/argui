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
        ("nav::input", "Input & Search"),
        ("nav::textarea", "Text area"),
        ("nav::checkbox", "Checkbox"),
        ("nav::switch", "Switch"),
        ("nav::radio-group", "Radio group"),
        ("nav::slider", "Slider"),
        ("nav::tabs", "Tabs"),
        ("nav::select", "Select"),
        ("nav::dialog", "Dialog"),
        ("nav::layout", "Web layout"),
        ("nav::motion", "Motion & loading"),
        ("nav::effects", "GPU effects / WGSL"),
        ("nav::typography", "Typography & selection"),
        ("nav::async-tasks", "Search 10,000 draft titles"),
        ("nav::actions", "Left scope"),
        ("nav::editing", "Unicode text"),
    ] {
        click_page(&gallery, key);
        assert!(
            contains_text(&gallery.render(), heading),
            "missing {heading}"
        );
    }
}
