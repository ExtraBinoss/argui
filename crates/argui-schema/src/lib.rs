//! Canonical, DSL-independent metadata and adapters for Argui native primitives.

mod adapter;
pub mod builtin;
mod error;
mod id;
mod metadata;
mod registry;
mod value;

pub use adapter::{NativeAdapter, NativeElementInput, NativeEventValue, NativeSlotValue};
pub use error::SchemaError;
pub use id::{EventId, NativeTypeId, PropertyId, SlotId, StylePartId, VariantId};
pub use metadata::{
    EventSchema, NativeSchema, PropertySchema, SlotArity, SlotSchema, StylePartSchema,
    VariantSchema,
};
pub use registry::SchemaRegistry;
pub use value::{SchemaValue, ValueType};
