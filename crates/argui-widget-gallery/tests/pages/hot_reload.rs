use super::*;

#[test]
fn hot_reload_demo_keeps_interactive_state_in_its_entity() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::hot-reload");

    assert!(contains_text(
        &app.render(),
        "State-preserving Rust patches"
    ));
    assert!(contains_text(&app.render(), "Preserved counter: 0"));

    click(&app, "hot-reload-increment");
    click(&app, "hot-reload-increment");
    assert!(contains_text(&app.render(), "Preserved counter: 2"));

    click(&app, "hot-reload-reset");
    assert!(contains_text(&app.render(), "Preserved counter: 0"));
}
