//! Source-located constraints for the VirtualWindow virtual repeater contract.

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{Diagnostic, DiagnosticCode, SlotDefinition, lower::direct_tokens};

/// Rejects VirtualWindow trees that could not form one stable row per model item.
///
/// `node` is the VirtualWindow element, `file` identifies its source, `slots` are
/// declarations in the enclosing component, and `diagnostics` collects errors.
pub(super) fn validate(
    node: &SyntaxNode,
    file: FileId,
    slots: &[SlotDefinition],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let offset_binding = node.children().find(|child| {
        matches!(
            child.kind(),
            SyntaxKind::PropertyAssignment | SyntaxKind::TwoWayBinding
        ) && direct_tokens(child)
            .find(|token| token.kind() == SyntaxKind::Ident)
            .is_some_and(|token| token.text() == "offset")
    });
    if !offset_binding
        .as_ref()
        .is_some_and(|binding| binding.kind() == SyntaxKind::TwoWayBinding)
    {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidTwoWayBinding,
            "VirtualWindow requires `offset <=> writable_property` so scrolling updates the virtual window",
            Span::new(file, offset_binding.as_ref().unwrap_or(node).text_range()),
        ));
    }
    for assignment in node.children().filter(|child| {
        matches!(
            child.kind(),
            SyntaxKind::PropertyAssignment | SyntaxKind::TwoWayBinding
        )
    }) {
        if direct_tokens(&assignment)
            .any(|token| token.kind() == SyntaxKind::Ident && token.text().starts_with("__"))
        {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnknownProperty,
                "VirtualWindow window metadata is compiler-owned and cannot be assigned",
                Span::new(file, assignment.text_range()),
            ));
        }
    }
    for states in node
        .children()
        .filter(|child| child.kind() == SyntaxKind::StatesBlock)
    {
        for assignment in states
            .descendants()
            .filter(|child| child.kind() == SyntaxKind::PropertyAssignment)
        {
            let structural = direct_tokens(&assignment)
                .find(|token| token.kind() == SyntaxKind::Ident)
                .is_some_and(|token| {
                    matches!(
                        token.text(),
                        "row_height" | "offset" | "viewport_height" | "overscan"
                    )
                });
            if structural {
                diagnostics.push(Diagnostic::error(DiagnosticCode::TypeMismatch,
                    "VirtualWindow structural properties cannot be overridden by `states`; bind a component property instead",
                    Span::new(file, assignment.text_range())));
            }
        }
    }
    let children = node
        .children()
        .filter(|child| {
            matches!(
                child.kind(),
                SyntaxKind::Element
                    | SyntaxKind::ForExpr
                    | SyntaxKind::IfExpr
                    | SyntaxKind::PathExpr
            )
        })
        .collect::<Vec<_>>();
    let [repeater] = children.as_slice() else {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            "VirtualWindow requires exactly one keyed `for` repeater as its visual child",
            Span::new(file, node.text_range()),
        ));
        return;
    };
    if repeater.kind() == SyntaxKind::PathExpr {
        let name = direct_tokens(repeater)
            .find(|token| token.kind() == SyntaxKind::Ident)
            .map(|token| token.text().to_string());
        if !name
            .as_deref()
            .is_some_and(|name| slots.iter().any(|slot| slot.template && slot.name == name))
        {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                "VirtualWindow child must reference a declared `slot ...: template`",
                Span::new(file, repeater.text_range()),
            ));
        }
        return;
    }
    if repeater.kind() != SyntaxKind::ForExpr {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            "VirtualWindow's visual child must be a keyed `for` repeater",
            Span::new(file, repeater.text_range()),
        ));
        return;
    }
    if !direct_tokens(repeater).any(|token| token.kind() == SyntaxKind::KeyKw) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::MissingRepeaterKey,
            "VirtualWindow rows require a stable `key` expression",
            Span::new(file, repeater.text_range()),
        ));
    }
}

/// Validates one caller-provided lazy row repeater for a template slot.
///
/// `node` is the component invocation, `file` identifies its source,
/// `slot_name` labels the expected template, and `diagnostics` receives errors.
pub(super) fn validate_template_argument(
    node: &SyntaxNode,
    file: FileId,
    slot_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let children = node
        .children()
        .filter(|child| {
            matches!(
                child.kind(),
                SyntaxKind::Element
                    | SyntaxKind::ForExpr
                    | SyntaxKind::IfExpr
                    | SyntaxKind::PathExpr
            )
        })
        .collect::<Vec<_>>();
    let [repeater] = children.as_slice() else {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("template slot `{slot_name}` requires exactly one keyed `for` repeater"),
            Span::new(file, node.text_range()),
        ));
        return;
    };
    if repeater.kind() != SyntaxKind::ForExpr {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("template slot `{slot_name}` requires a keyed `for` repeater"),
            Span::new(file, repeater.text_range()),
        ));
    } else if !direct_tokens(repeater).any(|token| token.kind() == SyntaxKind::KeyKw) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::MissingRepeaterKey,
            format!("template slot `{slot_name}` requires a stable `key` expression"),
            Span::new(file, repeater.text_range()),
        ));
    }
}
