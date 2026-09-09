use super::*;
#[test]
fn notifications_can_be_added_paused_and_closed_without_moving_focus() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::toast");
    click(&app, "toast-add");
    assert!(contains_text(&app.render(), "Notification 1"));
    dispatch(&app, "toasts::close::1", UiEventKind::Focused);
    dispatch(&app, "toasts::close::1", UiEventKind::Blurred);
    click(&app, "toasts::close::1");
    assert!(!contains_text(&app.render(), "Notification 1"));
}

#[test]
fn toast_capacity_is_reported_and_closing_visible_entries_promotes_the_queue() {
    let app = Entity::new(WidgetGallery::default());
    app.set_task_runtime(argui::runtime::tasks::TaskRuntime::new(|| {}));
    click(&app, "nav::toast");
    dispatch(&app, "toast-add", UiEventKind::Focused);
    for _ in 0..9 {
        click(&app, "toast-add");
    }
    assert!(contains_text(&app.render(), "Capacity"));
    assert!(contains_text(&app.render(), "Notification 1"));
    assert!(!contains_text(&app.render(), "Notification 4"));
    click(&app, "toasts::close::1");
    assert!(contains_text(&app.render(), "Notification 4"));
    for id in 2..=8 {
        click(&app, &format!("toasts::close::{id}"));
    }
    assert!(!contains_text(&app.render(), "Notification 8"));
}
