use super::*;

#[test]
fn panel_carousel_and_chart_examples_complete_their_actions() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::sheet");
    click(&app, "settings::trigger");
    assert!(keyed(&app.render(), "settings::panel").is_some());
    click(&app, "settings::close");
    click(&app, "nav::alert-dialog");
    click(&app, "confirm::trigger");
    click(&app, "confirm::backdrop");
    assert!(keyed(&app.render(), "confirm::panel").is_some());
    click(&app, "confirm::close");
    click(&app, "confirm::trigger");
    click(&app, "confirm::confirm");
    assert!(contains_text(&app.render(), "Project archived."));
    click(&app, "nav::drawer");
    click(&app, "details::trigger");
    assert!(keyed(&app.render(), "details::panel").is_some());
    click(&app, "details::close");
    click(&app, "nav::carousel");
    click(&app, "collection::next");
    assert!(contains_text(&app.render(), "Autumn"));
    keyboard(&app, "collection::viewport", Key::End);
    assert!(contains_text(&app.render(), "Winter"));
    click(&app, "collection::previous");
    click(&app, "nav::chart");
    click(&app, "sales::point::0::1");
    assert!(contains_text(&app.render(), "Studio · Q2: 32"));
    click(&app, "chart-line");
    click(&app, "nav::hover-card");
    click(&app, "profile");
    assert!(keyed(&app.render(), "profile::content").is_some());
    click(&app, "profile-follow");
    assert!(contains_text(&app.render(), "You are now following Ada."));
    keyboard(&app, "profile-follow", Key::Escape);
    assert!(keyed(&app.render(), "profile::content").is_none());
}
