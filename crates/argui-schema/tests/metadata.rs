use argui_schema::{
    EventId, EventSchema, NativeSchema, NativeTypeId, PropertyId, PropertySchema, SchemaValue,
    SlotArity, SlotId, SlotSchema, StylePartId, StylePartSchema, ValueType, VariantId,
    VariantSchema,
};
use argui_ui::EventType;

#[test]
fn property_builders_retain_binding_and_animation_contract() {
    let property = PropertySchema::new(PropertyId::from_raw(3), "count", ValueType::Int, "Count")
        .required()
        .default_value(SchemaValue::Int(7))
        .changed_by(EventId::from_raw(4))
        .not_animatable();
    assert!(property.required);
    assert_eq!(property.default, Some(SchemaValue::Int(7)));
    assert_eq!(property.change_event, Some(EventId::from_raw(4)));
    assert!(!property.animatable);
    assert_eq!(property.documentation, "Count");

    let ordinary = PropertySchema::new(PropertyId::from_raw(5), "size", ValueType::Float, "Size");
    assert!(!ordinary.required);
    assert_eq!(ordinary.default, None);
    assert_eq!(ordinary.change_event, None);
    assert!(ordinary.animatable);
}

#[test]
fn schema_builders_preserve_all_declarative_members() {
    let property = PropertySchema::new(PropertyId::from_raw(1), "value", ValueType::Bool, "Value");
    let event = EventSchema::new(EventId::from_raw(2), "click", EventType::Click, "Click")
        .payload(ValueType::Bool);
    let slot = SlotSchema {
        id: SlotId::from_raw(3),
        name: "children".into(),
        arity: SlotArity::Optional,
        documentation: "Child".into(),
    };
    let variant = VariantSchema {
        id: VariantId::from_raw(4),
        name: "quiet".into(),
        documentation: "Quiet".into(),
    };
    let style_part = StylePartSchema {
        id: StylePartId::from_raw(5),
        name: "track".into(),
        documentation: "Track".into(),
    };
    let schema = NativeSchema::new(NativeTypeId::from_raw(6), "Control", "A control")
        .property(property.clone())
        .event(event.clone())
        .slot(slot.clone())
        .variant(variant.clone())
        .style_part(style_part.clone());
    assert_eq!(schema.properties, [property]);
    assert_eq!(schema.events, [event]);
    assert_eq!(schema.slots, [slot]);
    assert_eq!(schema.variants, [variant]);
    assert_eq!(schema.style_parts, [style_part]);
    assert_eq!(schema.documentation, "A control");
}
