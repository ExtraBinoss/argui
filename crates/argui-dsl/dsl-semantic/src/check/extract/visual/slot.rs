//! Named content projection and default-slot validation.
use super::helpers::direct_ident;
use crate::{ComponentDefinition, Diagnostic, DiagnosticCode};
use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};
use std::collections::{HashMap, HashSet};

/// Checks supplied slot names and ambiguity on `node` in `file` for `component`.
/// Appends diagnostics; ordinary slots contain zero or more visual children.
pub(super) fn supplied(
    node: &SyntaxNode,
    file: FileId,
    component: Option<&ComponentDefinition>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = HashSet::new();
    let mut supplied_counts = HashMap::new();
    for supplied in node
        .children()
        .filter(|node| node.kind() == SyntaxKind::SlotContent)
    {
        let name = direct_ident(&supplied).unwrap_or_default();
        if !seen.insert(name.clone()) {
            issue(
                diagnostics,
                file,
                &supplied,
                "slot is supplied more than once",
            );
        }
        let visuals = supplied
            .children()
            .find(|child| child.kind() == SyntaxKind::Block)
            .into_iter()
            .flat_map(|block| block.children())
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::Element
                        | SyntaxKind::PathExpr
                        | SyntaxKind::ForExpr
                        | SyntaxKind::IfExpr
                )
            })
            .collect::<Vec<_>>();
        let dynamic = visuals.iter().any(|child| {
            matches!(
                child.kind(),
                SyntaxKind::PathExpr | SyntaxKind::ForExpr | SyntaxKind::IfExpr
            )
        });
        supplied_counts.insert(name.clone(), (visuals.len(), dynamic));
        if !component.is_some_and(|component| component.slots.iter().any(|slot| slot.name == name))
        {
            issue(
                diagnostics,
                file,
                &supplied,
                format!("unknown component slot `{name}`"),
            );
        }
    }
    let implicit = node.children().any(|node| {
        matches!(
            node.kind(),
            SyntaxKind::Element | SyntaxKind::PathExpr | SyntaxKind::ForExpr | SyntaxKind::IfExpr
        )
    });
    if implicit && !seen.is_empty() {
        issue(
            diagnostics,
            file,
            node,
            "use named slots for all content when supplying a named slot",
        );
    }
    if implicit && component.is_some_and(|component| component.slots.is_empty()) {
        issue(
            diagnostics,
            file,
            node,
            "component does not declare a content slot",
        );
    }
    if let Some(component) = component {
        for (index, slot) in component.slots.iter().enumerate() {
            let count = supplied_counts.get(&slot.name).copied().or_else(|| {
                (index == 0 && implicit && seen.is_empty()).then(|| {
                    let visuals = node
                        .children()
                        .filter(|child| {
                            matches!(
                                child.kind(),
                                SyntaxKind::Element
                                    | SyntaxKind::PathExpr
                                    | SyntaxKind::ForExpr
                                    | SyntaxKind::IfExpr
                            )
                        })
                        .collect::<Vec<_>>();
                    let dynamic = visuals.iter().any(|child| {
                        matches!(
                            child.kind(),
                            SyntaxKind::PathExpr | SyntaxKind::ForExpr | SyntaxKind::IfExpr
                        )
                    });
                    (visuals.len(), dynamic)
                })
            });
            if slot.required && count.is_none_or(|(count, _)| count == 0) {
                issue(
                    diagnostics,
                    file,
                    node,
                    format!("required slot `{}` is missing", slot.name),
                );
            }
            if slot.single && count.is_some_and(|(count, _)| count > 1) {
                issue(
                    diagnostics,
                    file,
                    node,
                    format!("slot `{}` accepts one visual child", slot.name),
                );
            }
            if slot.single && count.is_some_and(|(_, dynamic)| dynamic) {
                issue(
                    diagnostics,
                    file,
                    node,
                    format!(
                        "slot `{}` cannot prove single-child cardinality with dynamic content",
                        slot.name
                    ),
                );
            }
        }
    }
}

/// Checks references and default cycles in `syntax` for `component` in `file`.
/// Appends unknown-reference and cyclic-default diagnostics before lowering.
pub(super) fn references(
    syntax: &SyntaxNode,
    file: FileId,
    component: &ComponentDefinition,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let references = syntax
        .descendants()
        .filter(is_reference)
        .collect::<Vec<_>>();
    for reference in &references {
        let name = direct_ident(reference).unwrap_or_default();
        if !component.slots.iter().any(|slot| slot.name == name) {
            issue(
                diagnostics,
                file,
                reference,
                format!("unknown content slot `{name}`"),
            );
        }
    }
    let mut graph = HashMap::new();
    for declaration in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::SlotDecl)
    {
        let name = direct_ident(&declaration).unwrap_or_default();
        if declaration
            .children()
            .any(|node| node.kind() == SyntaxKind::Block)
            && component
                .slots
                .iter()
                .any(|slot| slot.name == name && slot.template)
        {
            issue(
                diagnostics,
                file,
                &declaration,
                "lazy template slots require caller-supplied row content",
            );
        }
        let dependencies = declaration
            .descendants()
            .filter(is_reference)
            .filter_map(|node| direct_ident(&node))
            .collect();
        graph.insert(name, dependencies);
    }
    super::super::named_cycle::report_named_cycles(
        &graph,
        &component
            .slots
            .iter()
            .map(|slot| (slot.name.clone(), slot.span))
            .collect(),
        DiagnosticCode::BindingCycle,
        "slot default cycle",
        diagnostics,
    );
}

/// Returns whether `node` is a bare visual slot reference, excluding expressions.
fn is_reference(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::PathExpr
        && node
            .parent()
            .is_some_and(|parent| matches!(parent.kind(), SyntaxKind::Element | SyntaxKind::Block))
}

/// Appends `message` at `node` in `file` to `diagnostics`.
fn issue(
    diagnostics: &mut Vec<Diagnostic>,
    file: FileId,
    node: &SyntaxNode,
    message: impl Into<String>,
) {
    diagnostics.push(Diagnostic::error(
        DiagnosticCode::TypeMismatch,
        message,
        Span::new(file, node.text_range()),
    ));
}
