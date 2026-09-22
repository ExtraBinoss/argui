//! Type checking for an identity-targeted scroll request.

use argui_dsl_syntax::{Span, SyntaxKind, SyntaxNode};

use super::{Context, infer, type_mismatch};
use crate::{Diagnostic, DiagnosticCode, Type};

/// Checks `scroll_to(#site, x, y)` in a handler expression.
///
/// `node` locates call diagnostics, `arguments` are its parsed operands, and
/// `context` supplies visible element identities and receives type errors.
/// Returns the builtin's void type.
pub(super) fn check(
    node: &SyntaxNode,
    arguments: &[SyntaxNode],
    context: &mut Context<'_, '_>,
) -> Type {
    if arguments.len() != 3 {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            "scroll_to() expects an element reference and two lengths",
            Span::new(context.file, node.text_range()),
        ));
    }
    if let Some(target) = arguments.first() {
        let reference = target
            .children()
            .find(|child| child.kind() == SyntaxKind::LiteralExpr)
            .and_then(|literal| {
                let mut tokens = literal
                    .children_with_tokens()
                    .filter_map(|child| child.into_token())
                    .filter(|token| !token.kind().is_trivia());
                (tokens.next()?.kind() == SyntaxKind::Hash)
                    .then(|| tokens.next())
                    .flatten()
                    .filter(|token| token.kind() == SyntaxKind::Ident)
                    .map(|token| token.text().to_string())
            });
        if !reference.as_ref().is_some_and(|name| {
            context
                .references
                .is_some_and(|sites| sites.contains_key(name))
        }) {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                "scroll_to() target must be a unique native #element identity",
                Span::new(context.file, target.text_range()),
            ));
        }
    }
    for coordinate in arguments.iter().skip(1) {
        let actual = infer(coordinate, context);
        if !Type::Length.accepts(&actual) {
            type_mismatch(context, coordinate, &Type::Length, &actual);
        }
    }
    Type::Void
}
