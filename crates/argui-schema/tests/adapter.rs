use argui_schema::{
    EventId, NativeElementInput, NativeEventValue, NativeSlotValue, PropertyId, SchemaValue, SlotId,
};
use argui_ui::{Element, EventHandler, EventHandlerId, EventOwnerId};

#[test]
fn input_lookup_resolves_first_matching_property_event_and_slot() {
    let property = PropertyId::from_raw(7);
    let event = EventId::from_raw(8);
    let slot = SlotId::from_raw(9);
    let first_handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(1), 2));
    let second_handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(3), 4));
    let input = NativeElementInput::new()
        .property(property, SchemaValue::Int(11))
        .property(property, SchemaValue::Int(22))
        .event(NativeEventValue::new(event, first_handler))
        .event(NativeEventValue::new(event, second_handler))
        .slot(NativeSlotValue::new(slot, [Element::text("first")]))
        .slot(NativeSlotValue::new(slot, [Element::text("second")]));
    assert_eq!(input.get(property), Some(&SchemaValue::Int(11)));
    assert_eq!(input.event_handler(event), Some(first_handler));
    assert_eq!(input.children(slot).len(), 1);
    assert_eq!(input.children(slot)[0], Element::text("first"));
    assert_eq!(input.get(PropertyId::from_raw(99)), None);
    assert_eq!(input.event_handler(EventId::from_raw(99)), None);
    assert!(input.children(SlotId::from_raw(99)).is_empty());
}
