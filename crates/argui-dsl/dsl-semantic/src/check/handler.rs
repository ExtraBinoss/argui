//! Whole-handler return contracts and lexical branch validation.

use super::{
    expression::{self, Context},
    statement,
};
use crate::{Diagnostic, DiagnosticCode, Type, lower::direct_tokens};
use argui_dsl_syntax::{Span, SyntaxKind, SyntaxNode};

/// Checks all handler statements and reports missing non-void returns.
/// `node` is the event body; `result` is its callback return contract; `context`
/// contains the surrounding properties and mutable lexical locals.
pub(super) fn check(node: &SyntaxNode, result: &Type, context: &mut Context<'_, '_>) {
    let returns = sequence(node, result, context);
    if *result != Type::Void && !returns {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("handler must return a value of type `{result}`"),
            Span::new(context.file, node.text_range()),
        ));
    }
}

/// Checks one lexical statement sequence and returns whether every path returns.
fn sequence(node: &SyntaxNode, result: &Type, context: &mut Context<'_, '_>) -> bool {
    let mut returns = false;
    for item in node.children().filter(|child| {
        matches!(
            child.kind(),
            SyntaxKind::Statement | SyntaxKind::IfStatement
        )
    }) {
        if returns {
            context.diagnostics.push(Diagnostic::warning(
                DiagnosticCode::UnreachableStatement,
                "statement is unreachable after return",
                Span::new(context.file, item.text_range()),
            ));
        }
        let item_returns = if item.kind() == SyntaxKind::IfStatement {
            branch(&item, result, context)
        } else {
            statement::check(&item, result, context);
            direct_tokens(&item).any(|token| token.kind() == SyntaxKind::ReturnKw)
        };
        returns |= item_returns;
    }
    returns
}

/// Checks a conditional handler branch with separate lexical local scopes.
/// Returns true only when both branches exist and both guarantee return.
fn branch(node: &SyntaxNode, result: &Type, context: &mut Context<'_, '_>) -> bool {
    if let Some(condition) = node
        .children()
        .find(|child| child.kind() == SyntaxKind::Expr)
    {
        let actual = expression::infer(&condition, context);
        if !Type::Bool.accepts(&actual) {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("handler condition expects `bool`, found `{actual}`"),
                Span::new(context.file, condition.text_range()),
            ));
        }
    }
    let outer = context.locals.clone();
    let mut blocks = node
        .children()
        .filter(|child| child.kind() == SyntaxKind::Block);
    let then_returns = blocks
        .next()
        .is_some_and(|block| sequence(&block, result, context));
    context.locals = outer.clone();
    let else_returns = blocks
        .next()
        .is_some_and(|block| sequence(&block, result, context));
    context.locals = outer;
    then_returns && else_returns
}
