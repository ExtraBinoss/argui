//! Prepared live effect definitions and monotonic renderer revisions.

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use argui_dsl_ir::{EffectId as DslEffectId, IrType};
use argui_render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition};

use crate::{LivePackage, LiveRuntime};

/// Computes stable fingerprints and monotonic revisions for one package.
///
/// `package` contains validated shaders; `previous` carries revisions from the
/// last visible generation. Returns one `(fingerprint, revision)` per effect.
pub(super) fn revisions(
    package: &LivePackage,
    previous: Option<&HashMap<DslEffectId, (u64, u64)>>,
) -> HashMap<DslEffectId, (u64, u64)> {
    package
        .ir
        .effects
        .iter()
        .map(|effect| {
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            package.assets[&effect.shader].bytes.hash(&mut hash);
            effect.bounded_damage.hash(&mut hash);
            for parameter in &effect.parameters {
                parameter.name.hash(&mut hash);
                format!("{:?}", parameter.value_type).hash(&mut hash);
            }
            let fingerprint = hash.finish();
            let revision =
                previous
                    .and_then(|old| old.get(&effect.id))
                    .map_or(1, |(old_hash, revision)| {
                        if *old_hash == fingerprint {
                            *revision
                        } else {
                            revision.saturating_add(1)
                        }
                    });
            (effect.id, (fingerprint, revision))
        })
        .collect()
}

/// Converts a checked effect parameter type into renderer metadata.
///
/// `value` is the normalized DSL parameter type. Returns the renderer type;
/// preparation rejects unsupported types before this function is called.
fn parameter_type(value: &IrType) -> EffectParameterType {
    match value {
        IrType::Float => EffectParameterType::F32,
        IrType::Int => EffectParameterType::I32,
        IrType::Bool => EffectParameterType::Bool,
        IrType::Color => EffectParameterType::Color,
        IrType::Length => EffectParameterType::LogicalPixels,
        IrType::Transform => EffectParameterType::Mat3,
        _ => unreachable!("prepared effect parameter type was validated"),
    }
}

impl LiveRuntime {
    /// Returns current shader definitions for transactional renderer registration.
    ///
    /// Each revision increases only when its WGSL or parameter schema changes.
    /// Identical reloads preserve the revision. Package preparation guarantees
    /// valid UTF-8 and supported parameter types.
    #[must_use]
    pub fn effect_definitions(&self) -> Vec<EffectDefinition> {
        self.package
            .ir
            .effects
            .iter()
            .map(|effect| {
                let source = std::str::from_utf8(&self.package.assets[&effect.shader].bytes)
                    .expect("prepared effect shader is UTF-8");
                let parameters = effect
                    .parameters
                    .iter()
                    .map(|parameter| {
                        EffectParameter::new(
                            parameter.name.clone(),
                            parameter_type(&parameter.value_type),
                        )
                    })
                    .collect::<Vec<_>>();
                EffectDefinition::new(
                    argui_paint::EffectId::from_owned(format!("dsl.{}", effect.id.raw())),
                    parameters,
                    [EffectPassDefinition::fragment("main", source.to_owned())],
                )
                .damage(if effect.bounded_damage {
                    argui_render::EffectDamage::Bounded
                } else {
                    argui_render::EffectDamage::Unbounded
                })
                .with_revision(self.effect_revisions[&effect.id].1)
            })
            .collect()
    }
}
