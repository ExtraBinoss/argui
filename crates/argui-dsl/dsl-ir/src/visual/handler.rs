//! Restricted handler call recognition during IR lowering.

use argui_dsl_syntax::{SyntaxKind, SyntaxNode};

use super::direct_identifier;
use crate::IrStatement;

/// Recognizes a checked focus or event-control call as a handler statement.
///
/// `node` is the event statement expression. Returns the corresponding host
/// action, or `None` when the expression is not a supported action call.
pub(super) fn host_action_call(node: &SyntaxNode) -> Option<IrStatement> {
    let call = node
        .children()
        .find(|child| child.kind() == SyntaxKind::CallExpr)?;
    let callee = call
        .descendants()
        .find(|child| child.kind() == SyntaxKind::PathExpr)
        .and_then(|path| direct_identifier(&path))?;
    match callee.as_str() {
        "focus_next" => Some(IrStatement::FocusNext),
        "focus_previous" => Some(IrStatement::FocusPrevious),
        "prevent_default" => Some(IrStatement::PreventDefault),
        "stop_propagation" => Some(IrStatement::StopPropagation),
        _ => None,
    }
}

/// Extracts an identity-targeted scroll request and its two coordinates.
///
/// `node` is a checked handler expression. Returns the source ID and expression
/// syntax only when it is a three-argument `scroll_to()` call.
pub(super) fn scroll_to_call(node: &SyntaxNode) -> Option<(String, SyntaxNode, SyntaxNode)> {
    let call = node
        .children()
        .find(|child| child.kind() == SyntaxKind::CallExpr)?;
    let callee = call
        .descendants()
        .find(|child| child.kind() == SyntaxKind::PathExpr)
        .and_then(|path| direct_identifier(&path))?;
    if callee != "scroll_to" {
        return None;
    }
    let mut arguments = call
        .children()
        .find(|child| child.kind() == SyntaxKind::ArgumentList)?
        .children()
        .filter(|child| child.kind() == SyntaxKind::Expr);
    let target = arguments.next()?;
    let literal = target
        .children()
        .find(|child| child.kind() == SyntaxKind::LiteralExpr)?;
    let mut tokens = literal
        .children_with_tokens()
        .filter_map(|child| child.into_token())
        .filter(|token| !token.kind().is_trivia());
    (tokens.next()?.kind() == SyntaxKind::Hash).then_some(())?;
    let name = tokens.next()?.text().to_string();
    let x = arguments.next()?;
    let y = arguments.next()?;
    arguments.next().is_none().then_some((name, x, y))
}

/// Extracts the checked mode expression from a `set_theme_mode()` handler call.
///
/// `node` is the complete event statement expression. Returns the argument
/// expression only when this is a mode-switch call.
pub(super) fn theme_mode_call(node: &SyntaxNode) -> Option<SyntaxNode> {
    let call = node
        .children()
        .find(|child| child.kind() == SyntaxKind::CallExpr)?;
    let callee = call
        .descendants()
        .find(|child| child.kind() == SyntaxKind::PathExpr)
        .and_then(|path| direct_identifier(&path))?;
    if callee != "set_theme_mode" {
        return None;
    }
    call.children()
        .find(|child| child.kind() == SyntaxKind::ArgumentList)?
        .children()
        .find(|child| child.kind() == SyntaxKind::Expr)
}
