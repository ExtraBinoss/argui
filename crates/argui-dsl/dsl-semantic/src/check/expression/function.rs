//! Signature checking for expression-bodied pure functions.

use argui_dsl_syntax::{Span, SyntaxNode};

use crate::{DefinitionKind, Diagnostic, DiagnosticCode, Type};

use super::{Context, infer, type_mismatch};

/// Resolves and checks a pure function call visible in this lexical module.
///
/// `name` is the callee, `node` its source, `arguments` its parsed inputs,
/// and `context` supplies imports and diagnostics. Returns `None` when this
/// name is not a pure function.
pub(super) fn check(
    name: &str,
    node: &SyntaxNode,
    arguments: &[SyntaxNode],
    context: &mut Context<'_, '_>,
) -> Option<Type> {
    let definition = context
        .symbols
        .get(name)
        .and_then(|id| context.definitions.get(id))?;
    let DefinitionKind::Function(function) = &definition.kind else {
        return None;
    };
    let function = function.clone();
    if function.parameters.len() != arguments.len() {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!(
                "function `{name}` expects {} arguments, received {}",
                function.parameters.len(),
                arguments.len()
            ),
            Span::new(context.file, node.text_range()),
        ));
    }
    for (argument, parameter) in arguments.iter().zip(&function.parameters) {
        let actual = infer(argument, context);
        if !parameter.value_type.accepts(&actual) {
            type_mismatch(context, argument, &parameter.value_type, &actual);
        }
    }
    Some(function.result)
}
