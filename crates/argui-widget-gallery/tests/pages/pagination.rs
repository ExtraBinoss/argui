use super::*;

#[test]
fn pagination_updates_the_content_and_preserves_its_page_between_visits() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::pagination");
    for (key, page) in [
        ("results::next", 2),
        ("results::page::5", 5),
        ("results::page::6", 6),
        ("results::page::12", 12),
        ("results::previous", 11),
        ("results::page::1", 1),
    ] {
        click(&app, key);
        assert!(contains_text(
            &app.render(),
            &format!("Page {page} of 12 · 36 notes")
        ));
    }
    click(&app, "results::next");
    click(&app, "nav::card");
    click(&app, "nav::pagination");
    assert!(contains_text(&app.render(), "Page 2 of 12 · 36 notes"));
    click(&app, "results::page::2");
    assert!(contains_text(&app.render(), "Page 2 of 12 · 36 notes"));
}
