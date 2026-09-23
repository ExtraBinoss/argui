use argui_ui::Element;

use crate::{EventId, PropertyId, SchemaError, SchemaValue, SlotId};

/// Opaque presentation handler supplied for one schema-declared event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeEventValue {
    pub id: EventId,
    pub handler: argui_ui::EventHandler,
}

impl NativeEventValue {
    /// Creates an event value for a schema event and runtime-owned handler.
    ///
    /// * `id` — stable event ID declared by the native schema.
    /// * `handler` — opaque handler registered by the presentation runtime.
    #[must_use]
    pub const fn new(id: EventId, handler: argui_ui::EventHandler) -> Self {
        Self { id, handler }
    }
}

/// Child elements supplied for a specific schema slot.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeSlotValue {
    pub id: SlotId,
    pub elements: Vec<Element>,
}

impl NativeSlotValue {
    /// Creates a populated slot value.
    ///
    /// * `id` — stable slot ID declared by the native schema.
    /// * `elements` — child elements assigned to the slot.
    #[must_use]
    pub fn new(id: SlotId, elements: impl IntoIterator<Item = Element>) -> Self {
        Self {
            id,
            elements: elements.into_iter().collect(),
        }
    }
}

/// Validated property and slot values passed to a native adapter.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeElementInput {
    pub properties: Vec<(PropertyId, SchemaValue)>,
    pub events: Vec<NativeEventValue>,
    pub slots: Vec<NativeSlotValue>,
    /// Compiler/runtime-owned virtual row measurements, present only for VirtualWindow.
    pub virtual_list: Option<argui_ui::VirtualList>,
}

impl NativeElementInput {
    /// Creates an empty native input.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            properties: Vec::new(),
            events: Vec::new(),
            slots: Vec::new(),
            virtual_list: None,
        }
    }

    /// Appends a property value.
    ///
    /// * `id` — property ID declared by the native schema.
    /// * `value` — typed property value.
    #[must_use]
    pub fn property(mut self, id: PropertyId, value: SchemaValue) -> Self {
        self.properties.push((id, value));
        self
    }

    /// Appends an opaque native event handler.
    ///
    /// * `event` — schema event and presentation-owned handler identity.
    #[must_use]
    pub fn event(mut self, event: NativeEventValue) -> Self {
        self.events.push(event);
        self
    }

    /// Appends a child slot value.
    ///
    /// * `slot` — slot and its children.
    #[must_use]
    pub fn slot(mut self, slot: NativeSlotValue) -> Self {
        self.slots.push(slot);
        self
    }

    /// Attaches a retained virtual list whose measured rows match this input.
    ///
    /// `list` is the compiler/runtime-owned window and measurement state.
    /// Returns the input with that list available to its native adapter.
    #[must_use]
    pub fn virtual_list(mut self, list: argui_ui::VirtualList) -> Self {
        self.virtual_list = Some(list);
        self
    }

    /// Returns a supplied property value by stable ID.
    ///
    /// * `id` — property ID to resolve.
    #[must_use]
    pub fn get(&self, id: PropertyId) -> Option<&SchemaValue> {
        self.properties
            .iter()
            .find_map(|(candidate, value)| (*candidate == id).then_some(value))
    }

    /// Returns a supplied handler by stable event ID.
    ///
    /// * `id` — event ID to resolve.
    #[must_use]
    pub fn event_handler(&self, id: EventId) -> Option<argui_ui::EventHandler> {
        self.events
            .iter()
            .find_map(|event| (event.id == id).then_some(event.handler))
    }

    /// Returns the children supplied for a slot.
    ///
    /// * `id` — slot ID to resolve.
    #[must_use]
    pub fn children(&self, id: SlotId) -> &[Element] {
        self.slots
            .iter()
            .find_map(|slot| (slot.id == id).then_some(slot.elements.as_slice()))
            .unwrap_or_default()
    }
}

/// Runtime construction boundary implemented by native primitives and behaviors.
pub trait NativeAdapter: Send + Sync {
    /// Builds an Argui element from schema-validated input.
    ///
    /// * `input` — properties and slots validated against the registered schema.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaError::Adapter`] when values are semantically invalid for the adapter.
    fn construct(&self, input: &NativeElementInput) -> Result<Element, SchemaError>;
}

impl<F> NativeAdapter for F
where
    F: Fn(&NativeElementInput) -> Result<Element, SchemaError> + Send + Sync,
{
    fn construct(&self, input: &NativeElementInput) -> Result<Element, SchemaError> {
        self(input)
    }
}
