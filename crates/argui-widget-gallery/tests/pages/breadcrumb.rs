use super::*;

#[test]
fn breadcrumb_destinations_open_the_corresponding_gallery_pages() {
    let app = Entity::new(WidgetGallery::default());
    for (key, heading) in [
        ("component-path::link::home", "Button"),
        ("component-path::link::components", "Card"),
        ("project-path::link::projects", "Empty"),
        ("project-path::link::library", "Card"),
    ] {
        click(&app, "nav::breadcrumb");
        click(&app, key);
        assert!(contains_text(find_content(&app.render()).unwrap(), heading));
    }
}
