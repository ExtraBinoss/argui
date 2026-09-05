use argui::runtime::Entity;
use argui::ui::{ClickEvent, UiEventKind, UiTree};
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
fn gallery_exposes_a_numeric_property_editor() {
    let gallery = Entity::new(WidgetGallery::default());
    let mut tree = UiTree::new(gallery.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("nav::composition"))
        .unwrap();
    for event in tree.event_deliveries(target, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event);
        }
    }
    let root = gallery.render();
    assert!(contains_text(&root, "Editable property control"));
    assert!(contains_text(&root, "%"));
}
