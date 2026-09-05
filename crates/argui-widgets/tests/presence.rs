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

#[test]
fn interrupted_open_and_close_transitions_reverse_from_current_progress() {
    let mut presence = Presence::default();
    presence.set_open(true, false);
    assert!(!presence.advance(Duration::ZERO));
    assert!(presence.animating());
    presence.advance(Duration::from_millis(70));
    assert!(presence.visible());
    assert!(presence.animating());

    presence.set_open(false, false);
    assert!(presence.visible());
    assert!(presence.animating());
    assert!(!presence.advance(Duration::from_millis(40)));
    assert!(presence.visible());

    presence.set_open(true, false);
    presence.advance(Duration::from_millis(70));
    assert!(presence.visible());
    assert!(presence.animating());
    assert!(!presence.advance(Duration::from_millis(100)));
    assert!(presence.visible());
    assert!(!presence.animating());
}

#[test]
fn reduced_motion_interrupts_an_exit_and_decoration_releases_all_input() {
    let mut presence = Presence::default();
    presence.set_open(true, false);
    presence.advance(Duration::from_millis(70));
    presence.set_open(false, true);
    assert!(!presence.visible());
    assert!(!presence.animating());
    presence.set_open(true, true);
    let open = presence.decorate(Element::container([]));
    assert!(!open.semantic_hidden);
    assert_eq!(open.hit_test.pointer_events, PointerEvents::Auto);
    assert!(open.bindings.len() >= 2);

    let nested = Element::container([
        Element::container([]).interaction(argui_ui::Interaction::default().focusable(true)),
        Element::container([
            Element::text("focus").interaction(argui_ui::Interaction::default().focusable(true))
        ])
        .interaction(argui_ui::Interaction::default().focusable(true)),
    ])
    .interaction(argui_ui::Interaction::default().focusable(true));
    presence.set_open(false, true);
    let closed = presence.decorate(nested);
    assert!(closed.semantic_hidden);
    assert_eq!(closed.hit_test.pointer_events, PointerEvents::None);
    assert!(closed.focus_scope.is_none());
    assert!(!closed.interaction.as_ref().unwrap().enabled);
    assert!(!closed.interaction.as_ref().unwrap().focusable);
    assert!(closed.children.iter().all(|child| {
        child
            .interaction
            .as_ref()
            .is_none_or(|interaction| !interaction.enabled && !interaction.focusable)
    }));
    assert!(
        !closed.children[1].children[0]
            .interaction
            .as_ref()
            .unwrap()
            .focusable
    );
}
