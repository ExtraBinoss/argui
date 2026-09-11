use super::*;

#[test]
fn visible_labels_are_associated_and_the_display_name_remains_editable() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::label");
    click(&app, "profile-name-label");
    click(&app, "profile-id-label");
    let tree = UiTree::new(app.render());
    assert!(tree.semantic_diagnostics().is_empty());
    dispatch(
        &app,
        "name",
        UiEventKind::TextChanged("Grace Hopper".into()),
    );
    let root = app.render();
    assert_eq!(
        keyed(&root, "name").unwrap().semantic_bindings.labelled_by[0].0,
        "profile-name-label"
    );
    assert!(
        !keyed(&root, "workspace-id")
            .unwrap()
            .interaction
            .as_ref()
            .unwrap()
            .enabled
    );
    click(&app, "nav::input");
    assert!(matches!(&keyed(&app.render(), "name").unwrap().kind,
        argui::ui::ElementKind::TextEditor { value, .. } if value == "Grace Hopper"));
}
