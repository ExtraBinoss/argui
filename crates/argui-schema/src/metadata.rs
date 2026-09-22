use argui_core::Name;

use crate::{
    EventId, NativeTypeId, PropertyId, SchemaValue, SlotId, StylePartId, ValueType, VariantId,
};

/// Engine interaction value sampled for a native output property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationKind {
    Hover,
    Pressed,
    Focused,
    FocusVisible,
    PointerX,
    PointerY,
    PointerGlobalX,
    PointerGlobalY,
    PressedX,
    PressedY,
    ScrollX,
    ScrollY,
    ViewportWidth,
    ViewportHeight,
    ContentWidth,
    ContentHeight,
}

impl ObservationKind {
    /// Returns the schema value type supplied by this observation.
    #[must_use]
    pub const fn value_type(self) -> ValueType {
        match self {
            Self::Hover | Self::Pressed | Self::Focused | Self::FocusVisible => ValueType::Bool,
            Self::PointerX
            | Self::PointerY
            | Self::PointerGlobalX
            | Self::PointerGlobalY
            | Self::PressedX
            | Self::PressedY
            | Self::ScrollX
            | Self::ScrollY
            | Self::ViewportWidth
            | Self::ViewportHeight
            | Self::ContentWidth
            | Self::ContentHeight => ValueType::Dimension,
        }
    }
}

/// Whether a child slot accepts one element or a sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotArity {
    Optional,
    Required,
    Many,
}

/// Declarative property metadata shared by all DSL tooling and runtimes.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertySchema {
    pub id: PropertyId,
    pub name: Name,
    pub value_type: ValueType,
    pub required: bool,
    /// Whether the value is supplied by the engine and cannot be assigned by authored UI.
    pub read_only: bool,
    /// Engine observation supplying this output property's value, when applicable.
    pub observation: Option<ObservationKind>,
    pub default: Option<SchemaValue>,
    pub change_event: Option<EventId>,
    /// Whether the property supports declarative visual interpolation.
    pub animatable: bool,
    pub documentation: String,
}

impl PropertySchema {
    /// Creates property metadata.
    ///
    /// * `id` — stable property ID within its native type.
    /// * `name` — public declarative name.
    /// * `value_type` — accepted runtime value type.
    /// * `documentation` — user-facing documentation.
    #[must_use]
    pub fn new(
        id: PropertyId,
        name: impl Into<Name>,
        value_type: ValueType,
        documentation: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            value_type,
            required: false,
            read_only: false,
            observation: None,
            default: None,
            change_event: None,
            animatable: true,
            documentation: documentation.into(),
        }
    }

    /// Marks the property as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Marks an engine-observed property as readable but not assignable.
    #[must_use]
    pub const fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Binds a read-only property to a generic engine interaction observation.
    ///
    /// * `kind` — interaction value to sample during rendering and event handling.
    #[must_use]
    pub const fn observed(mut self, kind: ObservationKind) -> Self {
        self.read_only = true;
        self.observation = Some(kind);
        self
    }

    /// Assigns a typed default value.
    ///
    /// * `value` — default used when the property is absent.
    #[must_use]
    pub fn default_value(mut self, value: SchemaValue) -> Self {
        self.default = Some(value);
        self
    }

    /// Declares the event which supplies updates for a two-way binding.
    ///
    /// * `event` — event whose payload has the same schema type as this property.
    #[must_use]
    pub const fn changed_by(mut self, event: EventId) -> Self {
        self.change_event = Some(event);
        self
    }

    /// Rejects declarative animation for a structural property that cannot be sampled independently.
    #[must_use]
    pub const fn not_animatable(mut self) -> Self {
        self.animatable = false;
        self
    }
}

/// Declarative event metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct EventSchema {
    pub id: EventId,
    pub name: Name,
    pub payload: Option<ValueType>,
    pub event_type: argui_ui::EventType,
    pub documentation: String,
}

impl EventSchema {
    /// Creates event metadata bound to one engine event kind.
    ///
    /// * `id` — stable event ID within its native type.
    /// * `name` — public declarative event name.
    /// * `event_type` — engine event delivered to the opaque handler.
    /// * `documentation` — user-facing documentation.
    #[must_use]
    pub fn new(
        id: EventId,
        name: impl Into<Name>,
        event_type: argui_ui::EventType,
        documentation: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            payload: None,
            event_type,
            documentation: documentation.into(),
        }
    }

    /// Declares the domain value emitted by this event.
    ///
    /// * `payload` — schema type extracted from the matching engine event.
    #[must_use]
    pub const fn payload(mut self, payload: ValueType) -> Self {
        self.payload = Some(payload);
        self
    }
}

/// Declarative child-slot metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct SlotSchema {
    pub id: SlotId,
    pub name: Name,
    pub arity: SlotArity,
    pub documentation: String,
}

/// Declarative named variant metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct VariantSchema {
    pub id: VariantId,
    pub name: Name,
    pub documentation: String,
}

/// Declarative style-part metadata exposed to themes and inspection tools.
#[derive(Clone, Debug, PartialEq)]
pub struct StylePartSchema {
    pub id: StylePartId,
    pub name: Name,
    pub documentation: String,
}

/// Canonical schema for one native primitive or behavior adapter.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeSchema {
    pub id: NativeTypeId,
    pub name: Name,
    pub documentation: String,
    pub properties: Vec<PropertySchema>,
    pub events: Vec<EventSchema>,
    pub slots: Vec<SlotSchema>,
    pub variants: Vec<VariantSchema>,
    pub style_parts: Vec<StylePartSchema>,
    /// The adapter consumes a compiler-mounted keyed virtual row window.
    pub virtual_window: bool,
}

impl NativeSchema {
    /// Creates an empty schema which can be extended with builder methods.
    ///
    /// * `id` — stable native type ID.
    /// * `name` — public schema name.
    /// * `documentation` — user-facing documentation.
    #[must_use]
    pub fn new(id: NativeTypeId, name: impl Into<Name>, documentation: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            documentation: documentation.into(),
            properties: Vec::new(),
            events: Vec::new(),
            slots: Vec::new(),
            variants: Vec::new(),
            style_parts: Vec::new(),
            virtual_window: false,
        }
    }

    /// Appends a property definition.
    #[must_use]
    pub fn property(mut self, property: PropertySchema) -> Self {
        self.properties.push(property);
        self
    }

    /// Appends an event definition.
    #[must_use]
    pub fn event(mut self, event: EventSchema) -> Self {
        self.events.push(event);
        self
    }

    /// Appends a child-slot definition.
    #[must_use]
    pub fn slot(mut self, slot: SlotSchema) -> Self {
        self.slots.push(slot);
        self
    }

    /// Appends a named variant definition.
    #[must_use]
    pub fn variant(mut self, variant: VariantSchema) -> Self {
        self.variants.push(variant);
        self
    }

    /// Appends a style-part definition.
    #[must_use]
    pub fn style_part(mut self, part: StylePartSchema) -> Self {
        self.style_parts.push(part);
        self
    }

    /// Marks this adapter as consuming a compiler-mounted virtual row window.
    #[must_use]
    pub const fn virtual_window(mut self) -> Self {
        self.virtual_window = true;
        self
    }
}
