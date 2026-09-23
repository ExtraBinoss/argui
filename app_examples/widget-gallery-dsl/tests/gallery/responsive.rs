use super::*;

/// The content sits next to navigation when wide and below it when narrow.
#[test]
fn gallery_wraps_sidebar_and_content_responsively() {
    let mut app = TestApp::new(Main::new());
    let coordinates = |app: &TestApp<Main>| {
        let nodes = &app.semantics().nodes;
        let navigation = nodes
            .iter()
            .find(|node| {
                node.semantics.role == Role::Button
                    && node.semantics.label.as_deref() == Some("Button")
            })
            .unwrap()
            .bounds
            .origin;
        let heading = nodes
            .iter()
            .find(|node| {
                node.semantics.role == Role::Text
                    && node.semantics.label.as_deref() == Some("Button")
            })
            .unwrap()
            .bounds
            .origin;
        (navigation, heading)
    };
    let (wide_nav, wide_heading) = coordinates(&app);
    assert!(wide_heading.x > wide_nav.x);
    assert!(
        wide_heading.y < wide_nav.y + 100.0,
        "wide navigation={wide_nav:?}, heading={wide_heading:?}"
    );

    app.resize(Size::new(420.0, 760.0)).unwrap();
    for _ in 0..10 {
        app.wheel(
            Point::new(100.0, 300.0),
            ScrollDelta::Pixels(Point::new(0.0, -700.0)),
        )
        .unwrap();
        if app.scroll_offset("workspace").unwrap().y > 0.0 {
            break;
        }
    }
    assert!(app.scroll_offset("workspace").unwrap().y > 0.0);
    let (_, narrow_heading) = coordinates(&app);
    assert!(
        narrow_heading.y > 100.0,
        "narrow heading={narrow_heading:?}"
    );
}
