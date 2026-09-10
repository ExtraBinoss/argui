use super::*;
use argui::ui::SemanticValue;

#[test]
fn progress_controls_advance_complete_reset_and_switch_modes() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::progress");
    for expected in [70.0, 95.0, 100.0] {
        click(&app, "progress-advance");
        let root = app.render();
        assert!(
            matches!(keyed(&root, "upload-progress").unwrap().semantics.as_ref().unwrap().value,
            Some(SemanticValue::Number { value, .. }) if value == expected)
        );
    }
    assert!(
        !keyed(&app.render(), "progress-advance")
            .unwrap()
            .interaction
            .as_ref()
            .unwrap()
            .enabled
    );
    click(&app, "progress-mode");
    assert!(contains_text(&app.render(), "Preparing your upload…"));
    assert!(
        keyed(&app.render(), "upload-progress")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .value
            .is_none()
    );
    click(&app, "progress-mode");
    assert!(contains_text(&app.render(), "Uploading files · 45%"));
    click(&app, "progress-reset");
    assert!(contains_text(&app.render(), "Uploading files · 0%"));
    click(&app, "nav::badge");
    click(&app, "nav::progress");
    assert!(contains_text(&app.render(), "Uploading files · 0%"));
}
