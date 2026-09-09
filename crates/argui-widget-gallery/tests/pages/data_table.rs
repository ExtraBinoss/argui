use super::*;
use argui::core::{Key, Point};
#[test]
fn grid_demo_edits_validates_sorts_and_keeps_active_rows_mounted() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::data-table");
    click(&app, "grid::select-page");
    click(&app, "grid::cell::1:0value");
    keyboard(&app, "grid", Key::Enter);
    dispatch(
        &app,
        "grid::edit::1:0value",
        UiEventKind::TextChanged("bad".into()),
    );
    dispatch(
        &app,
        "grid::edit::1:0value",
        UiEventKind::Submitted("bad".into()),
    );
    assert!(contains_text(&app.render(), "Enter an integer"));
    dispatch(
        &app,
        "grid::edit::1:0value",
        UiEventKind::TextChanged("42".into()),
    );
    dispatch(
        &app,
        "grid::edit::1:0value",
        UiEventKind::Submitted("42".into()),
    );
    assert!(keyed(&app.render(), "grid::edit::1:0value").is_none());
    assert!(contains_text(
        keyed(&app.render(), "grid::cell::1:0value").unwrap(),
        "42"
    ));
    click(&app, "grid::sort::value");
    click(&app, "grid::sort::value");
    keyboard(&app, "grid::resize::value", Key::ArrowRight);
    dispatch(
        &app,
        "grid::rows",
        UiEventKind::Scrolled {
            delta: Point::new(0.0, 200.0),
            offset: Point::new(0.0, 200.0),
        },
    );
    assert!(keyed(&app.render(), "grid::cell::1:0value").is_some());
}

#[test]
fn horizontal_scroll_notifications_do_not_change_the_vertical_virtual_window() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::data-table");
    let before = UiTree::new(app.render());
    let before: Vec<_> = before
        .node_ids()
        .iter()
        .filter_map(|id| before.key(*id))
        .filter(|key| key.starts_with("grid::row::"))
        .map(str::to_owned)
        .collect();
    dispatch(
        &app,
        "grid::horizontal",
        UiEventKind::Scrolled {
            delta: Point::new(50.0, 0.0),
            offset: Point::new(50.0, 0.0),
        },
    );
    let after = UiTree::new(app.render());
    let after: Vec<_> = after
        .node_ids()
        .iter()
        .filter_map(|id| after.key(*id))
        .filter(|key| key.starts_with("grid::row::"))
        .map(str::to_owned)
        .collect();
    assert_eq!(before, after);
}
