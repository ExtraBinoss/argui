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
    for _ in 0..24 {
        let sidebar = app.bounds("sidebar").unwrap();
        let button = app.bounds(selector.clone()).unwrap();
        let top = sidebar.origin.y + 4.0;
        let bottom = sidebar.origin.y + sidebar.size.height - 4.0;
        if button.origin.y >= top && button.origin.y + button.size.height <= bottom {
            app.get_by_role(Role::Button, page).click().unwrap();
            return;
        }
        let sidebar_middle = (top + bottom) * 0.5;
        let button_middle = button.origin.y + button.size.height * 0.5;
        let delta = (sidebar_middle - button_middle).clamp(-500.0, 500.0);
        app.wheel(
            Point::new(
                sidebar.origin.x + sidebar.size.width * 0.5,
                sidebar.origin.y + sidebar.size.height * 0.5,
            ),
            ScrollDelta::Pixels(Point::new(0.0, delta)),
        )
        .unwrap();
    }
    panic!("sidebar could not reveal navigation button {page:?}");
}
