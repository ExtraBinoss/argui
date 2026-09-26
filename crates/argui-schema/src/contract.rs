//! Serializable native contract for built-in and custom schema registries.

use serde::Serialize;

use crate::SchemaRegistry;

/// Complete contract consumed by the JSX type generator.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeContract {
    /// Stable hash of all registered schema metadata.
    pub abi_hash: String,
    /// Native primitives in numeric type-ID order.
    pub natives: Vec<NativeContractType>,
}

/// One native primitive and its public properties and events.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeContractType {
    /// Native type ID used by the host protocol.
    pub id: u32,
    /// Public TSX tag name.
    pub name: String,
    /// Property declarations.
    pub properties: Vec<NativeContractProperty>,
    /// Event declarations.
    pub events: Vec<NativeContractEvent>,
}

/// One public property declaration for generated JSX types.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeContractProperty {
    /// Property ID used by the host protocol.
    pub id: u16,
    /// Public camelCase property name.
    pub name: String,
    /// Canonical structured value type.
    pub value_type: String,
    /// Whether the runtime owns this value.
    pub read_only: bool,
    /// Whether construction requires the value.
    pub required: bool,
    /// Debug representation of the default, when supplied.
    pub default: Option<String>,
    /// Closed accepted string values, or empty for an open value.
    pub allowed_values: Vec<String>,
}

/// One public native event declaration for generated JSX types.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeContractEvent {
    /// Event ID used by the host protocol.
    pub id: u16,
    /// Public camelCase callback name.
    pub name: String,
    /// Scalar event payload type, when present.
    pub payload: Option<String>,
    /// Canonical event kind sent by native and web hosts.
    pub event_type: &'static str,
}

impl SchemaRegistry {
    /// Exports every registered primitive, including custom registrations, as
    /// the canonical serializable JSX contract.
    ///
    /// Returns metadata in stable numeric type-ID order.
    #[must_use]
    pub fn contract(&self) -> NativeContract {
        NativeContract {
            abi_hash: self.abi_hash().to_string(),
            natives: self
                .schemas()
                .map(|schema| NativeContractType {
                    id: schema.id.raw(),
                    name: schema.name.as_str().to_owned(),
                    properties: schema
                        .properties
                        .iter()
                        .map(|property| NativeContractProperty {
                            id: property.id.raw(),
                            name: property.name.as_str().to_owned(),
                            value_type: format!("{:?}", property.value_type),
                            read_only: property.read_only,
                            required: property.required,
                            default: property.default.as_ref().map(|value| format!("{value:?}")),
                            allowed_values: property.allowed_values.clone(),
                        })
                        .collect(),
                    events: schema
                        .events
                        .iter()
                        .map(|event| NativeContractEvent {
                            id: event.id.raw(),
                            name: event.name.as_str().to_owned(),
                            payload: event.payload.map(|value| format!("{value:?}")),
                            event_type: event.event_type.wire_name(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}
