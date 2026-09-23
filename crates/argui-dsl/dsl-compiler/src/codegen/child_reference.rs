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
        IrExpressionKind::Struct { fields, .. } => {
            fields.iter().any(|(_, value)| contextual_default(value))
        }
        IrExpressionKind::Index { base, index } => {
            contextual_default(base) || contextual_default(index)
        }
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
    /// receives `(site, property)` handle names. `repeater_key` scopes retained
    /// identities to a row when present. Returns an error if a checked child
    /// property cannot be emitted.
    pub(super) fn emit_child_references(
        &self,
        output: &mut String,
        nodes: &[IrNode],
        referenced: &HashSet<SiteId>,
        scope: &mut Scope,
        repeater_key: Option<&argui_dsl_ir::IrExpression>,
    ) -> Result<(), CompilerError> {
        for node in nodes {
            match node {
                IrNode::Element {
                    site,
                    target,
                    source_id,
                    properties,
                    children,
                    ..
                } => {
                    if source_id.is_some()
                        && referenced.contains(site)
                        && let IrElementTarget::Component(component) = target
                    {
                        self.emit_child_reference(
                            output,
                            *site,
                            *component,
                            properties,
                            scope,
                            repeater_key,
                        )?;
                    }
                    if matches!(target, IrElementTarget::Native(_)) {
                        self.emit_child_references(
                            output,
                            children,
                            referenced,
                            scope,
                            repeater_key,
                        )?;
                    }
                }
                IrNode::Conditional {
                    site,
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    let mut targets = Vec::new();
                    collect_conditional_children(then_body, referenced, &mut targets);
                    collect_conditional_children(else_body, referenced, &mut targets);
                    targets.sort_by_key(|(site, _)| site.raw());
                    targets.dedup_by_key(|(site, _)| *site);
                    if targets.is_empty() {
                        continue;
                    }
                    let mut outputs = Vec::new();
                    for (child_site, component) in targets {
                        for property in &self.components[&component].properties {
                            let variable = format!(
                                "optional_child_p_{}_{}_{}",
                                site.raw(),
                                child_site.raw(),
                                property.id.raw()
                            );
                            let value_type = self.ir_rust_type(&property.value_type)?;
                            writeln!(output, "let mut {variable}: Option<{value_type}> = None;")
                                .unwrap();
                            scope
                                .optional_child_properties
                                .insert((child_site, property.id), variable.clone());
                            outputs.push((child_site, property.id, variable));
                        }
                    }
                    let condition = self.expression(condition, scope)?;
                    for (body, predicate) in [(then_body, true), (else_body, false)] {
                        let mut present = Vec::new();
                        collect_conditional_children(body, referenced, &mut present);
                        if present.is_empty() {
                            continue;
                        }
                        writeln!(
                            output,
                            "if {}({condition}) {{",
                            if predicate { "" } else { "!" }
                        )
                        .unwrap();
                        let mut inner = scope.clone();
                        self.emit_child_references(
                            output,
                            body,
                            referenced,
                            &mut inner,
                            repeater_key,
                        )?;
                        for (child_site, property, variable) in &outputs {
                            if !present.iter().any(|(site, _)| site == child_site) {
                                continue;
                            }
                            let key = &(*child_site, *property);
                            if let Some(handle) = inner.child_properties.get(key) {
                                writeln!(output, "{variable} = Some({handle}.get());").unwrap();
                            } else if let Some(value) = inner.optional_child_properties.get(key) {
                                writeln!(output, "{variable} = {value}.clone();").unwrap();
                            }
                        }
                        writeln!(output, "}}").unwrap();
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Emits the canonical handles for one unconditional identified child call.
    ///
    /// `site` identifies the child, `component` resolves its typed properties,
    /// `bindings` supply parent inputs, `scope` receives readable handles, and
    /// `repeater_key` scopes retained identities to a row. Returns a
    /// code-generation error for unresolved two-way sources.
    fn emit_child_reference(
        &self,
        output: &mut String,
        site: SiteId,
        component: argui_dsl_ir::ComponentId,
        bindings: &[argui_dsl_ir::IrPropertyBinding],
        scope: &mut Scope,
        repeater_key: Option<&argui_dsl_ir::IrExpression>,
    ) -> Result<(), CompilerError> {
        let source = self.component_definition(component)?;
        let ir = self.components[&component];
        let identity = format!("child_ref_identity_{}", site.raw());
        let identity_value = self.identity(site.raw(), scope, repeater_key)?;
        writeln!(output, "let {identity} = {identity_value};").unwrap();
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
                    let value =
                        self.expression_as(&binding.value, &lowered.value_type, &canonical)?;
                    format!(
                        "child_properties.controlled({identity}.clone(), {}_u64, {value})",
                        lowered.id.raw()
                    )
                }
            } else {
                let value = if let Some(default) = &lowered.default {
                    let expression = self.expression_as(default, &lowered.value_type, &inner)?;
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

/// Collects identified component calls in one conditional branch without entering
/// nested repeaters. `nodes` is the branch body, `referenced` filters sites read
/// by expressions, and `output` receives source site/target pairs.
fn collect_conditional_children(
    nodes: &[IrNode],
    referenced: &HashSet<SiteId>,
    output: &mut Vec<(SiteId, argui_dsl_ir::ComponentId)>,
) {
    for node in nodes {
        match node {
            IrNode::Element {
                site,
                target,
                source_id,
                children,
                ..
            } => {
                if source_id.is_some()
                    && referenced.contains(site)
                    && let IrElementTarget::Component(component) = target
                {
                    output.push((*site, *component));
                }
                if matches!(target, IrElementTarget::Native(_)) {
                    collect_conditional_children(children, referenced, output);
                }
            }
            IrNode::Conditional {
                then_body,
                else_body,
                ..
            } => {
                collect_conditional_children(then_body, referenced, output);
                collect_conditional_children(else_body, referenced, output);
            }
            _ => {}
        }
    }
}
