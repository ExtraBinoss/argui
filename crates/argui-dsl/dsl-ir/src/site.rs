use argui_dsl_syntax::{SyntaxKind, SyntaxNode};

use crate::{ComponentId, SiteId, id::derive, id::hash_text};

/// Derives a source-site ID from explicit identity or a stable structural path.
pub(crate) fn identify(component: ComponentId, node: &SyntaxNode) -> SiteId {
    if let Some(explicit) = explicit_id(node) {
        return SiteId::from_raw(derive(
            component.raw(),
            "explicit-site",
            hash_text(&explicit),
        ));
    }
    let mut segments = Vec::new();
    let mut cursor = Some(node.clone());
    while let Some(current) = cursor {
        if current.kind() == SyntaxKind::ComponentDecl {
            break;
        }
        if is_identity_node(current.kind()) {
            let signature = signature(&current);
            let ordinal = same_signature_ordinal(&current, &signature);
            segments.push(format!("{signature}:{ordinal}"));
        }
        cursor = current.parent();
    }
    segments.reverse();
    SiteId::from_raw(derive(
        component.raw(),
        "structural-site",
        hash_text(&segments.join("/")),
    ))
}

/// Returns the optional source-authored element identity.
pub(crate) fn explicit_id(node: &SyntaxNode) -> Option<String> {
    node.children()
        .find(|child| child.kind() == SyntaxKind::ElementId)?
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
}

/// Returns whether a syntax node contributes to retained structural identity.
fn is_identity_node(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Element
            | SyntaxKind::ForExpr
            | SyntaxKind::IfExpr
            | SyntaxKind::ElseBranch
            | SyntaxKind::StateDecl
            | SyntaxKind::AnimateDecl
            | SyntaxKind::PathExpr
    )
}

/// Produces a trivia- and offset-independent identity signature.
fn signature(node: &SyntaxNode) -> String {
    let prefix = match node.kind() {
        SyntaxKind::Element => "element",
        SyntaxKind::ForExpr => "for",
        SyntaxKind::IfExpr => "if",
        SyntaxKind::ElseBranch => "else",
        SyntaxKind::StateDecl => "state",
        SyntaxKind::AnimateDecl => "animate",
        SyntaxKind::PathExpr => "slot",
        _ => "site",
    };
    let name = node
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map_or_else(String::new, |token| token.text().to_string());
    format!("{prefix}:{name}")
}

/// Counts preceding siblings with the same structural signature.
fn same_signature_ordinal(node: &SyntaxNode, expected: &str) -> u64 {
    let mut count = 0_u64;
    let mut sibling = node.prev_sibling();
    while let Some(current) = sibling {
        if is_identity_node(current.kind()) && signature(&current) == expected {
            count += 1;
        }
        sibling = current.prev_sibling();
    }
    count
}
