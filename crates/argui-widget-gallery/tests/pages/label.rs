use super::*;

#[test]
fn both_labels_focus_editable_fields_and_values_survive_navigation() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::label");
    for (label, field, value) in [
        ("profile-name-label", "name", "Grace Hopper"),
        ("profile-id-label", "workspace-id", "my-workspace"),
    ] {
        click(&app, label);
        dispatch(&app, field, UiEventKind::TextChanged(value.into()));
        let root = app.render();
        let input = keyed(&root, field).unwrap();
        assert_eq!(input.semantic_bindings.labelled_by[0].0, label);
        assert!(input.interaction.as_ref().unwrap().enabled);
        assert!(
            matches!(&input.kind, argui::ui::ElementKind::TextEditor { value: actual, .. } if actual == value)
        );
    }
    assert!(UiTree::new(app.render()).semantic_diagnostics().is_empty());
    click(&app, "nav::input");
    assert!(matches!(&keyed(&app.render(), "name").unwrap().kind,
        argui::ui::ElementKind::TextEditor { value, .. } if value == "Grace Hopper"));
    click(&app, "nav::label");
    assert!(
        matches!(&keyed(&app.render(), "workspace-id").unwrap().kind,
        argui::ui::ElementKind::TextEditor { value, .. } if value == "my-workspace")
    );
}
