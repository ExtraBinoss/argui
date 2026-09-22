//! Type checking for theme token declarations and mode overrides.

use std::collections::HashMap;

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{Definition, Diagnostic, DiagnosticCode, SymbolId, ThemeDefinition, Type};

use super::super::expression;
use crate::lower::direct_tokens;

/// Type-checks initial theme token values and mode overrides.
///
/// `file` and `syntax` identify the theme declaration, `theme` gives its
/// resolved tokens, `definitions` and `theme_tokens` resolve expressions,
/// and `diagnostics` receives source-located errors.
pub(super) fn validate_theme_values(
    file: FileId,
    syntax: &SyntaxNode,
    theme: &ThemeDefinition,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let empty_properties = HashMap::new();
    let empty_callbacks = HashMap::new();
    let empty_locals = HashMap::new();
    for (definition, node) in theme.tokens.iter().zip(
        syntax
            .children()
            .filter(|node| node.kind() == SyntaxKind::ThemeTokenDecl),
    ) {
        if let Some(value) = node.children().find(|node| node.kind() == SyntaxKind::Expr) {
            let mut context = expression::Context {
                file,
                properties: &empty_properties,
                callbacks: &empty_callbacks,
                locals: &empty_locals,
                definitions,
                theme_tokens,
                references: None,
                event_handler: false,
                diagnostics,
            };
            let actual = expression::infer(&value, &mut context);
            if !definition.value_type.accepts(&actual) {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!(
                        "theme token `{}` expects `{}`, found `{actual}`",
                        definition.name, definition.value_type
                    ),
                    Span::new(file, value.text_range()),
                ));
            }
        }
    }
    for assignment in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::ThemeModeDecl)
        .flat_map(|mode| mode.descendants())
        .filter(|node| node.kind() == SyntaxKind::PropertyAssignment)
    {
        let name = direct_tokens(&assignment)
            .find(|token| token.kind() == SyntaxKind::ThemeName)
            .map(|token| token.text().to_string());
        let Some(name) = name else { continue };
        let Some(expected) = theme_tokens.get(&name) else {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnknownName,
                format!("unknown theme token `{name}`"),
                Span::new(file, assignment.text_range()),
            ));
            continue;
        };
        let Some(value) = assignment
            .children()
            .find(|node| node.kind() == SyntaxKind::Expr)
        else {
            continue;
        };
        let mut context = expression::Context {
            file,
            properties: &empty_properties,
            callbacks: &empty_callbacks,
            locals: &empty_locals,
            definitions,
            theme_tokens,
            references: None,
            event_handler: false,
            diagnostics,
        };
        let actual = expression::infer(&value, &mut context);
        if !expected.accepts(&actual) {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("theme token `{name}` expects `{expected}`, found `{actual}`"),
                Span::new(file, value.text_range()),
            ));
        }
    }
}
