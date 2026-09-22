//! Renderer effect definition code generation.

use std::fmt::Write;

use argui_dsl_semantic::{Definition, DefinitionKind};

use super::{Context, escape, types};
use crate::CompilerError;

impl Context<'_> {
    /// Emits validated custom effect definitions for renderer registration.
    pub(super) fn emit_effects(&self, output: &mut String) -> Result<(), CompilerError> {
        writeln!(
            output,
            "pub fn effect_definitions() -> Vec<::argui::render::EffectDefinition> {{ vec!["
        )
        .unwrap();
        for effect in &self.ir.effects {
            if !self.reachable.effects.contains(&effect.id) {
                continue;
            }
            let asset = self
                .ir
                .assets
                .iter()
                .find(|asset| asset.id == effect.shader)
                .ok_or(CompilerError::Codegen(
                    "effect shader asset is missing".into(),
                ))?;
            writeln!(output, "::argui::render::EffectDefinition::new(::argui::paint::EffectId::new(\"dsl.{}\"), [", effect.id.raw()).unwrap();
            let definition = self
                .definitions
                .values()
                .find(|definition| definition.id.raw() == effect.id.raw());
            if let Some(Definition {
                kind: DefinitionKind::Effect(source),
                ..
            }) = definition
            {
                for parameter in &source.parameters {
                    writeln!(
                        output,
                        "::argui::render::EffectParameter::new(\"{}\", {}),",
                        escape(&parameter.name),
                        types::effect_parameter_type(&parameter.value_type)?
                    )
                    .unwrap();
                }
            }
            let damage = if effect.bounded_damage {
                "Bounded"
            } else {
                "Unbounded"
            };
            writeln!(output, "], [::argui::render::EffectPassDefinition::fragment(\"main\", include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\"))) ]).damage(::argui::render::EffectDamage::{damage}),", escape(&asset.path)).unwrap();
        }
        writeln!(output, "] }}").unwrap();
        Ok(())
    }
}
