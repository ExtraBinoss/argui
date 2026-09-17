use super::*;

#[test]
fn animation_lab_auto_runs_loops_and_toggles_every_track() {
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
    assert!(contains_text(&initial, "Stop animations"));
    for label in [
        "AnimatedOpacity",
        "Animated size",
        "Color & radius",
        "Padding & gap",
        "Composed transform",
        "Border & shadow",
        "Interruptible retargeting",
        "Typed multi-property timeline",
        "Multiple keyframes",
        "Held keyframes",
        "Step easing",
        "Alternate direction",
        "Stagger schedule",
        "Spring physics",
        "Bounded inertia",
        "Velocity-preserving retarget",
        "Squash & stretch",
        "Additive composition",
        "User-defined Curve",
        "Layer & compositor",
    ] {
        assert!(
            contains_text(&initial, label),
            "missing motion example: {label}"
        );
    }
    assert_eq!(count_cards(&initial), 20);
    let initial_spring = keyed(&initial, "motion-spring-square").unwrap().transform;
    let initial_scale = keyed(&initial, "motion-morph").unwrap().transform.scale.x;

    click("motion-toggle");
    assert!(contains_text(
        &gallery.render(Default::default()).unwrap(),
        "Run animations"
    ));
    click("motion-toggle");
    assert!(contains_text(
        &gallery.render(Default::default()).unwrap(),
        "Stop animations"
    ));

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

    for index in 31..=120 {
        gallery
            .animation_frame(Frame {
                now: Time::from_nanos(index * 16_000_000),
                elapsed: Duration::from_millis(16),
            })
            .unwrap();
    }
    let looped = gallery.render(Default::default()).unwrap();
    assert_eq!(
        keyed(&looped, "motion-implicit-transform")
            .unwrap()
            .transform
            .translation
            .x,
        130.0,
        "the implicit examples entered a third cycle instead of settling"
    );

    let before_stop = keyed(&looped, "motion-morph").unwrap().transform.scale.x;
    click("motion-toggle");
    let stopped = gallery.render(Default::default()).unwrap();
    assert!(contains_text(&stopped, "Run animations"));
    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(5_000_000_000),
            elapsed: Duration::from_millis(16),
        })
        .unwrap();
    let frozen = gallery.render(Default::default()).unwrap();
    assert_eq!(
        keyed(&frozen, "motion-morph").unwrap().transform.scale.x,
        before_stop
    );

    click("motion-toggle");
    let resumed = gallery.render(Default::default()).unwrap();
    assert!(contains_text(&resumed, "Stop animations"));
    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(5_000_000_000),
            elapsed: Duration::from_millis(16),
        })
        .unwrap();
    assert_eq!(
        keyed(&gallery.render(Default::default()).unwrap(), "motion-morph")
            .unwrap()
            .transform
            .scale
            .x,
        before_stop,
        "resuming preserves the paused phase"
    );
    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(5_080_000_000),
            elapsed: Duration::from_millis(16),
        })
        .unwrap();
    assert_ne!(
        keyed(&gallery.render(Default::default()).unwrap(), "motion-morph")
            .unwrap()
            .transform
            .scale
            .x,
        before_stop
    );
}

/// Counts animation example cards recursively in the rendered tree.
fn count_cards(element: &Element) -> usize {
    usize::from(
        element
            .key
            .as_deref()
            .is_some_and(|key| key.starts_with("motion-card-")),
    ) + element.children.iter().map(count_cards).sum::<usize>()
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

    let restored = gallery.render_in(WindowEnvironment::default());
    assert!(contains_text(&restored, "Stop animations"));
}
