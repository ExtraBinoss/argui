//! Generic live effect instance construction for visual elements.

use std::collections::HashMap;

use argui_dsl_ir::{IrEffectBinding, IrEffectScope, IrType, LocalId};
use argui_paint::{EffectArgument, EffectId, EffectInstance, EffectValue};
use argui_runtime::VisualEffectTarget;

use crate::{ComponentInstance, DslValue, LiveRuntime, RuntimeError};

impl LiveRuntime {
    /// Applies a typed effect instance to one rendered element when configured.
    ///
    /// `instance` provides reactive values, `binding` selects an optional
    /// effect, `locals` contains repeater values, and `element` is the rendered
    /// native or component visual. Returns the decorated element.
    ///
    /// # Errors
    ///
    /// Returns an evaluation or type error for an invalid package expression.
    pub(super) fn apply_effect(
        &self,
        instance: &mut ComponentInstance,
        binding: Option<&IrEffectBinding>,
        locals: &HashMap<LocalId, DslValue>,
        element: argui_ui::Element,
    ) -> Result<argui_ui::Element, RuntimeError> {
        let Some(binding) = binding else {
            return Ok(element);
        };
        let definition = self
            .package
            .ir
            .effects
            .iter()
            .find(|effect| effect.id == binding.effect)
            .ok_or_else(|| {
                RuntimeError::InvalidBytecode(format!(
                    "effect {} is not defined",
                    binding.effect.raw()
                ))
            })?;
        let mut arguments = Vec::with_capacity(binding.parameters.len());
        for argument in &binding.parameters {
            let parameter = definition
                .parameters
                .iter()
                .find(|parameter| parameter.id == argument.parameter)
                .ok_or_else(|| {
                    RuntimeError::InvalidBytecode(format!(
                        "effect parameter {} is not defined",
                        argument.parameter.raw()
                    ))
                })?;
            let value = self.evaluate(instance, &argument.value, locals)?;
            let value = match (value, &parameter.value_type) {
                (DslValue::Float(value), IrType::Float) => EffectValue::F32(value as f32),
                (DslValue::Int(value), IrType::Int) => {
                    EffectValue::I32(value.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
                }
                (DslValue::Bool(value), IrType::Bool) => EffectValue::Bool(value),
                (DslValue::Color(value), IrType::Color) => EffectValue::Color(value),
                (DslValue::Float(value), IrType::Length) => {
                    EffectValue::LogicalPixels(value as f32)
                }
                (actual, expected) => {
                    return Err(RuntimeError::TypeMismatch {
                        expected: format!("effect parameter {expected:?}"),
                        actual: actual.type_name().into(),
                    });
                }
            };
            arguments.push(EffectArgument::from_owned(parameter.name.clone(), value));
        }
        let id = EffectId::from_owned(format!("dsl.{}", binding.effect.raw()));
        let target = match binding.scope {
            IrEffectScope::Whole => VisualEffectTarget::WholeElement,
            IrEffectScope::Background => VisualEffectTarget::Background,
            IrEffectScope::Border => VisualEffectTarget::Border,
            IrEffectScope::Content => VisualEffectTarget::Content,
            IrEffectScope::Text => VisualEffectTarget::Text,
            IrEffectScope::Backdrop => VisualEffectTarget::Backdrop,
        };
        Ok(argui_runtime::apply_visual_effect_scoped(
            element,
            EffectInstance::new(id, arguments),
            target,
        ))
    }
}
