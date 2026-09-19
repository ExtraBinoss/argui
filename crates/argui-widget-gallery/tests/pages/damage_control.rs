use super::*;

#[test]
fn damage_control_switches_modes_and_keeps_a_controllable_workload() {
    use argui::animation::{Duration, Frame, Time};

    let gallery = Entity::new(WidgetGallery::default()).mount().unwrap();
    let click = |key: &str| {
        let mut tree = UiTree::new(gallery.render(Default::default()).unwrap());
        let node = tree
            .node_ids()
            .iter()
            .copied()
            .find(|id| tree.key(*id) == Some(key))
            .unwrap_or_else(|| panic!("missing {key}"));
        for event in tree.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility())) {
            if event.should_dispatch() {
                gallery.dispatch_event(&event).unwrap();
            }
        }
    };

    click("nav::damage-control");
    let initial = gallery.render(Default::default()).unwrap();
    assert!(contains_text(&initial, "Same scene, real renderer modes"));
    assert!(contains_text(&initial, "Waiting…"));
    assert!(
        keyed(&initial, "damage-control-mode::tab::0")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .state
            .selected
    );

    let before = keyed(&initial, "damage-control-orb")
        .unwrap()
        .style
        .margin
        .left;
    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(16_000_000),
            elapsed: Duration::from_millis(16),
        })
        .unwrap();
    let moving = gallery.render(Default::default()).unwrap();
    let after = keyed(&moving, "damage-control-orb")
        .unwrap()
        .style
        .margin
        .left;
    assert_ne!(after, before);

    for frame in 2..=8 {
        gallery
            .animation_frame(Frame {
                now: Time::from_nanos(frame * 16_000_000),
                elapsed: Duration::from_millis(16),
            })
            .unwrap();
    }

    click("damage-control-mode::tab::1");
    let off = gallery.render(Default::default()).unwrap();
    assert!(contains_text(
        &off,
        "Off forces a complete surface render on every animation frame."
    ));
    assert!(
        keyed(&off, "damage-control-mode::tab::1")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .state
            .selected
    );
    click("damage-control-mode::tab::1");
    click("damage-control-mode::tab::0");
    click("damage-control-mode::tab::0");
    assert!(
        keyed(
            &gallery.render(Default::default()).unwrap(),
            "damage-control-mode::tab::0"
        )
        .unwrap()
        .semantics
        .as_ref()
        .unwrap()
        .state
        .selected
    );
    click("damage-control-mode::tab::1");

    click("damage-control-pause");
    let paused = gallery.render(Default::default()).unwrap();
    assert!(contains_text(&paused, "Run"));
    let paused_position = keyed(&paused, "damage-control-orb")
        .unwrap()
        .style
        .margin
        .left;
    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(32_000_000),
            elapsed: Duration::from_millis(16),
        })
        .unwrap();
    assert_eq!(
        keyed(
            &gallery.render(Default::default()).unwrap(),
            "damage-control-orb"
        )
        .unwrap()
        .style
        .margin
        .left,
        paused_position
    );
    for _ in 0..8 {
        click("damage-control-step");
    }
    assert_ne!(
        keyed(
            &gallery.render(Default::default()).unwrap(),
            "damage-control-orb"
        )
        .unwrap()
        .style
        .margin
        .left,
        paused_position
    );

    click("damage-control-reset");
    click("nav::button");
    assert!(contains_text(
        &gallery.render(Default::default()).unwrap(),
        "Actions with variants, icons, loading and accessible activation."
    ));
    click("nav::damage-control");

    let reduced = WindowEnvironment {
        reduced_motion: true,
        ..WindowEnvironment::default()
    };
    let reduced_page = gallery.render(reduced.clone()).unwrap();
    let reduced_position = keyed(&reduced_page, "damage-control-orb")
        .unwrap()
        .style
        .margin
        .left;
    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(160_000_000),
            elapsed: Duration::from_millis(16),
        })
        .unwrap();
    assert_eq!(
        keyed(&gallery.render(reduced).unwrap(), "damage-control-orb")
            .unwrap()
            .style
            .margin
            .left,
        reduced_position
    );
}
