//! Exports the canonical native schema for JavaScript code generation.

use argui_schema::builtin;
use serde_json::{Value, json};

/// Converts the built-in native registry into a stable JSON contract.
///
/// # Errors
///
/// Returns a schema error if the built-in registry cannot be constructed.
fn contract() -> Result<Value, argui_schema::SchemaError> {
    let registry = builtin::registry()?;
    let natives = registry
        .schemas()
        .map(|schema| {
            json!({
                "id": schema.id.raw(),
                "name": schema.name.as_str(),
                "properties": schema.properties.iter().map(|property| json!({
                    "id": property.id.raw(),
                    "name": property.name.as_str(),
                    "valueType": format!("{:?}", property.value_type),
                    "readOnly": property.read_only,
                    "required": property.required,
                    "default": property.default.as_ref().map(|value| format!("{value:?}")),
                })).collect::<Vec<_>>(),
                "events": schema.events.iter().map(|event| json!({
                    "id": event.id.raw(),
                    "name": event.name.as_str(),
                    "payload": event.payload.map(|value| format!("{value:?}")),
                })).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({"abiHash": registry.abi_hash().to_string(), "natives": natives}))
}

/// Writes the schema contract to stdout for the TypeScript generator.
///
/// # Errors
///
/// Returns a schema or JSON serialization error when export fails.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&contract()?)?);
    Ok(())
}
