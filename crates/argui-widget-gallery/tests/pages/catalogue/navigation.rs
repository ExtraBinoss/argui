use super::*;

#[test]
fn collection_examples_toggle_and_navigate_using_public_widget_actions() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::accordion");
    click(&app, "faq::item::keyboard::trigger");
    assert!(contains_text(
        &app.render(),
        "Use Tab to reach a heading, Enter to expand it, and arrows to move between headings."
    ));
    keyboard(&app, "faq::item::keyboard::trigger", Key::ArrowDown);
    click(&app, "faq::item::keyboard::trigger");
    click(&app, "nav::toggle");
    click(&app, "bold");
    assert!(contains_text(&app.render(), "Bold enabled"));
    click(&app, "bold");
    assert!(contains_text(&app.render(), "Bold disabled"));
    click(&app, "nav::toggle-group");
    click(&app, "format::item::bold");
    keyboard(&app, "format::item::bold", Key::ArrowRight);
    let root = app.render();
    assert_eq!(
        keyed(&root, "format::item::bold")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .state
            .pressed,
        Some(true)
    );
    click(&app, "nav::button-group");
    click(&app, "group-copy");
    assert!(contains_text(&app.render(), "Action: copy"));
    click(&app, "nav::navigation-menu");
    click(&app, "docs::item::overview");
    assert!(contains_text(&app.render(), "Opened overview"));
    keyboard(&app, "docs::item::overview", Key::ArrowRight);
    keyboard(&app, "docs::item::guide", Key::ArrowDown);
    assert!(keyed(&app.render(), "docs::item::guide::content").is_some());
    keyboard(&app, "docs::item::guide", Key::Escape);
    assert!(keyed(&app.render(), "docs::item::guide::content").is_none());
    click(&app, "docs::item::guide");
    click(&app, "guide-start");
    assert!(contains_text(
        &app.render(),
        "Getting started: create a window and add a Button."
    ));
    click(&app, "nav::sidebar");
    click(&app, "workspace::toggle");
    assert!(contains_text(&app.render(), "P"));
    click(&app, "workspace-Projects");
    assert!(contains_text(&app.render(), "Opened Projects"));
    click(&app, "workspace::toggle");
    assert!(contains_text(&app.render(), "Activity"));
}
