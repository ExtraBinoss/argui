use super::*;

#[test]
fn loading_shapes_unmount_when_content_is_ready_and_can_be_shown_again() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::skeleton");
    let root = app.render();
    assert!(keyed(&root, "skeleton-cover").is_some());
    assert!(
        keyed(&root, "skeleton-card")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .state
            .busy
    );
    click(&app, "skeleton-toggle");
    let root = app.render();
    assert!(keyed(&root, "skeleton-cover").is_none());
    assert!(
        !keyed(&root, "skeleton-card")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .state
            .busy
    );
    assert!(contains_text(&root, "Designing something new"));
    click(&app, "nav::card");
    click(&app, "nav::skeleton");
    assert!(keyed(&app.render(), "skeleton-cover").is_none());
    click(&app, "skeleton-toggle");
    assert!(keyed(&app.render(), "skeleton-cover").is_some());
}
