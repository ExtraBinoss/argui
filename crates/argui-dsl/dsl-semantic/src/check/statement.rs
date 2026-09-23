//! Handler assignments and return contracts, checked before IR lowering.

use argui_dsl_syntax::{Span, SyntaxKind, SyntaxNode};

use super::expression::{self, Context};
use crate::{Diagnostic, DiagnosticCode, PropertyDirection, Type, lower::direct_tokens};

/// Validates a handler `node` against its callback `result` and lexical `context`.
/// Diagnostics are appended to the context; no runtime state is changed.
pub(super) fn check(node: &SyntaxNode, result: &Type, context: &mut Context<'_, '_>) {
    let values = node
        .children()
        .filter(|child| child.kind() == SyntaxKind::Expr)
        .collect::<Vec<_>>();
    if direct_tokens(node).any(|token| token.kind() == SyntaxKind::LetKw) {
        let [destination, value] = values.as_slice() else {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                "let requires a name and initializer",
                Span::new(context.file, node.text_range()),
            ));
            return;
        };
        let Some(name) = property_name(destination) else {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::ReadOnlyProperty,
                "let requires a bare local name",
                Span::new(context.file, destination.text_range()),
            ));
            return;
        };
        if context.locals.contains_key(&name) || context.properties.contains_key(&name) {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateMember,
                format!("local `{name}` already exists in this scope"),
                Span::new(context.file, destination.text_range()),
            ));
        }
        if !direct_tokens(node).any(|token| token.kind() == SyntaxKind::Eq) {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                "let requires `=` before its initializer",
                Span::new(context.file, node.text_range()),
            ));
        }
        let actual = expression::infer(value, context);
        context.locals.insert(name, actual);
        return;
    }
    if direct_tokens(node).any(|token| token.kind() == SyntaxKind::ReturnKw) {
        let actual = values
            .first()
            .map_or(Type::Void, |value| expression::infer(value, context));
        require(node, result, &actual, context);
        return;
    }
    let operator = direct_tokens(node).map(|token| token.kind()).find(|kind| {
        matches!(
            kind,
            SyntaxKind::Eq
                | SyntaxKind::PlusEq
                | SyntaxKind::MinusEq
                | SyntaxKind::StarEq
                | SyntaxKind::SlashEq
        )
    });
    let Some(operator) = operator else {
        for value in values {
            expression::infer(&value, context);
        }
        return;
    };
    let [destination, value] = values.as_slice() else {
        return;
    };
    let target = property_name(destination);
    let expected = expression::infer(destination, context);
    let actual = expression::infer(value, context);
    let writable = target.as_ref().is_some_and(|name| {
        context.locals.contains_key(name)
            || context
                .properties
                .get(name)
                .is_some_and(|property| property.direction != PropertyDirection::Input)
    });
    if !writable {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::ReadOnlyProperty,
            "assignment target must be a writable local or component property",
            Span::new(context.file, destination.text_range()),
        ));
    }
    let actual = match operator {
        SyntaxKind::Eq => actual,
        SyntaxKind::PlusEq if expected == Type::String && actual == Type::String => Type::String,
        _ => {
            let operation = match operator {
                SyntaxKind::PlusEq => SyntaxKind::Plus,
                SyntaxKind::MinusEq => SyntaxKind::Minus,
                SyntaxKind::StarEq => SyntaxKind::Star,
                _ => SyntaxKind::Slash,
            };
            expression::arithmetic(context, node, operation, expected.clone(), actual)
        }
    };
    require(value, &expected, &actual, context);
}

/// Returns a bare property/local name from `node`, or `None` for computed targets.
pub(super) fn property_name(node: &SyntaxNode) -> Option<String> {
    let child = if node.kind() == SyntaxKind::Expr {
        node.children().next()?
    } else {
        node.clone()
    };
    if child.kind() != SyntaxKind::PathExpr {
        return None;
    }
    direct_tokens(&child)
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_owned())
}

/// Appends a source error at `node` if `actual` cannot be assigned to `expected`.
fn require(node: &SyntaxNode, expected: &Type, actual: &Type, context: &mut Context<'_, '_>) {
    if !expected.accepts(actual) {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("expected `{expected}`, found `{actual}`"),
            Span::new(context.file, node.text_range()),
        ));
    }
}
