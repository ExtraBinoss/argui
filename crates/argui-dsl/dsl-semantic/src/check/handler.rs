//! Whole-handler return contracts and statement validation.

use super::{expression::Context, statement};
use crate::{Diagnostic, DiagnosticCode, Type, lower::direct_tokens};
use argui_dsl_syntax::{Span, SyntaxKind, SyntaxNode};

/// Checks the statements in handler `node` against `result` using `context`.
/// Reports missing non-void returns and validates even unreachable statements.
pub(super) fn check(node: &SyntaxNode, result: &Type, context: &mut Context<'_, '_>) {
    let mut returns = false;
    for item in node
        .children()
        .filter(|child| child.kind() == SyntaxKind::Statement)
    {
        if returns {
            context.diagnostics.push(Diagnostic::warning(
                DiagnosticCode::UnreachableStatement,
                "statement is unreachable after return",
                Span::new(context.file, item.text_range()),
            ));
        }
        statement::check(&item, result, context);
        returns |= direct_tokens(&item).any(|token| token.kind() == SyntaxKind::ReturnKw);
    }
    if *result != Type::Void && !returns {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("handler must return a value of type `{result}`"),
            Span::new(context.file, node.text_range()),
        ));
    }
}
