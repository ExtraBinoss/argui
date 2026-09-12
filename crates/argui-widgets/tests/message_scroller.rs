use argui_core::{Color, ColorScheme, Point};
use argui_ui::{ClickEvent, Element, ScrollTarget, UiEvent, UiEventKind, UiTree};
use argui_widgets::{MessageScrollState, MessageScroller, MessageScrollerAction, shadcn};

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}

#[test]
fn appends_respect_reading_position_and_history_insertion_preserves_the_anchor() {
    let mut state = MessageScrollState::default();
    assert!(state.appended("chat", 1, 200.0).is_some());
    state.observe(
        &event(
            "other",
            UiEventKind::Scrolled {
                delta: Point::default(),
                offset: Point::new(0.0, 40.0),
            },
        ),
        "chat",
        200.0,
    );
    assert!(state.following);
    state.observe(
        &event(
            "chat",
            UiEventKind::Scrolled {
                delta: Point::default(),
                offset: Point::new(0.0, 40.0),
            },
        ),
        "chat",
        200.0,
    );
    assert!(!state.following);
    assert!(state.appended("chat", 3, 280.0).is_none());
    assert_eq!(state.unread, 3);
    let request = state.prepended("chat", 120.0);
    assert!(matches!(request.target, ScrollTarget::Offset { offset, .. } if offset.y == 160.0));
    let request = state.latest("chat");
    assert!(matches!(request.target, ScrollTarget::Offset { offset, .. } if offset.y == 400.0));
    assert_eq!(state.unread, 0);
    assert!(state.following);
    state.jump("message-2");
    assert!(!state.following);
    state.observe(
        &event(
            "chat",
            UiEventKind::Scrolled {
                delta: Point::default(),
                offset: Point::new(0.0, 398.0),
            },
        ),
        "chat",
        400.0,
    );
    assert!(state.following);
    state.observe(
        &event("link", UiEventKind::Click(ClickEvent::accessibility())),
        "chat",
        400.0,
    );
    assert!(!state.following);
}

#[test]
fn scroller_controls_reflect_history_loading_and_live_edge() {
    let mut scroller = MessageScroller::new("chat", "Conversation", Element::text("Hello"));
    for has_earlier in [false, true] {
        for loading in [false, true] {
            for following in [false, true] {
                scroller.has_earlier = has_earlier;
                scroller.loading = loading;
                scroller.state.following = following;
                let root = scroller
                    .clone()
                    .build(shadcn(Color::BLACK).resolve(ColorScheme::Dark));
                assert_eq!(root.children.len(), if following { 1 } else { 2 });
                assert_eq!(
                    scroller.action(&event(
                        "chat::earlier",
                        UiEventKind::Click(ClickEvent::accessibility())
                    )),
                    (has_earlier && !loading).then_some(MessageScrollerAction::LoadEarlier)
                );
                assert_eq!(
                    scroller.action(&event(
                        "chat::latest",
                        UiEventKind::Click(ClickEvent::accessibility())
                    )),
                    (!following).then_some(MessageScrollerAction::Latest)
                );
            }
        }
    }
}

#[test]
fn releasing_a_navigation_key_does_not_suspend_the_live_edge() {
    let mut state = MessageScrollState::default();
    for (phase, following) in [
        (argui_core::KeyState::Released, true),
        (argui_core::KeyState::Pressed, false),
    ] {
        state.observe(
            &event(
                "chat",
                UiEventKind::KeyInput(argui_core::KeyInput {
                    key: argui_core::Key::End,
                    state: phase,
                    modifiers: Default::default(),
                    repeat: false,
                    text: None,
                }),
            ),
            "chat",
            200.0,
        );
        assert_eq!(state.following, following);
    }
}
