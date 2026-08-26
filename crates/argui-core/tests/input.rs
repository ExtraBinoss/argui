use argui_core::{Point, ScrollDelta};

#[test]
fn scroll_units_stay_explicit_until_ui_dispatch() {
    assert_ne!(
        ScrollDelta::Lines(Point::new(0.0, 1.0)),
        ScrollDelta::Pixels(Point::new(0.0, 1.0))
    );
}
