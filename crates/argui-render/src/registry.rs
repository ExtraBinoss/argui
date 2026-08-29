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

#[cfg(test)]
mod tests {
    use argui_core::Color;
    use argui_paint::{EffectId, EffectInstance, EffectValue};

    use super::{
        EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition,
        EffectRegistry,
    };

    const WGSL: &str = r#"
fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> {
    return source * argui_param_f32(0u);
}
"#;
    const PARAMETERS: &[EffectParameter] =
        &[EffectParameter::new("amount", EffectParameterType::F32)];
    const PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment("main", WGSL)];

    fn definition(id: EffectId) -> EffectDefinition {
        EffectDefinition::new(id, PARAMETERS, PASSES)
    }

    #[test]
    fn registry_requires_namespaces_and_unique_ids() {
        assert!(EffectRegistry::new([definition(EffectId::new("plain"))]).is_err());
        assert!(EffectRegistry::new([definition(EffectId::new(""))]).is_err());
        let id = EffectId::new("test.effect");
        assert!(EffectRegistry::new([definition(id), definition(id)]).is_err());
        assert_eq!(
            EffectRegistry::new([definition(id)])
                .unwrap()
                .definitions()
                .len(),
            1
        );
    }

    #[test]
    fn instances_are_checked_by_parameter_name_and_type() {
        let definition = definition(EffectId::new("test.effect"));
        assert!(
            definition
                .validate_instance(&EffectInstance::new(
                    definition.id,
                    [("amount", EffectValue::F32(0.5))],
                ))
                .is_ok()
        );
        assert!(
            definition
                .validate_instance(&EffectInstance::new(
                    definition.id,
                    [("wrong", EffectValue::F32(0.5))],
                ))
                .is_err()
        );
        assert!(
            definition
                .validate_instance(&EffectInstance::new(
                    definition.id,
                    [("amount", EffectValue::U32(1))],
                ))
                .is_err()
        );
    }

    #[test]
    fn every_parameter_type_has_an_exact_storage_width_and_value_type() {
        let cases = [
            (EffectParameterType::F32, EffectValue::F32(1.0), 1),
            (EffectParameterType::I32, EffectValue::I32(-1), 1),
            (EffectParameterType::U32, EffectValue::U32(1), 1),
            (EffectParameterType::Bool, EffectValue::Bool(true), 1),
            (EffectParameterType::Vec2, EffectValue::Vec2([0.0; 2]), 2),
            (EffectParameterType::Vec3, EffectValue::Vec3([0.0; 3]), 3),
            (EffectParameterType::Vec4, EffectValue::Vec4([0.0; 4]), 4),
            (EffectParameterType::Mat3, EffectValue::Mat3([0.0; 9]), 9),
            (EffectParameterType::Mat4, EffectValue::Mat4([0.0; 16]), 16),
            (
                EffectParameterType::Color,
                EffectValue::Color(Color::WHITE),
                4,
            ),
            (
                EffectParameterType::LogicalPixels,
                EffectValue::LogicalPixels(2.0),
                1,
            ),
        ];
        for (parameter_type, value, words) in cases {
            assert_eq!(parameter_type.words(), words);
            assert!(parameter_type.accepts(&value));
            if parameter_type != EffectParameterType::U32 {
                assert!(!parameter_type.accepts(&EffectValue::U32(4)));
            }
        }
        assert_eq!(
            definition(EffectId::new("test.effect")).parameter_words(),
            1
        );
    }

    #[test]
    fn pass_builders_and_definition_validation_cover_every_invalid_shape() {
        const DUPLICATE_PARAMETERS: &[EffectParameter] = &[
            EffectParameter::new("same", EffectParameterType::F32),
            EffectParameter::new("same", EffectParameterType::F32),
        ];
        const EMPTY_PARAMETER: &[EffectParameter] =
            &[EffectParameter::new("", EffectParameterType::F32)];
        const DUPLICATE_PASSES: &[EffectPassDefinition] = &[
            EffectPassDefinition::fragment("same", WGSL),
            EffectPassDefinition::fragment("same", WGSL),
        ];
        const EMPTY_PASS: &[EffectPassDefinition] = &[EffectPassDefinition::fragment("", WGSL)];
        const ZERO_SCALE: &[EffectPassDefinition] = &[EffectPassDefinition {
            name: "main",
            wgsl: WGSL,
            inputs: &[],
            scale_divisor: 0,
        }];
        const BAD_WGSL: &[EffectPassDefinition] =
            &[EffectPassDefinition::fragment("main", "not wgsl")];

        let configured = EffectPassDefinition::fragment("configured", WGSL)
            .inputs(&[])
            .downsampled(4);
        assert!(configured.inputs.is_empty());
        assert_eq!(configured.scale_divisor, 4);
        assert_eq!(configured.downsampled(0).scale_divisor, 1);

        for invalid in [
            EffectDefinition::new(EffectId::new("test.empty"), PARAMETERS, &[]),
            EffectDefinition::new(
                EffectId::new("test.empty-parameter"),
                EMPTY_PARAMETER,
                PASSES,
            ),
            EffectDefinition::new(
                EffectId::new("test.duplicate-parameters"),
                DUPLICATE_PARAMETERS,
                PASSES,
            ),
            EffectDefinition::new(EffectId::new("test.empty-pass"), PARAMETERS, EMPTY_PASS),
            EffectDefinition::new(
                EffectId::new("test.duplicate-passes"),
                PARAMETERS,
                DUPLICATE_PASSES,
            ),
            EffectDefinition::new(EffectId::new("test.zero-scale"), PARAMETERS, ZERO_SCALE),
            EffectDefinition::new(EffectId::new("test.bad-wgsl"), PARAMETERS, BAD_WGSL),
        ] {
            assert!(invalid.validate().is_err());
        }

        let registry = EffectRegistry::new([definition(EffectId::new("test.effect"))]).unwrap();
        assert!(registry.get(EffectId::new("test.effect")).is_some());
        assert!(registry.get(EffectId::new("test.missing")).is_none());
        assert!(!registry.is_empty());
        assert!(EffectRegistry::default().is_empty());
    }
}

#[derive(Clone, Debug, Default)]
pub struct EffectRegistry(Arc<EffectRegistryInner>);

#[derive(Debug, Default)]
struct EffectRegistryInner {
    definitions: Vec<EffectDefinition>,
    indices: HashMap<EffectId, usize>,
}

impl EffectRegistry {
    pub fn new(
        definitions: impl IntoIterator<Item = EffectDefinition>,
    ) -> Result<Self, RendererError> {
        let mut ordered = Vec::new();
        let mut indices = HashMap::new();
        for definition in definitions {
            definition.validate()?;
            let id = definition.id;
            if indices.insert(id, ordered.len()).is_some() {
                return Err(RendererError::DuplicateEffect(id.0));
            }
            ordered.push(definition);
        }
        Ok(Self(Arc::new(EffectRegistryInner {
            definitions: ordered,
            indices,
        })))
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
