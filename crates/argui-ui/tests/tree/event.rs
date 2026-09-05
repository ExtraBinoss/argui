use argui_core::{Point, PointerEvent, PointerPhase};
use argui_ui::{
    ClickEvent, Element, EventHandlerId, EventListener, EventOwnerId, EventType, UiEventKind,
    UiTree,
};

#[test]
fn public_tree_events_keep_non_bubbling_pointer_enter_at_the_target() {
    let root = Element::container([Element::text("target").on(EventListener::new(
        EventType::PointerEnter,
        EventHandlerId::new(EventOwnerId(1), 0),
    ))])
    .on(EventListener::new(
        EventType::PointerEnter,
        EventHandlerId::new(EventOwnerId(1), 1),
    ));
    let mut tree = UiTree::new(root);
    let target = tree.node_id_at(1).unwrap();
    let events = tree.event_deliveries(
        target,
        UiEventKind::Pointer(PointerEvent::mouse(PointerPhase::Entered, Point::default())),
    );

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].current_target(), target);
    assert!(!events[0].bubbles());
    let click = tree.event_deliveries(target, UiEventKind::Click(ClickEvent::accessibility()));
    assert!(click.is_empty());
}
