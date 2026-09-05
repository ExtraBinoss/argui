use argui_animation::{Duration, Tween};
use argui_core::{Point, Rect, Size};
use argui_ui::{ScrollAlignment, ScrollBehavior, ScrollRequest, ScrollTarget, Sides};

#[test]
fn scroll_requests_encode_offset_reveal_and_rect_targets() {
    let offset = ScrollRequest::offset("feed", Point::new(0.0, 240.0))
        .align(ScrollAlignment::Center, ScrollAlignment::End)
        .margin(Sides {
            left: 4.0,
            right: 8.0,
            top: 12.0,
            bottom: 16.0,
        })
        .behavior(ScrollBehavior::Smooth(Tween::new(Duration::from_millis(
            180,
        ))));
    assert!(matches!(
        offset.target,
        ScrollTarget::Offset { ref container, offset }
            if container == &argui_ui::FocusTarget::from("feed")
                && offset == Point::new(0.0, 240.0)
    ));
    assert_eq!(offset.x, ScrollAlignment::Center);
    assert_eq!(offset.y, ScrollAlignment::End);
    assert_eq!(offset.margin.top, 12.0);
    assert!(matches!(offset.behavior, ScrollBehavior::Smooth(_)));

    let reveal = ScrollRequest::reveal("result");
    assert!(matches!(
        reveal.target,
        ScrollTarget::Element(ref target)
            if target == &argui_ui::FocusTarget::from("result")
    ));
    assert_eq!(reveal.x, ScrollAlignment::Nearest);
    assert_eq!(reveal.y, ScrollAlignment::Nearest);

    let visible = Rect::new(Point::new(10.0, 20.0), Size::new(80.0, 40.0));
    let rect = ScrollRequest::rect("viewport", visible);
    assert!(matches!(
        rect.target,
        ScrollTarget::Rect { ref container, rect }
            if container == &argui_ui::FocusTarget::from("viewport") && rect == visible
    ));
}
