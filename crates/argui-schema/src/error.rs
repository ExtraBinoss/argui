use argui_core::Name;

use crate::{EventId, NativeTypeId, ObservationKind, PropertyId, SlotId, ValueType};

/// Schema registration, validation, or construction failure.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum SchemaError {
    #[error("native schema ID {0:?} is already registered")]
    DuplicateNativeId(NativeTypeId),
    #[error("native schema name `{0}` is already registered")]
    DuplicateNativeName(Name),
    #[error("duplicate {kind} ID {id} in native schema `{native}`")]
    DuplicateMemberId {
        native: Name,
        kind: &'static str,
        id: u16,
    },
    #[error("duplicate {kind} name `{name}` in native schema `{native}`")]
    DuplicateMemberName {
        native: Name,
        kind: &'static str,
        name: Name,
    },
    #[error("default for property `{property}` has type {actual:?}, expected {expected:?}")]
    InvalidDefault {
        property: Name,
        expected: ValueType,
        actual: ValueType,
    },
    #[error("property `{property}` uses an absent or incompatible change event {event:?}")]
    InvalidChangeEvent { property: Name, event: EventId },
    #[error(
        "property `{property}` observes {observation:?} with type {actual:?}, expected {expected:?}"
    )]
    InvalidObservation {
        property: Name,
        observation: ObservationKind,
        expected: ValueType,
        actual: ValueType,
    },
    #[error("native schema ID {0:?} is not registered")]
    UnknownNative(NativeTypeId),
    #[error("property ID {property:?} is not defined by native schema `{native}`")]
    UnknownProperty { native: Name, property: PropertyId },
    #[error("slot ID {slot:?} is not defined by native schema `{native}`")]
    UnknownSlot { native: Name, slot: SlotId },
    #[error("event ID {event:?} is not defined by native schema `{native}`")]
    UnknownEvent { native: Name, event: EventId },
    #[error("property `{property}` has type {actual:?}, expected {expected:?}")]
    PropertyType {
        property: Name,
        expected: ValueType,
        actual: ValueType,
    },
    #[error("property `{0}` is read-only")]
    ReadOnlyProperty(Name),
    #[error("required property `{property}` is missing")]
    MissingProperty { property: Name },
    #[error("property ID {0:?} was supplied more than once")]
    DuplicateProperty(PropertyId),
    #[error("slot ID {0:?} was supplied more than once")]
    DuplicateSlot(SlotId),
    #[error("event ID {0:?} was supplied more than once")]
    DuplicateEvent(EventId),
    #[error("slot `{slot}` expects {expected}, received {actual} children")]
    SlotArity {
        slot: Name,
        expected: &'static str,
        actual: usize,
    },
    #[error("native adapter rejected the input: {0}")]
    Adapter(String),
}
