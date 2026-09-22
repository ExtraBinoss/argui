//! Retained child output handles prepared before sibling visual evaluation.

use std::collections::HashSet;
use std::fmt::Write;

use argui_dsl_ir::{
    IrElementTarget, IrExpression, IrExpressionKind, IrNode, PropertyTargetId, SiteId,
};

use super::{Context, expression::Scope};
use crate::CompilerError;

/// Returns whether a default reads retained visual state and needs render-time evaluation.
///
/// `expression` is a typed property default. The result is true for a native
/// observation or child output, including reads nested in arithmetic and calls.
pub(super) fn contextual_default(expression: &IrExpression) -> bool {
    match &expression.kind {
        IrExpressionKind::ObservedRead { .. } | IrExpressionKind::ChildPropertyRead { .. } => true,
        IrExpressionKind::FieldRead { base, .. }
        | IrExpressionKind::Unary { operand: base, .. } => contextual_default(base),
        IrExpressionKind::BuiltinCall { arguments, .. }
        | IrExpressionKind::CallbackCall { arguments, .. }
        | IrExpressionKind::Array(arguments) => arguments.iter().any(contextual_default),
        IrExpressionKind::Binary { left, right, .. } => {
            contextual_default(left) || contextual_default(right)
        }
        IrExpressionKind::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            contextual_default(condition)
                || contextual_default(then_value)
                || contextual_default(else_value)
        }
        _ => false,
    }
}

impl Context<'_> {
    /// Prepares stable child property handles before any sibling visual reads.
    ///
    /// `output` receives generated Rust; `nodes` are the component's visual
    /// tree; `referenced` contains child sites actually read; and `scope`
    /// receives `(site, property)` handle names. Returns an error if a checked
    /// child property cannot be emitted.
    pub(super) fn emit_child_references(
        &self,
        output: &mut String,
        nodes: &[IrNode],
        referenced: &HashSet<SiteId>,
        scope: &mut Scope,
    ) -> Result<(), CompilerError> {
        for node in nodes {
            if let IrNode::Element {
                site,
                target,
                source_id,
                properties,
                children,
                ..
            } = node
            {
                if source_id.is_some()
                    && referenced.contains(site)
                    && let IrElementTarget::Component(component) = target
                {
                    self.emit_child_reference(output, *site, *component, properties, scope)?;
                }
                if matches!(target, IrElementTarget::Native(_)) {
                    self.emit_child_references(output, children, referenced, scope)?;
                }
            }
        }
        Ok(())
    }

    /// Emits the canonical handles for one unconditional identified child call.
    ///
    /// `site` identifies the child, `component` resolves its typed properties,
    /// `bindings` supply parent inputs, and `scope` receives readable handles.
    /// Returns a code-generation error for unresolved two-way sources.
    fn emit_child_reference(
        &self,
        output: &mut String,
        site: SiteId,
        component: argui_dsl_ir::ComponentId,
        bindings: &[argui_dsl_ir::IrPropertyBinding],
        scope: &mut Scope,
    ) -> Result<(), CompilerError> {
        let source = self.component_definition(component)?;
        let ir = self.components[&component];
        let identity = format!("child_ref_identity_{}", site.raw());
        writeln!(
            output,
            "let {identity} = ::argui::ui::RetainedIdentity::new(owner, {});",
            site.raw()
        )
        .unwrap();
        let mut inner = Scope {
            translator: scope.translator.clone(),
            ..Scope::default()
        };
        let mut canonical = scope.clone();
        canonical.rendered_properties.clear();
        for (property, lowered) in source.properties.iter().zip(&ir.properties) {
            let variable = format!("_child_ref_p_{}_{}", site.raw(), lowered.id.raw());
            let binding = bindings
                .iter()
                .find(|binding| binding.target == PropertyTargetId::Component(lowered.id));
            let initial = if let Some(binding) = binding {
                if binding.two_way {
                    let IrExpressionKind::PropertyRead(parent) = binding.value.kind else {
                        return Err(CompilerError::Codegen(
                            "two-way child reference source is not a property".into(),
                        ));
                    };
                    format!(
                        "{}.clone()",
                        scope.properties.get(&parent).ok_or(CompilerError::Codegen(
                            "two-way child reference source is outside scope".into()
                        ))?
                    )
                } else {
                    let value = self.expression(&binding.value, &canonical)?;
                    format!(
                        "child_properties.controlled({identity}.clone(), {}_u64, {value})",
                        lowered.id.raw()
                    )
                }
            } else {
                let value = if let Some(default) = &lowered.default {
                    let expression = self.expression(default, &inner)?;
                    format!(
                        "{{ let owner = child_owner(&{identity}); let _ = owner; {expression} }}"
                    )
                } else {
                    self.default_value(&property.value_type)?
                };
                format!(
                    "child_properties.defaulted({identity}.clone(), {}_u64, {value})",
                    lowered.id.raw()
                )
            };
            writeln!(output, "let {variable} = {initial};").unwrap();
            inner.properties.insert(lowered.id, variable.clone());
            scope.child_properties.insert((site, lowered.id), variable);
        }
        Ok(())
    }
}
