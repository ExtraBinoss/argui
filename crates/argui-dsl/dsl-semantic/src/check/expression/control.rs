//! Event propagation and default-action builtin validation.

use argui_dsl_syntax::{Span, SyntaxNode};

use super::{Context, infer};
use crate::{Diagnostic, DiagnosticCode, Type};

/// Checks a zero-argument event control call within a handler.
///
/// `name` identifies the builtin; `node` locates the call; `arguments` are
/// parsed operands; and `context` receives source diagnostics. Returns void.
pub(super) fn check(
    name: &str,
    node: &SyntaxNode,
    arguments: &[SyntaxNode],
    context: &mut Context<'_, '_>,
) -> Type {
    if !context.event_handler {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("{name}() is only valid inside an event handler"),
            Span::new(context.file, node.text_range()),
        ));
    }
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
