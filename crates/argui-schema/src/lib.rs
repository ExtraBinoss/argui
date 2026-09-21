//! Canonical, DSL-independent metadata and adapters for Argui native primitives.

mod adapter;
pub mod builtin;
mod error;
mod id;
mod metadata;
mod property_motion;
mod registry;
mod value;

pub use adapter::{NativeAdapter, NativeElementInput, NativeEventValue, NativeSlotValue};
pub use argui_assets::{AssetHandle, AssetKey, AssetRecord, AssetRegistry};
pub use error::SchemaError;
pub use id::{EventId, NativeTypeId, PropertyId, SlotId, StylePartId, VariantId};
pub use metadata::{
    EventSchema, NativeSchema, PropertySchema, SlotArity, SlotSchema, StylePartSchema,
    VariantSchema,
};
pub use property_motion::{
    PropertyAnimation, PropertyMotionKey, PropertyMotionStore, StateTransitionPolicy,
};
pub use registry::SchemaRegistry;
pub use value::{SchemaValue, ValueType};
