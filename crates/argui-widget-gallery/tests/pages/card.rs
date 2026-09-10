use super::*;

#[test]
fn card_action_opens_the_project_details_page() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::card");
    assert!(contains_text(&app.render(), "Your workspace"));
    click(&app, "card-details");
    assert!(keyed(&app.render(), "project-files::trigger").is_some());
    assert!(keyed(&app.render(), "project-card").is_none());
}
