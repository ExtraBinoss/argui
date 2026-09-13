use super::*;
use argui::ui::WritingDirection;

#[test]
fn fluent_gallery_switches_locale_plural_fallback_and_direction() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::i18n");

    let english = app.render();
    assert!(contains_text(&english, "Welcome, \u{2068}Ada\u{2069}!"));
    assert!(contains_text(&english, "You have one notification."));
    assert!(contains_text(
        &english,
        "This sentence comes from the English fallback catalog."
    ));
    assert_eq!(
        keyed(&english, "i18n-content").unwrap().direction_scope,
        Some(WritingDirection::Ltr)
    );

    click(&app, "i18n-more");
    click(&app, "i18n-more");
    assert!(contains_text(
        &app.render(),
        "You have \u{2068}3\u{2069} notifications."
    ));

    click(&app, "i18n-french");
    let french = app.render();
    assert!(contains_text(&french, "Bienvenue, \u{2068}Ada\u{2069} !"));
    assert!(contains_text(
        &french,
        "Vous avez \u{2068}3\u{2069} notifications."
    ));
    assert_eq!(
        keyed(&french, "i18n-save")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Enregistrer")
    );
    assert!(contains_text(
        &french,
        "This sentence comes from the English fallback catalog."
    ));

    click(&app, "i18n-arabic");
    let arabic = app.render();
    assert!(contains_text(&arabic, "مرحبًا، \u{2068}Ada\u{2069}!"));
    assert_eq!(
        keyed(&arabic, "i18n-content").unwrap().direction_scope,
        Some(WritingDirection::Rtl)
    );
    assert_eq!(
        keyed(&arabic, "i18n-save")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("حفظ التغييرات")
    );

    click(&app, "i18n-english");
    assert!(contains_text(
        &app.render(),
        "You have \u{2068}3\u{2069} notifications."
    ));
}
