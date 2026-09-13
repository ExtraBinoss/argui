use super::*;

#[test]
fn mounted_nested_counters_receive_frames_and_render_intermediate_positions() {
    use argui::animation::{Duration, Frame, Time};
    let gallery = Entity::new(WidgetGallery::default()).mount().unwrap();
    let click = |key: &str| {
        let mut tree = UiTree::new(gallery.render(Default::default()).unwrap());
        let node = tree
            .node_ids()
            .iter()
            .copied()
            .find(|id| tree.key(*id) == Some(key))
            .unwrap();
        for event in tree.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility())) {
            if event.should_dispatch() {
                gallery.dispatch_event(&event).unwrap();
            }
        }
    };
    click("nav::animated-text");
    click("animated-increment");
    let start = gallery.render(Default::default()).unwrap();
    let before = keyed(&start, "animated-Roll").unwrap();
    assert_eq!(before.children[1].children[0].transform.translation.y, 0.0);
    for index in 1..=2 {
        gallery
            .animation_frame(Frame {
                now: Time::from_nanos(index * 50_000_000),
                elapsed: Duration::from_millis(50),
            })
            .unwrap();
    }
    let middle = gallery.render(Default::default()).unwrap();
    let counter = keyed(&middle, "animated-Roll").unwrap();
    assert!(counter.children[0].ptr_eq(&before.children[0]));
    let y = counter.children[1].children[0].transform.translation.y;
    assert!(y < 0.0 && y > -55.0, "intermediate reel position: {y}");
    let fade = keyed(&middle, "animated-Fade").unwrap();
    let argui::ui::ElementKind::Text { style, .. } = &fade.children[1].children[0].children[1].kind
    else {
        panic!("incoming digit")
    };
    let alpha = style.color.to_linear_rgba()[3];
    assert!(alpha > 0.0 && alpha < 1.0);
    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(1_000_000_000),
            elapsed: Duration::from_secs(1),
        })
        .unwrap();
    let end = gallery.render(Default::default()).unwrap();
    assert!(
        keyed(&end, "animated-Roll").unwrap().children[1]
            .children
            .is_empty()
    );
}

#[test]
fn animated_values_follow_controls_and_survive_navigation() {
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "nav::animated-text");
    let environment = WindowEnvironment {
        reduced_motion: true,
        ..Default::default()
    };
    for (action, expected) in [
        ("animated-increment", "11"),
        ("animated-decrement", "10"),
        ("animated-carry", "99"),
        ("animated-carry", "100"),
        ("animated-reset", "10"),
    ] {
        click(&gallery, action);
        let root = gallery.render_in(environment.clone());
        for mode in ["Roll", "Slide", "Fade"] {
            let counter = keyed(&root, &format!("animated-{mode}")).unwrap();
            assert_eq!(
                counter.semantics.as_ref().unwrap().label.as_deref(),
                Some(expected)
            );
            assert_eq!(counter.children.len(), expected.len());
        }
    }
    for expected in ["Saved", "Draft", "Saved"] {
        click(&gallery, "animated-save");
        let root = gallery.render_in(environment.clone());
        assert_eq!(
            keyed(&root, "animated-status")
                .unwrap()
                .semantics
                .as_ref()
                .unwrap()
                .label
                .as_deref(),
            Some(expected)
        );
    }
    click(&gallery, "nav::button");
    click(&gallery, "nav::animated-text");
    let root = gallery.render_in(environment);
    assert_eq!(
        keyed(&root, "animated-status")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Saved")
    );
}
