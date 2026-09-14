use super::*;

#[test]
fn animation_lab_replays_spring_layout_color_and_transform_tracks() {
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
    click("nav::motion");
    let initial = gallery.render(Default::default()).unwrap();
    assert!(contains_text(&initial, "Spring physics"));
    let initial_spring = keyed(&initial, "motion-spring-square").unwrap().transform;
    let initial_scale = keyed(&initial, "motion-morph").unwrap().transform.scale.x;

    for index in 0..=30 {
        gallery
            .animation_frame(Frame {
                now: Time::from_nanos(index * 16_000_000),
                elapsed: Duration::from_millis(16),
            })
            .unwrap();
    }
    let moving = gallery.render(Default::default()).unwrap();
    let spring = keyed(&moving, "motion-spring-square").unwrap().transform;
    let scale = keyed(&moving, "motion-morph").unwrap().transform.scale.x;
    assert!(spring.translation.x > initial_spring.translation.x);
    assert!(spring.rotation > initial_spring.rotation);
    assert!(scale > initial_scale);

    click("motion-replay");
    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(600_000_000),
            elapsed: Duration::from_millis(16),
        })
        .unwrap();
    for index in 1..=30 {
        gallery
            .animation_frame(Frame {
                now: Time::from_nanos(600_000_000 + index * 16_000_000),
                elapsed: Duration::from_millis(16),
            })
            .unwrap();
    }
    let reversed = gallery.render(Default::default()).unwrap();
    assert!(keyed(&reversed, "motion-morph").unwrap().transform.scale.x < scale);
}

#[test]
fn reduced_motion_snaps_the_animation_lab_to_its_destination() {
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "nav::motion");
    let root = gallery.render_in(WindowEnvironment {
        reduced_motion: true,
        ..Default::default()
    });
    let square = keyed(&root, "motion-spring-square").unwrap();
    let morph = keyed(&root, "motion-morph").unwrap();
    assert_eq!(square.transform.translation.x, 150.0);
    assert_eq!(morph.style.size.width.value(), 212.0);
    assert_eq!(morph.transform.scale.x, 1.0);
}
