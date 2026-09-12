use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Point, PointerId};
use argui_ui::{
    ClickEvent, Element, GestureDelivery, GestureEvent, GestureKind, GesturePhase, UiEvent,
    UiEventKind, UiTree,
};
use argui_widgets::{Carousel, shadcn};

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}
fn key(value: Key) -> UiEventKind {
    UiEventKind::KeyInput(KeyInput {
        key: value,
        state: KeyState::Pressed,
        repeat: false,
        modifiers: Default::default(),
        text: None,
    })
}

#[test]
fn carousel_boundaries_looping_rtl_and_swipes_share_one_selection_contract() {
    let mut carousel = Carousel::new(
        "c",
        "Photos",
        [
            Element::text("One").keyed("one"),
            Element::text("Two").keyed("two"),
        ],
        0,
    );
    let click = |part: &str| {
        event(
            &format!("c::{part}"),
            UiEventKind::Click(ClickEvent::accessibility()),
        )
    };
    assert_eq!(carousel.action(&click("previous")), None);
    assert_eq!(carousel.action(&click("next")), Some(1));
    carousel.selected = 1;
    assert_eq!(carousel.action(&click("next")), None);
    assert_eq!(carousel.action(&click("previous")), Some(0));
    carousel.looping = true;
    assert_eq!(carousel.action(&click("next")), Some(0));
    carousel.selected = 0;
    assert_eq!(carousel.action(&click("previous")), Some(1));
    for (rtl, value, expected) in [
        (false, Key::ArrowRight, Some(1)),
        (true, Key::ArrowLeft, Some(1)),
        (false, Key::Home, Some(0)),
        (false, Key::End, Some(1)),
        (false, Key::Tab, None),
    ] {
        carousel.rtl = rtl;
        assert_eq!(carousel.action(&event("c::viewport", key(value))), expected);
    }
    let node = click("next").target;
    let gesture = |phase, x| {
        UiEventKind::Gesture(GestureEvent {
            target: node,
            pointer: PointerId::new(1),
            phase,
            delivery: GestureDelivery::Immediate,
            kind: GestureKind::Pan {
                position: Point::default(),
                delta: Point::default(),
                total: Point::new(x, 0.0),
                velocity: Point::default(),
            },
        })
    };
    assert_eq!(
        carousel.action(&event("c::viewport", gesture(GesturePhase::Ended, -60.0))),
        Some(1)
    );
    assert_eq!(
        carousel.action(&event("c::viewport", gesture(GesturePhase::Ended, -20.0))),
        None
    );
    assert_eq!(
        carousel.action(&event(
            "c::viewport",
            gesture(GesturePhase::Cancelled, -60.0)
        )),
        None
    );
    let tree = UiTree::new(carousel.build(shadcn(Color::BLACK).resolve(ColorScheme::Light)));
    assert!(
        !tree
            .node_ids()
            .iter()
            .any(|node| tree.key(*node) == Some("two"))
    );
    carousel.slides.clear();
    assert_eq!(carousel.action(&click("next")), None);
    assert_eq!(carousel.action(&event("c::viewport", key(Key::End))), None);
    assert!(
        UiTree::new(carousel.build(shadcn(Color::BLACK).resolve(ColorScheme::Dark)))
            .semantic_diagnostics()
            .is_empty()
    );
}
