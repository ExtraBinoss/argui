use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use argui_core::Name;
use argui_paint::{EffectId, EffectInstance, EffectValue};
use argui_shader::{ShaderParameterMetadata, validate_effect_source};

use crate::RendererError;

/// Spatial dependency declared by a custom GPU effect for damage tracking.
///
/// Built-in effects already expose conservative bounds to the renderer. Custom
/// shaders default to [`Self::Unbounded`] until their author confirms that all
/// texture reads stay inside the layer allocation derived from layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EffectDamage {
    /// Any scene change may affect the shader output, so partial repainting is unsafe.
    #[default]
    Unbounded,
    /// Every texture read stays inside the effect layer's allocated bounds.
    Bounded,
}

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
    /// Returns the number of 32-bit words in this parameter's packed representation.
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectParameter {
    pub name: Name,
    pub parameter_type: EffectParameterType,
}

impl EffectParameter {
    /// Creates a named parameter with its expected value type.
    /// * `name` — shader-visible parameter name; `parameter_type` — expected value type.
    #[must_use]
    pub fn new(name: impl Into<Name>, parameter_type: EffectParameterType) -> Self {
        Self {
            name: name.into(),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectPassDefinition {
    pub name: Name,
    pub wgsl: Arc<str>,
    pub inputs: Arc<[EffectInput]>,
    pub scale_divisor: u32,
}

impl EffectPassDefinition {
    /// Creates a fragment-shader pass that reads the source image.
    /// Declares the input images consumed by this pass.
    /// * `name` — pass identifier; `wgsl` — WGSL fragment-shader source.
    #[must_use]
    pub fn fragment(name: impl Into<Name>, wgsl: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            wgsl: wgsl.into(),
            inputs: Arc::from([EffectInput::Source]),
            scale_divisor: 1,
        }
    }

    /// Sets the image inputs consumed by this pass.
    /// * `inputs` — image inputs read by this pass.
    #[must_use]
    pub fn inputs(mut self, inputs: impl Into<Arc<[EffectInput]>>) -> Self {
        self.inputs = inputs.into();
        self
    }

    /// Sets the downsampling divisor for this pass; zero is treated as one.
    /// * `divisor` — divisor applied to the pass resolution.
    #[must_use]
    pub const fn downsampled(mut self, divisor: u32) -> Self {
        self.scale_divisor = if divisor == 0 { 1 } else { divisor };
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDefinition {
    pub id: EffectId,
    pub revision: u64,
    pub parameters: Arc<[EffectParameter]>,
    pub passes: Arc<[EffectPassDefinition]>,
    /// Spatial dependency used to propagate scene damage through this effect.
    pub damage: EffectDamage,
}

impl EffectDefinition {
    /// Creates an effect definition from its identifier, ordered parameters, and passes.
    /// * `id` — effect identifier; `parameters` — ordered parameter schema; `passes` — render passes.
    #[must_use]
    pub fn new(
        id: EffectId,
        parameters: impl Into<Arc<[EffectParameter]>>,
        passes: impl Into<Arc<[EffectPassDefinition]>>,
    ) -> Self {
        Self {
            id,
            revision: 1,
            parameters: parameters.into(),
            passes: passes.into(),
            damage: EffectDamage::Unbounded,
        }
    }

    /// Sets the monotonically increasing source/schema revision.
    ///
    /// * `revision` — non-zero revision assigned by the definition owner.
    #[must_use]
    pub const fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    /// Declares how scene damage propagates through this effect.
    ///
    /// Use [`EffectDamage::Bounded`] only when every pass samples within the
    /// layer allocation. The allocation already includes the dynamic expansion
    /// carried by [`EffectInstance`].
    ///
    /// * `damage` — conservative spatial dependency of the shader passes.
    #[must_use]
    pub const fn damage(mut self, damage: EffectDamage) -> Self {
        self.damage = damage;
        self
    }

    /// Validates identifiers, parameter names, pass definitions, and shader source.
    ///
    /// # Errors
    /// Returns a renderer error if the definition is malformed or shader source is invalid.
    pub fn validate(&self) -> Result<(), RendererError> {
        if self.id.as_str().is_empty() || !self.id.as_str().contains('.') {
            return Err(RendererError::InvalidEffectDefinition(format!(
                "effect id '{}' must be a non-empty namespaced name",
                self.id.as_str()
            )));
        }
        if self.revision == 0 {
            return Err(RendererError::InvalidEffectDefinition(format!(
                "effect '{}' has revision zero",
                self.id.as_str()
            )));
        }
        if self.passes.is_empty() {
            return Err(RendererError::InvalidEffectDefinition(format!(
                "effect '{}' has no passes",
                self.id.as_str()
            )));
        }
        let mut parameter_names = HashSet::with_capacity(self.parameters.len());
        for parameter in self.parameters.iter() {
            if parameter.name.as_str().is_empty() || !parameter_names.insert(parameter.name.clone())
            {
                return Err(RendererError::InvalidEffectDefinition(format!(
                    "effect '{}' has an empty or duplicate parameter name",
                    self.id.as_str()
                )));
            }
        }
        let mut pass_names = HashSet::with_capacity(self.passes.len());
        let shader_parameters = self
            .parameters
            .iter()
            .map(|parameter| {
                ShaderParameterMetadata::new(
                    parameter.name.clone(),
                    parameter.parameter_type.words(),
                )
            })
            .collect::<Vec<_>>();
        for pass in self.passes.iter() {
            if pass.name.as_str().is_empty() {
                return Err(RendererError::InvalidEffectDefinition(format!(
                    "effect '{}' has an unnamed pass",
                    self.id.as_str()
                )));
            }
            if !pass_names.insert(pass.name.clone()) {
                return Err(RendererError::InvalidEffectDefinition(format!(
                    "effect '{}' has duplicate pass '{}'",
                    self.id.as_str(),
                    pass.name
                )));
            }
            if pass.scale_divisor == 0 {
                return Err(RendererError::InvalidEffectDefinition(format!(
                    "effect '{}' pass '{}' has a zero scale divisor",
                    self.id.as_str(),
                    pass.name
                )));
            }
            validate_effect_source(
                format!("effect://{}/{}", self.id.as_str(), pass.name),
                &pass.wgsl,
                &shader_parameters,
            )
            .map_err(|error| RendererError::InvalidShader(error.to_string()))?;
        }
        Ok(())
    }

    /// Checks that an instance's ordered arguments match this definition's schema.
    ///
    /// # Errors
    /// Returns a renderer error if argument count, names, or value types do not match.
    pub fn validate_instance(&self, instance: &EffectInstance) -> Result<(), RendererError> {
        if instance.parameters.len() != self.parameters.len() {
            return Err(RendererError::InvalidEffectParameters {
                effect: self.id.clone(),
                message: format!(
                    "expected {} named values, received {}",
                    self.parameters.len(),
                    instance.parameters.len()
                ),
            });
        }
        for (schema, argument) in self.parameters.iter().zip(&instance.parameters) {
            if argument.name != schema.name {
                return Err(RendererError::InvalidEffectParameters {
                    effect: self.id.clone(),
                    message: format!(
                        "expected parameter '{}', received '{}'",
                        schema.name, argument.name
                    ),
                });
            }
            if !schema.parameter_type.accepts(&argument.value) {
                return Err(RendererError::InvalidEffectParameters {
                    effect: self.id.clone(),
                    message: format!("parameter '{}' has the wrong type", schema.name),
                });
            }
        }
        Ok(())
    }

    /// Returns the total packed word count for all parameters.
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
    /// Creates a registry after validating every definition and checking identifiers are unique.
    ///
    /// # Errors
    /// Returns a renderer error if a definition is invalid or an identifier is duplicated.
    /// * `definitions` — effect definitions to validate and register.
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
    ///
    /// # Errors
    /// Returns a renderer error if the definition is invalid or its identifier is already registered.
    pub fn with_definition(mut self, definition: EffectDefinition) -> Result<Self, RendererError> {
        definition.validate()?;
        let id = definition.id.clone();
        if self.0.indices.contains_key(&id) {
            return Err(RendererError::DuplicateEffect(id));
        }
        let registry = Arc::make_mut(&mut self.0);
        registry.indices.insert(id, registry.definitions.len());
        registry.definitions.push(definition);
        Ok(self)
    }

    /// Transactionally replaces an existing definition with a newer validated revision.
    ///
    /// Validation completes before copy-on-write state changes, so errors leave this
    /// registry and every clone untouched.
    ///
    /// * `definition` — replacement with the same ID and a greater revision.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid definition, missing ID, or stale revision.
    pub fn with_replacement(mut self, definition: EffectDefinition) -> Result<Self, RendererError> {
        definition.validate()?;
        let Some(index) = self.0.indices.get(&definition.id).copied() else {
            return Err(RendererError::MissingEffect(definition.id));
        };
        let current = &self.0.definitions[index];
        if definition.revision <= current.revision {
            return Err(RendererError::InvalidEffectDefinition(format!(
                "effect '{}' replacement revision {} must be greater than {}",
                definition.id, definition.revision, current.revision
            )));
        }
        Arc::make_mut(&mut self.0).definitions[index] = definition;
        Ok(self)
    }

    /// Removes a definition while preserving registration order for the survivors.
    ///
    /// * `id` — effect definition to remove.
    #[must_use]
    pub fn without_definition(mut self, id: &EffectId) -> Self {
        if !self.0.indices.contains_key(id) {
            return self;
        }
        let registry = Arc::make_mut(&mut self.0);
        registry
            .definitions
            .retain(|definition| &definition.id != id);
        registry.indices.clear();
        for (index, definition) in registry.definitions.iter().enumerate() {
            registry.indices.insert(definition.id.clone(), index);
        }
        self
    }

    /// Returns the definition registered for `id`, if present.
    #[must_use]
    pub fn get(&self, id: &EffectId) -> Option<&EffectDefinition> {
        self.0
            .indices
            .get(id)
            .map(|index| &self.0.definitions[*index])
    }

    /// Returns all definitions in registration order.
    #[must_use]
    pub fn definitions(&self) -> &[EffectDefinition] {
        &self.0.definitions
    }

    /// Returns whether the registry contains no effect definitions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.definitions.is_empty()
    }

    /// Returns the largest packed parameter schema in 32-bit words.
    #[must_use]
    pub fn maximum_parameter_words(&self) -> usize {
        self.0
            .definitions
            .iter()
            .map(EffectDefinition::parameter_words)
            .max()
            .unwrap_or(1)
    }
}
