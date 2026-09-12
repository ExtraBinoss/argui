use argui_core::{Color, ColorScheme, Point, PointerEvent, PointerPhase};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Button, HoverCard, HoverCardState, shadcn};
use std::time::Duration;

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}
fn pointer(phase: PointerPhase) -> UiEventKind {
    UiEventKind::Pointer(PointerEvent::mouse(phase, Point::default()))
}

#[test]
fn moving_across_the_gap_or_focusing_panel_content_cancels_close_deadlines() {
    let mut state = HoverCardState::new("profile");
    let at = Duration::from_millis;
    assert!(!state.advance(at(0)));
    assert!(!state.update(&event("other", UiEventKind::Focused), at(0)));
    assert!(state.update(&event("profile", pointer(PointerPhase::Entered)), at(0)));
    assert_eq!(state.next_deadline(), Some(at(500)));
    assert!(!state.advance(at(499)));
    assert!(state.advance(at(500)));
    state.update(&event("profile", pointer(PointerPhase::Left)), at(501));
    state.update(
        &event("profile::content", pointer(PointerPhase::Entered)),
        at(550),
    );
    assert_eq!(state.next_deadline(), None);
    assert!(state.is_open());
    state.update(&event("profile::content", UiEventKind::Focused), at(560));
    state.update(
        &event("profile::content", pointer(PointerPhase::Left)),
        at(570),
    );
    assert_eq!(state.next_deadline(), None);
    state.update(&event("profile::content", UiEventKind::Blurred), at(580));
    assert!(state.advance(at(780)));
    assert!(!state.is_open());
    state.update(
        &event("profile", UiEventKind::Click(ClickEvent::accessibility())),
        at(800),
    );
    assert!(state.is_open());
    state.update(
        &event("profile::content", UiEventKind::DismissRequested),
        at(801),
    );
    assert!(!state.is_open());
    state.update(&event("profile", UiEventKind::Focused), at(900));
    state.reset();
    assert_eq!(state.next_deadline(), None);
    assert!(!state.update(&event("profile", pointer(PointerPhase::Moved)), at(901)));
}

#[test]
fn rich_preview_keeps_interactive_content_in_a_nonmodal_scope() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    for open in [false, true] {
        let card = HoverCard::new(
            "profile",
            "Ada",
            open,
            Button::new("profile", "Ada", theme.ghost_button()).build(),
            Button::new("follow", "Follow", theme.button()).build(),
        );
        let root = card.build(theme);
        assert_eq!(root.children.len(), if open { 2 } else { 1 });
        if open {
            assert!(!root.children[1].focus_scope.as_ref().unwrap().traps());
        }
        assert!(UiTree::new(root).semantic_diagnostics().is_empty());
    }
}
