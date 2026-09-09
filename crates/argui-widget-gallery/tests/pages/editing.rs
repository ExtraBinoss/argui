use super::*;
use argui::ui::TextPrivacy;
#[test]
fn editing_page_updates_drafts_reveals_passwords_and_resets_external_values() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::editing");
    dispatch(&app, "editing-0", UiEventKind::Focused);
    dispatch(
        &app,
        "editing-0",
        UiEventKind::TextChanged("Changed".into()),
    );
    click(&app, "editing-replace");
    click(&app, "editing-reset");
    assert!(matches!(&keyed(&app.render(), "editing-0").unwrap().kind,
        argui::ui::ElementKind::TextEditor { value, .. } if value == "External reset 1"));
    click(&app, "editing-reveal");
    let root = app.render();
    assert_eq!(
        keyed(&root, "editing-4").unwrap().text_privacy,
        TextPrivacy::RevealedPassword
    );
    click(&app, "editing-reveal");
    assert!(contains_text(&app.render(), "Show password"));
}
