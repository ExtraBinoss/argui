//! Application-owned native tile used to prove extension registration without Argui edits.

use argui::{
    schema::{
        self, EventId, EventSchema, NativeElementInput, NativeSchema, NativeTypeId, PropertyId,
        PropertySchema, SchemaError, SchemaRegistry, SchemaValue, SlotArity, SlotId, SlotSchema,
        ValueType,
    },
    ui::{Element, EventType, GestureSet, HitTestStyle, Interaction, PointerEvents, TapGesture},
};

/// Stable application primitive type identity.
pub const PROOF_TILE: NativeTypeId = NativeTypeId::from_raw(4096);
/// Stable identity of the tile's text property.
pub const LABEL: PropertyId = PropertyId::from_raw(1);
/// Stable identity of the tile's asset property.
pub const ICON: PropertyId = PropertyId::from_raw(2);
/// Stable identity of the tile's child-content slot.
pub const BODY: SlotId = SlotId::from_raw(1);
/// Stable identity of its click event.
pub const ACTIVATE: EventId = EventId::from_raw(1);

/// Registers an application native tile beside the built-in Argui primitives.
///
/// Returns the registry installed in the compiler, AOT renderer, live runtime,
/// and language server.
///
/// # Errors
///
/// Returns schema validation failures if an application ID or member collides.
pub fn registry() -> Result<SchemaRegistry, SchemaError> {
    let mut registry = schema::builtin::registry()?;
    let tile = NativeSchema::new(PROOF_TILE, "ProofTile", "Application-owned clickable tile.")
        .abi_version(1)
        .property(PropertySchema::new(LABEL, "label", ValueType::String, "Tile title.").required())
        .property(PropertySchema::new(
            ICON,
            "icon",
            ValueType::Asset,
            "Optional icon asset.",
        ))
        .event(EventSchema::new(
            ACTIVATE,
            "activate",
            EventType::Click,
            "Tile activation.",
        ))
        .slot(SlotSchema {
            id: BODY,
            name: "body".into(),
            arity: SlotArity::Many,
            documentation: "Extra tile content.".into(),
        });
    registry.register(tile, |input: &NativeElementInput| {
        let Some(SchemaValue::String(label)) = input.get(LABEL) else {
            return Err(SchemaError::Adapter("ProofTile requires label".into()));
        };
        if let Some(value) = input.get(ICON)
            && !matches!(value, SchemaValue::Asset(_))
        {
            return Err(SchemaError::Adapter(
                "ProofTile icon must be an asset".into(),
            ));
        }
        let mut children = vec![Element::text(label.clone())];
        children.extend_from_slice(input.children(BODY));
        let mut element = Element::column(children)
            .interaction(
                Interaction::default().gestures(GestureSet::default().tap(TapGesture::default())),
            )
            .hit_test(HitTestStyle::default().pointer_events(PointerEvents::BoxOnly));
        if let Some(handler) = input.event_handler(ACTIVATE) {
            element = element.on(handler.direct_listener(EventType::Click));
        }
        Ok(element)
    })?;
    Ok(registry)
}
