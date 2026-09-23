//! Syntax helpers shared by visual validation passes.

use argui_dsl_syntax::{SyntaxKind, SyntaxNode};

use crate::lower::direct_tokens;

/// Returns an element/block's immediate nested visual nodes.
pub(super) fn visual_children(node: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> + '_ {
    node.children()
        .flat_map(|child| {
            if child.kind() == SyntaxKind::Block {
                child.children().collect::<Vec<_>>()
            } else {
                vec![child]
            }
        })
        .filter(|child| {
            matches!(
                child.kind(),
                SyntaxKind::Element
                    | SyntaxKind::ForExpr
                    | SyntaxKind::IfExpr
                    | SyntaxKind::ElseBranch
                    | SyntaxKind::Block
                    | SyntaxKind::SlotContent
            )
        })
}

/// Returns whether a node directly owns a token kind.
pub(super) fn has_direct_token(node: &SyntaxNode, kind: SyntaxKind) -> bool {
    direct_tokens(node).any(|token| token.kind() == kind)
}

/// Returns the first direct identifier token.
pub(super) fn direct_ident(node: &SyntaxNode) -> Option<String> {
    direct_tokens(node)
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
}

/// Returns a direct identifier or theme-token name.
pub(super) fn direct_ident_or_theme(node: &SyntaxNode) -> Option<String> {
    direct_ident(node).or_else(|| {
        direct_tokens(node)
            .find(|token| token.kind() == SyntaxKind::ThemeName)
            .map(|token| token.text().to_string())
    })
}

/// Returns the first identifier after a direct keyword.
pub(super) fn identifier_after(node: &SyntaxNode, keyword: SyntaxKind) -> Option<String> {
    let mut seen = false;
    for token in direct_tokens(node) {
        if seen && token.kind() == SyntaxKind::Ident {
            return Some(token.text().to_string());
        }
        seen |= token.kind() == keyword;
    }
    None
}

/// Returns names explicitly bound by one event handler in declaration order.
///
/// * `node` — handler syntax whose direct tokens contain the optional parameter list.
///
/// Returns an empty list when the handler omits parentheses.
pub(super) fn event_parameters(node: &SyntaxNode) -> Vec<String> {
    let mut inside = false;
    let mut parameters = Vec::new();
    for token in direct_tokens(node) {
        match token.kind() {
            SyntaxKind::LParen => inside = true,
            SyntaxKind::RParen => break,
            SyntaxKind::Ident if inside => parameters.push(token.text().to_string()),
            _ => {}
        }
    }
    parameters
}
