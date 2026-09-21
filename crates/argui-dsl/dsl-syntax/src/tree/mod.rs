use rowan::Language;

use crate::SyntaxKind;

/// Rowan language marker for Argui syntax trees.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArguiLanguage {}

impl Language for ArguiLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::ALL
            .get(usize::from(raw.0))
            .copied()
            .unwrap_or(SyntaxKind::ErrorToken)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

/// Typed Argui syntax node.
pub type SyntaxNode = rowan::SyntaxNode<ArguiLanguage>;
/// Typed Argui syntax token.
pub type SyntaxToken = rowan::SyntaxToken<ArguiLanguage>;
/// Typed node-or-token syntax element.
pub type SyntaxElement = rowan::SyntaxElement<ArguiLanguage>;
