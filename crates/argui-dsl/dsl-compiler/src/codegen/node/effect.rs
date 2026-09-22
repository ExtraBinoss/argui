//! Generic AOT effect instance emission for visual elements.

use std::fmt::Write;

use argui_dsl_ir::{IrEffectBinding, IrEffectScope, IrType};

use crate::{
    CompilerError,
    codegen::{Context, expression::Scope},
};

impl Context<'_> {
    /// Wraps the most recently emitted child with a typed effect instance.
    ///
    /// `output` receives Rust source, `binding` selects the effect and ordered
    /// arguments, `destination` is the element vector, `depth` controls
    /// indentation, and `scope` resolves live property expressions.
    ///
    /// # Errors
    ///
    /// Returns a code-generation error for a missing effect definition or
    /// unsupported parameter type.
    pub(super) fn emit_applied_effect(
        &self,
        output: &mut String,
        binding: &IrEffectBinding,
        destination: &str,
        depth: usize,
        scope: &Scope,
    ) -> Result<(), CompilerError> {
        let definition = self
            .ir
            .effects
            .iter()
            .find(|effect| effect.id == binding.effect)
            .ok_or(CompilerError::Codegen(format!(
                "effect {} is not defined",
                binding.effect.raw()
            )))?;
        let pad = "    ".repeat(depth);
        let target = match binding.scope {
            IrEffectScope::Whole => "WholeElement",
            IrEffectScope::Background => "Background",
            IrEffectScope::Border => "Border",
            IrEffectScope::Content => "Content",
            IrEffectScope::Text => "Text",
            IrEffectScope::Backdrop => "Backdrop",
        };
        writeln!(
            output,
            "{pad}{{ let element = {destination}.pop().expect(\"effect target was just emitted\");"
        )
        .unwrap();
        writeln!(output, "{pad}{destination}.push(::argui::runtime::apply_visual_effect_scoped(element, ::argui::paint::EffectInstance::new(::argui::paint::EffectId::new(\"dsl.{}\"), [", binding.effect.raw()).unwrap();
        for argument in &binding.parameters {
            let parameter = definition
                .parameters
                .iter()
                .find(|parameter| parameter.id == argument.parameter)
                .ok_or(CompilerError::Codegen(format!(
                    "effect parameter {} is not defined",
                    argument.parameter.raw()
                )))?;
            let value = self.expression(&argument.value, scope)?;
            let converted = match &parameter.value_type {
                IrType::Float => format!("::argui::paint::EffectValue::F32(({value}) as f32)"),
                IrType::Int => format!(
                    "::argui::paint::EffectValue::I32((({value}) as i64).clamp(i32::MIN as i64, i32::MAX as i64) as i32)"
                ),
                IrType::Bool => format!("::argui::paint::EffectValue::Bool({value})"),
                IrType::Color => format!("::argui::paint::EffectValue::Color({value})"),
                IrType::Length => {
                    format!("::argui::paint::EffectValue::LogicalPixels(({value}) as f32)")
                }
                other => {
                    return Err(CompilerError::Codegen(format!(
                        "unsupported effect parameter type {other:?}"
                    )));
                }
            };
            writeln!(
                output,
                "{pad}::argui::paint::EffectArgument::new(\"{}\", {converted}),",
                parameter.name.escape_default()
            )
            .unwrap();
        }
        writeln!(
            output,
            "{pad}]), ::argui::runtime::VisualEffectTarget::{target})); }}"
        )
        .unwrap();
        Ok(())
    }
}
