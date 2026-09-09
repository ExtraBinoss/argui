use super::*;
use argui::runtime::tasks::TaskRuntime;
use std::{sync::mpsc, time::Duration};

#[test]
fn asynchronous_search_validates_queries_cancels_stale_work_and_scrolls_results() {
    let app = Entity::new(WidgetGallery::default());
    let (sender, wake) = mpsc::channel();
    let runtime = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    app.set_task_runtime(runtime.clone());
    click(&app, "nav::async-tasks");
    let drain = || {
        while runtime.pending() > 0 {
            runtime.drain();
            if runtime.pending() > 0 {
                wake.recv_timeout(Duration::from_secs(5)).unwrap();
            }
        }
    };
    for (query, result) in [
        ("id:9998", "2 matching drafts"),
        ("Architecture", "2000 matching drafts"),
    ] {
        dispatch(&app, "tasks-query", UiEventKind::TextChanged(query.into()));
        drain();
        assert!(contains_text(&app.render(), result));
    }
    dispatch(
        &app,
        "tasks-results",
        UiEventKind::Scrolled {
            delta: Default::default(),
            offset: argui::core::Point::new(0.0, 320.0),
        },
    );
    dispatch(
        &app,
        "tasks-query",
        UiEventKind::TextChanged("id:abc".into()),
    );
    drain();
    let root = app.render();
    fn invalid(element: &Element) -> bool {
        matches!(&element.kind, argui::ui::ElementKind::Text { content, .. } if content.as_str().starts_with("Invalid query:"))
            || element.children.iter().any(invalid)
    }
    assert!(invalid(&root));
    click(&app, "tasks-search");
    click(&app, "tasks-cancel");
    drain();
    assert!(contains_text(&app.render(), "Cancelled"));
}
