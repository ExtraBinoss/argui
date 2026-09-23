use argui::{
    core::{Point, ScrollDelta},
    runtime::Render,
    ui::Role,
};
use argui_testing::{Selector, TestApp};

/// Scrolls the gallery sidebar until `page` is visible, then activates its button.
///
/// `app` is the generated or live gallery and `page` is its accessible navigation label.
pub fn navigate_to_page<A: Render>(app: &mut TestApp<A>, page: &str) {
    let selector = Selector::role(Role::Button, page);
    let sidebar = app.bounds("sidebar_list").unwrap();
    let pointer = Point::new(
        sidebar.origin.x + sidebar.size.width * 0.5,
        sidebar.origin.y + sidebar.size.height * 0.5,
    );
    app.wheel(pointer, ScrollDelta::Pixels(Point::new(0.0, 10_000.0)))
        .unwrap();
    for _ in 0..32 {
        let button = app.bounds(selector.clone()).ok();
        let top = sidebar.origin.y + 4.0;
        let bottom = sidebar.origin.y + sidebar.size.height - 4.0;
        if button.as_ref().is_some_and(|button| {
            button.origin.y >= top && button.origin.y + button.size.height <= bottom
        }) {
            app.get_by_role(Role::Button, page).click().unwrap();
            return;
        }
        app.wheel(pointer, ScrollDelta::Pixels(Point::new(0.0, -500.0)))
            .unwrap();
    }
    panic!("sidebar could not reveal navigation button {page:?}");
}
