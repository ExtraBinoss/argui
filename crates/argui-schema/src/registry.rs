use std::{collections::HashMap, sync::Arc};

use argui_core::Name;
use argui_ui::Element;

use crate::{
    NativeAdapter, NativeElementInput, NativeSchema, NativeTypeId, SchemaError, SlotArity,
};

struct RegisteredNative {
    schema: NativeSchema,
    adapter: Arc<dyn NativeAdapter>,
}

/// Validated registry that owns canonical schemas and their native construction adapters.
#[derive(Default)]
pub struct SchemaRegistry {
    natives: HashMap<NativeTypeId, RegisteredNative>,
    names: HashMap<Name, NativeTypeId>,
}

impl SchemaRegistry {
    /// Creates an empty schema registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a schema and its construction adapter atomically.
    ///
    /// * `schema` — canonical metadata to expose to every consumer.
    /// * `adapter` — native construction implementation.
    ///
    /// # Errors
    ///
    /// Returns a validation error for duplicate IDs/names or mismatched defaults.
    pub fn register(
        &mut self,
        schema: NativeSchema,
        adapter: impl NativeAdapter + 'static,
    ) -> Result<(), SchemaError> {
        self.validate_schema(&schema)?;
        if self.natives.contains_key(&schema.id) {
            return Err(SchemaError::DuplicateNativeId(schema.id));
        }
        if self.names.contains_key(&schema.name) {
            return Err(SchemaError::DuplicateNativeName(schema.name.clone()));
        }
        self.names.insert(schema.name.clone(), schema.id);
        self.natives.insert(
            schema.id,
            RegisteredNative {
                schema,
                adapter: Arc::new(adapter),
            },
        );
        Ok(())
    }

    /// Returns canonical metadata for a native type ID.
    ///
    /// * `id` — stable native type ID.
    #[must_use]
    pub fn schema(&self, id: NativeTypeId) -> Option<&NativeSchema> {
        self.natives.get(&id).map(|entry| &entry.schema)
    }

    /// Resolves a native schema by public name.
    ///
    /// * `name` — public schema name.
    #[must_use]
    pub fn schema_named(&self, name: &str) -> Option<&NativeSchema> {
        self.names
            .get(name)
            .and_then(|id| self.natives.get(id))
            .map(|entry| &entry.schema)
    }

    /// Iterates over canonical schemas in stable numeric-ID order.
    pub fn schemas(&self) -> impl Iterator<Item = &NativeSchema> {
        let mut schemas = self
            .natives
            .values()
            .map(|entry| &entry.schema)
            .collect::<Vec<_>>();
        schemas.sort_unstable_by_key(|schema| schema.id);
        schemas.into_iter()
    }

    /// Validates input and constructs an element through the registered adapter.
    ///
    /// * `id` — native type to construct.
    /// * `input` — typed property and child-slot values.
    ///
    /// # Errors
    ///
    /// Returns an error for unknown IDs, invalid input, or adapter rejection.
    pub fn construct(
        &self,
        id: NativeTypeId,
        input: &NativeElementInput,
    ) -> Result<Element, SchemaError> {
        let registered = self
            .natives
            .get(&id)
            .ok_or(SchemaError::UnknownNative(id))?;
        validate_input(&registered.schema, input)?;
        registered.adapter.construct(input)
    }

    fn validate_schema(&self, schema: &NativeSchema) -> Result<(), SchemaError> {
        validate_members(
            &schema.name,
            "property",
            schema
                .properties
                .iter()
                .map(|property| (property.id.raw(), &property.name)),
        )?;
        validate_members(
            &schema.name,
            "event",
            schema
                .events
                .iter()
                .map(|event| (event.id.raw(), &event.name)),
        )?;
        validate_members(
            &schema.name,
            "slot",
            schema.slots.iter().map(|slot| (slot.id.raw(), &slot.name)),
        )?;
        validate_members(
            &schema.name,
            "variant",
            schema
                .variants
                .iter()
                .map(|variant| (variant.id.raw(), &variant.name)),
        )?;
        validate_members(
            &schema.name,
            "style part",
            schema.parts_iter().map(|part| (part.id.raw(), &part.name)),
        )?;
        for property in &schema.properties {
            if let Some(observation) = property.observation
                && (property.value_type != observation.value_type() || !property.read_only)
            {
                return Err(SchemaError::InvalidObservation {
                    property: property.name.clone(),
                    observation,
                    expected: observation.value_type(),
                    actual: property.value_type,
                });
            }
            if let Some(default) = &property.default
                && default.value_type() != property.value_type
            {
                return Err(SchemaError::InvalidDefault {
                    property: property.name.clone(),
                    expected: property.value_type,
                    actual: default.value_type(),
                });
            }
            if let Some(event) = property.change_event
                && !schema.events.iter().any(|candidate| {
                    candidate.id == event
                        && (candidate.payload == Some(property.value_type)
                            || (property.value_type == crate::ValueType::String
                                && candidate.payload.is_none()
                                && candidate.event_type == argui_ui::EventType::TextEdit))
                })
            {
                return Err(SchemaError::InvalidChangeEvent {
                    property: property.name.clone(),
                    event,
                });
            }
        }
        Ok(())
    }
}

