use super::*;
use argui::core::Key;
#[test]
fn calendar_and_date_picker_pages_handle_keyboard_validation_and_escape() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::calendar");
    let tree = UiTree::new(app.render());
    let day = tree
        .node_ids()
        .iter()
        .find_map(|id| {
            tree.key(*id)
                .filter(|key| key.starts_with("calendar::day::"))
        })
        .unwrap()
        .to_string();
    keyboard(&app, &day, Key::PageDown);
    click(&app, "nav::date-picker");
    dispatch(&app, "date::input", UiEventKind::Submitted("bad".into()));
    assert!(keyed(&app.render(), "date::error").is_some());
    dispatch(
        &app,
        "date::input",
        UiEventKind::Submitted("2026-09-09".into()),
    );
    assert!(keyed(&app.render(), "date::error").is_none());
    click(&app, "date::popup");
    assert!(keyed(&app.render(), "date::popup::content").is_some());
    keyboard(&app, "date::popup", Key::Escape);
    assert!(keyed(&app.render(), "date::popup::content").is_none());
}

#[test]
fn unrelated_keys_leave_calendar_and_date_drafts_unchanged() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::calendar");
    let before = UiTree::new(app.render()).semantic_tree(&[], 1.0);
    keyboard(&app, "calendar", Key::Other);
    let after = UiTree::new(app.render()).semantic_tree(&[], 1.0);
    assert_eq!(
        before
            .nodes
            .iter()
            .map(|node| &node.semantics.label)
            .collect::<Vec<_>>(),
        after
            .nodes
            .iter()
            .map(|node| &node.semantics.label)
            .collect::<Vec<_>>()
    );
    click(&app, "nav::date-picker");
    dispatch(
        &app,
        "date::input",
        UiEventKind::TextChanged("unfinished".into()),
    );
    keyboard(&app, "date::input", Key::Other);
    assert!(
        matches!(&keyed(&app.render(), "date::input").unwrap().kind, argui::ui::ElementKind::TextEditor { value, .. } if value == "unfinished")
    );
}
