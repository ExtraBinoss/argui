use super::*;

#[test]
fn project_files_open_close_and_keep_their_state_across_navigation() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::collapsible");
    assert!(!contains_text(&app.render(), "components/button.rs"));
    click(&app, "project-files::trigger");
    assert!(contains_text(&app.render(), "components/button.rs"));
    click(&app, "nav::badge");
    click(&app, "nav::collapsible");
    assert!(contains_text(&app.render(), "components/button.rs"));
    click(&app, "archived-files::trigger");
    assert!(!contains_text(&app.render(), "Archive contents"));
    click(&app, "project-files::trigger");
    assert!(!contains_text(&app.render(), "components/button.rs"));
    assert!(!app.read(argui::runtime::Render::wants_animation_frame));
}
