use argui::core::{Point, ScrollDelta};
use argui::ui::Role;
use argui_example_dsl_virtual_list::Main;
use argui_testing::TestApp;

/// A large caller-owned model mounts only a bounded row window and can scroll.
#[test]
fn shared_dsl_virtual_list_keeps_large_models_bounded() {
    let root = Main::new();
    root.set_entries(
        (0..10_000)
            .map(|index| format!("Record {:05}", index + 1))
            .collect(),
    );
    let mut app = TestApp::new(root);
    app.assert_text("Record 00001");
    let mounted = |app: &TestApp<Main>| {
        app.semantics()
            .nodes
            .iter()
            .filter(|node| {
                node.semantics.role == Role::Text
                    && node
                        .semantics
                        .label
                        .as_deref()
                        .is_some_and(|label| label.starts_with("Record "))
            })
            .count()
    };
    assert!((1..80).contains(&mounted(&app)));
    app.wheel(
        Point::new(400.0, 350.0),
        ScrollDelta::Pixels(Point::new(0.0, -2_000.0)),
    )
    .unwrap();
    assert!(app.scroll_offset("virtual_viewport").unwrap().y > 0.0);
    assert!((1..80).contains(&mounted(&app)));
}