impl NativeSchema {
    fn parts_iter(&self) -> impl Iterator<Item = &crate::StylePartSchema> {
        self.style_parts.iter()
    }
}

fn validate_members<'a>(
    native: &Name,
    kind: &'static str,
    members: impl Iterator<Item = (u16, &'a Name)>,
) -> Result<(), SchemaError> {
    let mut ids = HashMap::new();
    let mut names = HashMap::new();
    for (id, name) in members {
        if ids.insert(id, ()).is_some() {
            return Err(SchemaError::DuplicateMemberId {
                native: native.clone(),
                kind,
                id,
            });
        }
        if names.insert(name.clone(), ()).is_some() {
            return Err(SchemaError::DuplicateMemberName {
                native: native.clone(),
                kind,
                name: name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_input(schema: &NativeSchema, input: &NativeElementInput) -> Result<(), SchemaError> {
    let mut properties = HashMap::new();
    for (id, value) in &input.properties {
        if properties.insert(*id, ()).is_some() {
            return Err(SchemaError::DuplicateProperty(*id));
        }
        let property = schema
            .properties
            .iter()
            .find(|property| property.id == *id)
            .ok_or_else(|| SchemaError::UnknownProperty {
                native: schema.name.clone(),
                property: *id,
            })?;
        if property.read_only {
            return Err(SchemaError::ReadOnlyProperty(property.name.clone()));
        }
        if value.value_type() != property.value_type {
            return Err(SchemaError::PropertyType {
                property: property.name.clone(),
                expected: property.value_type,
                actual: value.value_type(),
            });
        }
    }
    for property in &schema.properties {
        if property.required && property.default.is_none() && !properties.contains_key(&property.id)
        {
            return Err(SchemaError::MissingProperty {
                property: property.name.clone(),
            });
        }
    }
    let mut events = HashMap::new();
    for value in &input.events {
        if events.insert(value.id, ()).is_some() {
            return Err(SchemaError::DuplicateEvent(value.id));
        }
        if !schema.events.iter().any(|event| event.id == value.id) {
            return Err(SchemaError::UnknownEvent {
                native: schema.name.clone(),
                event: value.id,
            });
        }
    }
    let mut slots = HashMap::new();
    for value in &input.slots {
        if slots.insert(value.id, ()).is_some() {
            return Err(SchemaError::DuplicateSlot(value.id));
        }
        let slot = schema
            .slots
            .iter()
            .find(|slot| slot.id == value.id)
            .ok_or_else(|| SchemaError::UnknownSlot {
                native: schema.name.clone(),
                slot: value.id,
            })?;
        let valid = match slot.arity {
            SlotArity::Optional => value.elements.len() <= 1,
            SlotArity::Required => value.elements.len() == 1,
            SlotArity::Many => true,
        };
        if !valid {
            return Err(SchemaError::SlotArity {
                slot: slot.name.clone(),
                expected: match slot.arity {
                    SlotArity::Optional => "zero or one child",
                    SlotArity::Required => "exactly one child",
                    SlotArity::Many => "any number of children",
                },
                actual: value.elements.len(),
            });
        }
    }
    for slot in &schema.slots {
        if slot.arity == SlotArity::Required && !slots.contains_key(&slot.id) {
            return Err(SchemaError::SlotArity {
                slot: slot.name.clone(),
                expected: "exactly one child",
                actual: 0,
            });
        }
    }
    Ok(())
}
