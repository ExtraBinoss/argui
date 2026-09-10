use super::*;

#[test]
fn empty_state_action_navigates_to_the_component_library() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::empty");
    assert!(contains_text(&app.render(), "No projects yet"));
    click(&app, "empty-browse");
    assert!(keyed(&app.render(), "project-card").is_some());
}
