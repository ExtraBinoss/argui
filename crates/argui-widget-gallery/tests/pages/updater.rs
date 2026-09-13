use super::*;
use argui::animation::{Duration, Frame, Time};

#[test]
fn update_preview_downloads_cancels_retries_and_installs_without_changing_the_application() {
    let gallery = Entity::new(WidgetGallery::default()).mount().unwrap();
    let render = || gallery.render(Default::default()).unwrap();
    let dispatch = |key: &str, kind| {
        let mut tree = UiTree::new(render());
        let target = tree
            .node_ids()
            .iter()
            .copied()
            .find(|id| tree.key(*id) == Some(key))
            .unwrap();
        for event in tree.event_deliveries(target, kind) {
            if event.should_dispatch() {
                gallery.dispatch_event(&event).unwrap();
            }
        }
    };
    let click = |key: &str| dispatch(key, UiEventKind::Click(ClickEvent::accessibility()));
    let frame = |millis| {
        gallery
            .animation_frame(Frame {
                now: Time::ZERO,
                elapsed: Duration::from_millis(millis),
            })
            .unwrap();
    };
    click("nav::updater");
    frame(100);
    click("update-demo::trigger");
    assert!(contains_text(&render(), "A new version is available."));
    dispatch(
        "update-demo::panel",
        UiEventKind::KeyInput(argui::core::KeyInput {
            key: argui::core::Key::Escape,
            state: argui::core::KeyState::Pressed,
            modifiers: Default::default(),
            repeat: false,
            text: None,
        }),
    );
    assert!(!contains_text(&render(), "A new version is available."));
    click("update-demo::trigger");
    click("update-demo::primary");
    frame(3000);
    assert!(contains_text(&render(), "24.0 / 96.0 MB · 25%"));
    click("update-demo::primary");
    assert!(contains_text(
        &render(),
        "Download cancelled. You can try again."
    ));
    frame(1000);
    click("update-demo::close");
    click("update-demo-unknown");
    click("update-demo::trigger");
    click("update-demo::primary");
    frame(3000);
    assert!(contains_text(&render(), "24.0 MB downloaded"));
    frame(9000);
    assert!(contains_text(&render(), "Verifying the download…"));
    frame(100);
    frame(750);
    assert!(contains_text(
        &render(),
        "Download verified. Ready to install."
    ));
    click("update-demo::primary");
    frame(100);
    frame(750);
    assert!(contains_text(
        &render(),
        "Update installed. Restart the application to use it."
    ));
    click("update-demo::close");
    click("update-demo-unknown");
    click("update-demo-error");
    assert!(contains_text(
        &render(),
        "Unable to reach the update server. Try again when you're online."
    ));
    click("update-demo::primary");
    frame(100);
    assert!(contains_text(&render(), "Checking for updates…"));
    frame(500);
    assert!(contains_text(&render(), "A new version is available."));
    click("update-demo::close");
    click("update-demo-current");
    assert!(contains_text(&render(), "You're up to date."));
    click("update-demo::primary");
    frame(500);
    assert!(contains_text(&render(), "A new version is available."));
}
