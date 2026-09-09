use argui_core::{Point, Rect, Size};
use argui_widgets::MenuIntent;
use std::time::Duration;

#[test]
fn submenu_corridor_delays_sibling_switching_and_cancels_on_entry() {
    let mut intent = MenuIntent::default();
    let delay = Duration::from_millis(200);
    assert_eq!(intent.take_due(Duration::ZERO), None);
    for direction in [1.0, -1.0] {
        let popup = Rect::new(
            Point::new(if direction > 0.0 { 100.0 } else { -150.0 }, 0.0),
            Size::new(50.0, 100.0),
        );
        intent.schedule("sibling", Point::new(0.0, 50.0), Duration::ZERO, delay);
        assert_eq!(intent.take_due(Duration::from_millis(100)), None);
        assert!(intent.pointer_moved(
            Point::new(50.0 * direction, 50.0),
            popup,
            Duration::from_millis(100),
            delay
        ));
        assert_eq!(intent.next_deadline(), Some(Duration::from_millis(300)));
        assert!(!intent.pointer_moved(
            Point::new(125.0 * direction, 50.0),
            popup,
            Duration::from_millis(150),
            delay
        ));
        assert_eq!(intent.next_deadline(), None);
    }
    intent.schedule("sibling", Point::default(), Duration::ZERO, delay);
    assert!(!intent.pointer_moved(
        Point::new(-20.0, -20.0),
        Rect::new(Point::new(100.0, 0.0), Size::new(50.0, 100.0)),
        Duration::from_millis(100),
        delay
    ));
    assert_eq!(intent.take_due(delay), Some("sibling".into()));
    assert_eq!(intent.next_deadline(), None);
}
