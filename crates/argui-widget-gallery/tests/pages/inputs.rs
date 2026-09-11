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
fn input_page_is_present_in_the_public_gallery_tree() {
    let gallery = Entity::new(WidgetGallery::default());
    assert!(contains_text(&gallery.render(), "Input"));
    assert!(contains_text(&gallery.render(), "Text area"));
}

#[test]
fn search_and_email_validation_follow_edits_and_survive_navigation() {
    use super::{click, dispatch, keyed};
    use argui::ui::{ElementKind, UiEventKind};
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "nav::input");
    dispatch(
        &gallery,
        "input-search-demo",
        UiEventKind::TextChanged("shadows".into()),
    );
    for (email, invalid) in [
        ("missing-at", true),
        ("@example.com", true),
        ("person@localhost", true),
        ("person@example.com", false),
    ] {
        dispatch(&gallery, "invalid", UiEventKind::TextChanged(email.into()));
        assert_eq!(
            keyed(&gallery.render(), "invalid")
                .unwrap()
                .semantics
                .as_ref()
                .unwrap()
                .state
                .invalid,
            invalid
        );
    }
    click(&gallery, "nav::label");
    click(&gallery, "nav::input");
    assert!(
        matches!(&keyed(&gallery.render(), "input-search-demo").unwrap().kind, ElementKind::TextEditor { value, .. } if value == "shadows")
    );
    assert!(
        matches!(&keyed(&gallery.render(), "invalid").unwrap().kind, ElementKind::TextEditor { value, .. } if value == "person@example.com")
    );
}
