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
    let sidebar = app.bounds("sidebar").unwrap();
    let content = app.bounds("content").unwrap();
    assert!(content.origin.y >= sidebar.origin.y + sidebar.size.height);
}
