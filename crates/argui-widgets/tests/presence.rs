use argui_animation::Duration;
use argui_ui::{Element, PointerEvents};
use argui_widgets::Presence;

#[test]
fn presence_retains_exit_without_intercepting_input_and_can_reverse() {
    let mut presence = Presence::default();
    assert!(!presence.visible());
    presence.set_open(true, false);
    assert!(presence.visible());
    assert!(presence.animating());
    presence.advance(Duration::from_millis(140));
    assert!(!presence.animating());
    presence.set_open(false, false);
    let exit = presence.decorate(Element::container([]));
    assert_eq!(exit.hit_test.pointer_events, PointerEvents::None);
    assert!(exit.semantic_hidden);
    assert!(!presence.advance(Duration::from_millis(50)));
    assert!(presence.visible());
    presence.set_open(true, false);
    presence.advance(Duration::from_millis(140));
    assert!(presence.visible());
    presence.set_open(false, false);
    assert!(presence.advance(Duration::from_millis(100)));
    assert!(!presence.visible());
}

#[test]
fn reduced_motion_finishes_without_requesting_frames() {
    let mut presence = Presence::default();
    presence.set_open(true, true);
    assert!(presence.visible());
    assert!(!presence.animating());
    presence.set_open(false, true);
    assert!(!presence.visible());
    assert!(!presence.animating());
}
