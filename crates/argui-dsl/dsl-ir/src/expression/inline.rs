//! Stable identities for expressions expanded from typed pure functions.

use std::collections::HashMap;

use argui_dsl_syntax::SyntaxNode;

use crate::{ExpressionId, IrExpression, IrExpressionKind, SourceInfo};

use super::{Context, expression_id, lower};

/// Expands a resolved typed pure call to ordinary expression IR.
///
/// `callee` is the visible name, `arguments` are source expressions, `source`
/// identifies the call site, and `context` owns the project indices. Returns
/// `None` when the name is not a pure function.
pub(super) fn lower_call(
    callee: &str,
    arguments: &[SyntaxNode],
    source: &SourceInfo,
    context: &mut Context<'_>,
) -> Option<IrExpression> {
    let function = context
        .symbols
        .get(callee)
        .and_then(|id| context.functions.get(id))
        .cloned()?;
    let values = arguments
        .iter()
        .map(|node| lower(node, context))
        .collect::<Vec<_>>();
    if values.len() != function.parameters.len() {
        return None;
    }
    let values = function
        .parameters
        .iter()
        .cloned()
        .zip(values)
        .collect::<HashMap<_, _>>();
    let empty_properties = HashMap::new();
    let empty_callbacks = HashMap::new();
    let empty_locals = HashMap::new();
    let mut nested = Context {
        file: function.file,
        module_path: &function.module_path,
        component: context.component,
        site: context.site,
        properties: &empty_properties,
        callbacks: &empty_callbacks,
        locals: &empty_locals,
        tokens: context.tokens,
        fields: context.fields,
        symbols: &function.scope,
        functions: context.functions,
        function_arguments: Some(&values),
        references: None,
        assets: context.assets,
        errors: context.errors,
    };
    let mut result = lower(&function.body, &mut nested);
    rebase_expression_ids(&mut result, expression_id(source).raw());
    Some(result)
}

/// Gives every expanded expression a stable identity under its call site.
///
/// `expression` is the expanded tree and `callsite` is the owning call ID.
pub(super) fn rebase_expression_ids(expression: &mut IrExpression, callsite: u64) {
    expression.id =
        ExpressionId::from_raw(crate::id::derive(callsite, "pure", expression.id.raw()));
    match &mut expression.kind {
        IrExpressionKind::FieldRead { base, .. } => rebase_expression_ids(base, callsite),
        IrExpressionKind::BuiltinCall { arguments, .. }
        | IrExpressionKind::CallbackCall { arguments, .. }
        | IrExpressionKind::Array(arguments) => {
            for argument in arguments {
                rebase_expression_ids(argument, callsite);
            }
        }
        IrExpressionKind::Unary { operand, .. } => rebase_expression_ids(operand, callsite),
        IrExpressionKind::Binary { left, right, .. }
        | IrExpressionKind::Index {
            base: left,
            index: right,
        } => {
            rebase_expression_ids(left, callsite);
            rebase_expression_ids(right, callsite);
        }
        IrExpressionKind::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            rebase_expression_ids(condition, callsite);
            rebase_expression_ids(then_value, callsite);
            rebase_expression_ids(else_value, callsite);
        }
        IrExpressionKind::Struct { fields, .. } => {
            for (_, value) in fields {
                rebase_expression_ids(value, callsite);
            }
        }
        _ => {}
    }
}
