//! Direct expression operator lookup.

use argui_dsl_syntax::{SyntaxKind, SyntaxNode};

/// Returns the direct operator token owned by an expression node.
pub(super) fn direct_operator(node: &SyntaxNode) -> Option<SyntaxKind> {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .map(|token| token.kind())
        .find(|kind| {
            matches!(
                kind,
                SyntaxKind::Bang
                    | SyntaxKind::Plus
                    | SyntaxKind::Minus
                    | SyntaxKind::Star
                    | SyntaxKind::Slash
                    | SyntaxKind::Percent
                    | SyntaxKind::EqEq
                    | SyntaxKind::BangEq
                    | SyntaxKind::Lt
                    | SyntaxKind::LtEq
                    | SyntaxKind::Gt
                    | SyntaxKind::GtEq
                    | SyntaxKind::AndAnd
                    | SyntaxKind::OrOr
            )
        })
}
