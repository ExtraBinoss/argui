//! Type checking for focus traversal expressions.

use argui_dsl_syntax::{Span, SyntaxNode};

use super::{Context, infer};
use crate::{Diagnostic, DiagnosticCode, Type};

/// Checks one zero-argument focus traversal call.
///
/// `name` identifies the traversal builtin, `node` locates its diagnostic,
/// `arguments` are parsed call operands, and `context` receives diagnostics.
/// Returns the builtin's void type.
pub(super) fn check(
    name: &str,
    node: &SyntaxNode,
    arguments: &[SyntaxNode],
    context: &mut Context<'_, '_>,
) -> Type {
    if !arguments.is_empty() {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("{name}() expects no arguments"),
            Span::new(context.file, node.text_range()),
        ));
        for argument in arguments {
            let _ = infer(argument, context);
        }
    }
    Type::Void
}
