use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use argui_paint::{EffectId, EffectInstance, EffectValue};

use crate::{RendererError, effect::validated_custom_source};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectParameterType {
    F32,
    I32,
    U32,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
    Color,
    LogicalPixels,
}

impl EffectParameterType {
    #[must_use]
    pub const fn words(self) -> usize {
        match self {
            Self::F32 | Self::I32 | Self::U32 | Self::Bool | Self::LogicalPixels => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 | Self::Color => 4,
            Self::Mat3 => 9,
            Self::Mat4 => 16,
        }
    }

    fn accepts(self, value: &EffectValue) -> bool {
        matches!(
            (self, value),
            (Self::F32, EffectValue::F32(_))
                | (Self::I32, EffectValue::I32(_))
                | (Self::U32, EffectValue::U32(_))
                | (Self::Bool, EffectValue::Bool(_))
                | (Self::Vec2, EffectValue::Vec2(_))
                | (Self::Vec3, EffectValue::Vec3(_))
                | (Self::Vec4, EffectValue::Vec4(_))
                | (Self::Mat3, EffectValue::Mat3(_))
                | (Self::Mat4, EffectValue::Mat4(_))
                | (Self::Color, EffectValue::Color(_))
                | (Self::LogicalPixels, EffectValue::LogicalPixels(_))
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectParameter {
    pub name: &'static str,
    pub parameter_type: EffectParameterType,
}

impl EffectParameter {
    #[must_use]
    pub const fn new(name: &'static str, parameter_type: EffectParameterType) -> Self {
        Self {
            name,
            parameter_type,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectInput {
    Source,
    Backdrop,
    Mask,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectPassDefinition {
    pub name: &'static str,
    pub wgsl: &'static str,
    pub inputs: &'static [EffectInput],
    pub scale_divisor: u32,
}

impl EffectPassDefinition {
    #[must_use]
    pub const fn fragment(name: &'static str, wgsl: &'static str) -> Self {
        Self {
            name,
            wgsl,
            inputs: &[EffectInput::Source],
            scale_divisor: 1,
        }
    }

    #[must_use]
    pub const fn inputs(mut self, inputs: &'static [EffectInput]) -> Self {
        self.inputs = inputs;
        self
    }

    #[must_use]
    pub const fn downsampled(mut self, divisor: u32) -> Self {
        self.scale_divisor = if divisor == 0 { 1 } else { divisor };
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDefinition {
    pub id: EffectId,
    pub parameters: &'static [EffectParameter],
    pub passes: &'static [EffectPassDefinition],
}

impl EffectDefinition {
    #[must_use]
    pub const fn new(
        id: EffectId,
        parameters: &'static [EffectParameter],
        passes: &'static [EffectPassDefinition],
    ) -> Self {
        Self {
            id,
            parameters,
            passes,
        }
    }

    pub fn validate(&self) -> Result<(), RendererError> {
        if self.id.0.is_empty() || !self.id.0.contains('.') {
            return Err(RendererError::InvalidEffectDefinition(format!(
                "effect id '{}' must be a non-empty namespaced name",
                self.id.0
            )));
        }
        if self.passes.is_empty() {
            return Err(RendererError::InvalidEffectDefinition(format!(
                "effect '{}' has no passes",
                self.id.0
            )));
        }
        let mut parameter_names = HashSet::with_capacity(self.parameters.len());
        for parameter in self.parameters {
            if parameter.name.is_empty() || !parameter_names.insert(parameter.name) {
                return Err(RendererError::InvalidEffectDefinition(format!(
                    "effect '{}' has an empty or duplicate parameter name",
                    self.id.0
                )));
            }
        }
        let mut pass_names = HashSet::with_capacity(self.passes.len());
        for pass in self.passes {
            if pass.name.is_empty() {
                return Err(RendererError::InvalidEffectDefinition(format!(
                    "effect '{}' has an unnamed pass",
                    self.id.0
                )));
            }
            if !pass_names.insert(pass.name) {
                return Err(RendererError::InvalidEffectDefinition(format!(
                    "effect '{}' has duplicate pass '{}'",
                    self.id.0, pass.name
                )));
            }
            if pass.scale_divisor == 0 {
                return Err(RendererError::InvalidEffectDefinition(format!(
                    "effect '{}' pass '{}' has a zero scale divisor",
                    self.id.0, pass.name
                )));
            }
            validated_custom_source(pass.wgsl)?;
        }
        Ok(())
    }

    pub fn validate_instance(&self, instance: &EffectInstance) -> Result<(), RendererError> {
        if instance.parameters.len() != self.parameters.len() {
            return Err(RendererError::InvalidEffectParameters {
                effect: self.id.0,
                message: format!(
                    "expected {} named values, received {}",
                    self.parameters.len(),
                    instance.parameters.len()
                ),
            });
        }
        for (schema, argument) in self.parameters.iter().zip(&instance.parameters) {
            if schema.name != argument.name {
                return Err(RendererError::InvalidEffectParameters {
                    effect: self.id.0,
                    message: format!(
                        "expected parameter '{}', received '{}'",
                        schema.name, argument.name
                    ),
                });
            }
            if !schema.parameter_type.accepts(&argument.value) {
                return Err(RendererError::InvalidEffectParameters {
                    effect: self.id.0,
                    message: format!("parameter '{}' has the wrong type", schema.name),
                });
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn parameter_words(&self) -> usize {
        self.parameters
            .iter()
            .map(|parameter| parameter.parameter_type.words())
            .sum()
    }
}

#[derive(Clone, Debug, Default)]
pub struct EffectRegistry(Arc<EffectRegistryInner>);

#[derive(Clone, Debug, Default)]
struct EffectRegistryInner {
    definitions: Vec<EffectDefinition>,
    indices: HashMap<EffectId, usize>,
}

impl EffectRegistry {
    pub fn new(
        definitions: impl IntoIterator<Item = EffectDefinition>,
    ) -> Result<Self, RendererError> {
        let mut registry = Self::default();
        for definition in definitions {
            registry = registry.with_definition(definition)?;
        }
        Ok(registry)
    }

    /// Adds one definition without revalidating existing immutable definitions.
    pub fn with_definition(mut self, definition: EffectDefinition) -> Result<Self, RendererError> {
        definition.validate()?;
        let id = definition.id;
        if self.0.indices.contains_key(&id) {
            return Err(RendererError::DuplicateEffect(id.0));
        }
        let registry = Arc::make_mut(&mut self.0);
        registry.indices.insert(id, registry.definitions.len());
        registry.definitions.push(definition);
        Ok(self)
    }

    #[must_use]
    pub fn get(&self, id: EffectId) -> Option<&EffectDefinition> {
        self.0
            .indices
            .get(&id)
            .map(|index| &self.0.definitions[*index])
    }

    #[must_use]
    pub fn definitions(&self) -> &[EffectDefinition] {
        &self.0.definitions
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.definitions.is_empty()
    }
}
