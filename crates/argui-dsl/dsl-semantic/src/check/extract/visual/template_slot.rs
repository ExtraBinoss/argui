//! Source-located checks for lazy visual template slot placement.

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{Diagnostic, DiagnosticCode, SlotDefinition};

use super::{Scope, direct_ident, virtual_list};

/// Validates native VLists and ensures each lazy slot is forwarded exactly once.
///
/// `syntax` is the defining component, `file` identifies its source, `slots`
/// are its declared content slots, `scope` resolves native aliases, and
/// `diagnostics` receives errors at misplaced or missing references.
pub(super) fn validate_usage(
    syntax: &SyntaxNode,
    file: FileId,
    slots: &[SlotDefinition],
    scope: &Scope,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for element in syntax
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::Element)
    {
        if direct_ident(&element).and_then(|name| scope.natives.get(&name).copied())
            == Some(argui_schema::builtin::VIRTUAL_LIST)
        {
            virtual_list::validate(&element, file, slots, diagnostics);
        }
    }
    for slot in slots.iter().filter(|slot| slot.template) {
        let mut uses = 0;
        for reference in syntax.descendants().filter(|node| {
            node.kind() == SyntaxKind::PathExpr
                && direct_ident(node).as_deref() == Some(slot.name.as_str())
        }) {
            let forwarded = reference.parent().is_some_and(|parent| {
                parent.kind() == SyntaxKind::Element
                    && direct_ident(&parent).and_then(|name| scope.natives.get(&name).copied())
                        == Some(argui_schema::builtin::VIRTUAL_LIST)
            });
            if forwarded {
                uses += 1;
            } else {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!(
                        "template slot `{}` must be forwarded directly as a VList child",
                        slot.name
                    ),
                    Span::new(file, reference.text_range()),
                ));
            }
        }
        if uses != 1 {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!(
                    "template slot `{}` must be forwarded into exactly one native VList",
                    slot.name
                ),
                slot.span,
            ));
        }
    }
}
