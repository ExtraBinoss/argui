use argui::runtime::Entity;
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
fn button_page_is_present_in_the_public_gallery_tree() {
    let gallery = Entity::new(WidgetGallery::default());
    assert!(contains_text(&gallery.render(), "Actions with variants"));
}
