//! Extraction of slot cardinality and typed row-template contracts.

use super::*;
use crate::SlotDefinition;

/// Extracts declared slots from `syntax`, resolving parameter types in `scope`
/// and `modules`. `names` tracks duplicate component members, while
/// `diagnostics` receives source-located errors. Returns ordered slots.
#[allow(clippy::too_many_arguments)]
pub(super) fn extract(
    syntax: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    modules: &[LoweredModule],
    names: &mut HashMap<String, Span>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<SlotDefinition> {
    let mut slots = Vec::new();
    for slot in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::SlotDecl)
    {
        if let Some(name) = identifier_after(&slot, SyntaxKind::SlotKw) {
            let span = Span::new(file, slot.text_range());
            duplicate_member(&name, span, names, diagnostics);
            let mut kinds = direct_tokens(&slot)
                .filter(|token| token.kind() == SyntaxKind::Ident)
                .skip(1);
            let kind = kinds.next();
            let modifier = kinds.next();
            if let Some(kind) = &kind
                && !matches!(kind.text(), "template" | "required" | "optional" | "single")
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("unknown slot kind `{}`", kind.text()),
                    Span::new(file, kind.text_range()),
                ));
            }
            if let Some(modifier) = &modifier
                && (!matches!(
                    kind.as_ref().map(|token| token.text()),
                    Some("required" | "optional")
                ) || modifier.text() != "single")
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    "only `required single` and `optional single` combine slot kinds",
                    Span::new(file, modifier.text_range()),
                ));
            }
            let parameters = slot
                .children()
                .filter(|node| node.kind() == SyntaxKind::SlotParameter)
                .filter_map(|parameter| {
                    let name = direct_ident(&parameter)?;
                    let value_type = parameter
                        .children()
                        .find(|node| node.kind() == SyntaxKind::TypeRef)
                        .map_or(Type::Unknown, |node| {
                            resolve_type(&node, file, scope, modules, diagnostics)
                        });
                    Some(FieldDefinition {
                        name,
                        value_type,
                        span: Span::new(file, parameter.text_range()),
                    })
                })
                .collect::<Vec<_>>();
            if !parameters.is_empty()
                && !kind
                    .as_ref()
                    .is_some_and(|token| token.text() == "template")
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    "typed slot parameters require `: template`",
                    span,
                ));
            }
            if parameters.len() > 1 {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    "row templates accept one typed item parameter",
                    span,
                ));
            }
            slots.push(SlotDefinition {
                name,
                template: kind
                    .as_ref()
                    .is_some_and(|token| token.text() == "template"),
                required: kind
                    .as_ref()
                    .is_some_and(|token| token.text() == "required"),
                single: kind.as_ref().is_some_and(|token| token.text() == "single")
                    || modifier.is_some_and(|token| token.text() == "single"),
                parameters,
                span,
            });
        }
    }
    slots
}
