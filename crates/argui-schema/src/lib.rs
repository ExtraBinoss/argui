//! Canonical metadata and adapters for Argui native primitives.

mod adapter;
pub mod backdrop_filter;
pub mod builtin;
mod error;
mod id;
mod metadata;
mod native_cache;
mod property_motion;
mod registry;
mod value;
mod virtual_viewport;

pub use adapter::{NativeAdapter, NativeElementInput, NativeEventValue, NativeSlotValue};
pub use argui_assets::{AssetHandle, AssetKey, AssetRecord, AssetRegistry};
pub use error::SchemaError;
pub use id::{EventId, NativeTypeId, PropertyId, SlotId, StylePartId, VariantId};
pub use metadata::{
    EventSchema, NativeSchema, ObservationKind, PropertySchema, SlotArity, SlotSchema,
    StylePartSchema, VariantSchema,
};
pub use native_cache::NativeElementCache;
pub use property_motion::{
    PropertyAnimation, PropertyMotionKey, PropertyMotionStore, StateTransitionPolicy,
};
pub use registry::SchemaRegistry;
pub use value::{SchemaValue, ValueType};
pub use virtual_viewport::VirtualViewportStore;
