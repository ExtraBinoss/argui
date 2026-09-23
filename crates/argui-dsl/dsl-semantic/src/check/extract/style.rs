//! Explicit named-style checking without an implicit cascade.
use super::super::{Scope, expression};
use crate::{
    Definition, DefinitionKind, Diagnostic, DiagnosticCode, PropertyDirection, StyleDefinition,
    SymbolId, Type,
};
use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};
use std::collections::{HashMap, HashSet};

/// Checks `syntax` for `style` in `file` against `definitions`, `tokens` and
/// `schema`; appends localized errors to `diagnostics`. Styles can read tokens
/// and constants, so their expressions have the same meaning at every use site.
#[allow(clippy::too_many_arguments)]
pub(super) fn validate(
    syntax: &SyntaxNode,
    file: FileId,
    style: &StyleDefinition,
    definitions: &HashMap<SymbolId, Definition>,
    tokens: &HashMap<String, Type>,
    schema: &argui_schema::SchemaRegistry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let native = style.native_target.and_then(|id| schema.schema(id));
    let component = style
        .component_target
        .and_then(|id| definitions.get(&id))
        .and_then(|definition| match &definition.kind {
            DefinitionKind::Component(component) => Some(component),
            _ => None,
        });
    if native.is_none() && component.is_none() {
        error(
            diagnostics,
            file,
            syntax,
            DiagnosticCode::UnknownComponent,
            "style target must be a native or DSL component",
        );
        return;
    }
    let properties = HashMap::new();
    let callbacks = HashMap::new();
    let locals = HashMap::new();
    let mut context = expression::Context {
        symbols: HashMap::new(),
        file,
        properties: &properties,
        callbacks: &callbacks,
        locals: locals.clone(),
        definitions,
        theme_tokens: tokens,
        references: None,
        event_handler: false,
        diagnostics,
    };
    for block in syntax
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::Block)
    {
        let mut assigned = HashSet::new();
        for assignment in block
            .children()
            .filter(|node| node.kind() == SyntaxKind::PropertyAssignment)
        {
            let name = name(&assignment).unwrap_or_default();
            if !assigned.insert(name.clone()) {
                error(
                    context.diagnostics,
                    file,
                    &assignment,
                    DiagnosticCode::DuplicateMember,
                    "style property is assigned twice in the same block",
                );
            }
            let expected = native
                .and_then(|native| {
                    native
                        .properties
                        .iter()
                        .find(|property| property.name.as_str() == name)
                })
                .map(|property| {
                    (
                        crate::types::from_schema(property.value_type),
                        property.read_only,
                    )
                })
                .or_else(|| {
                    component
                        .and_then(|component| {
                            component
                                .properties
                                .iter()
                                .find(|property| property.name == name)
                        })
                        .map(|property| {
                            (
                                property.value_type.clone(),
                                matches!(
                                    property.direction,
                                    PropertyDirection::Private | PropertyDirection::Output
                                ),
                            )
                        })
                });
            let Some((expected, readonly)) = expected else {
                error(
                    context.diagnostics,
                    file,
                    &assignment,
                    DiagnosticCode::UnknownProperty,
                    "style assigns an unknown target property",
                );
                continue;
            };
            if readonly {
                error(
                    context.diagnostics,
                    file,
                    &assignment,
                    DiagnosticCode::ReadOnlyProperty,
                    "style cannot assign a read-only property",
                );
            }
            if let Some(value) = assignment
                .children()
                .find(|node| node.kind() == SyntaxKind::Expr)
            {
                let actual = expression::infer(&value, &mut context);
                if !expected.accepts(&actual) {
                    error(
                        context.diagnostics,
                        file,
                        &value,
                        DiagnosticCode::TypeMismatch,
                        format!("style property `{name}` expects `{expected}`, found `{actual}`"),
                    );
                }
            }
        }
    }
    for state in syntax
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::StyleStateDecl)
    {
        if component.is_some()
            || !matches!(
                name(&state).as_deref(),
                Some("hover" | "pressed" | "focus" | "focus_visible")
            )
        {
            error(
                context.diagnostics,
                file,
                &state,
                DiagnosticCode::UnknownName,
                "style states require a native target and hover, pressed, focus, or focus_visible",
            );
        }
    }
}

/// Checks styles supplied by `element` in `file`, resolving through `scope` and
/// `definitions`. Appends unknown, duplicate, or incompatible-use diagnostics,
/// and returns the properties supplied by a matching style's base block.
pub(super) fn applications(
    element: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashSet<String> {
    let mut supplied = HashSet::new();
    let target = name(element).unwrap_or_default();
    for (index, application) in element
        .children()
        .filter(|node| node.kind() == SyntaxKind::StyleApplication)
        .enumerate()
    {
        if index > 0 {
            error(
                diagnostics,
                file,
                &application,
                DiagnosticCode::DuplicateMember,
                "an element accepts one explicit style",
            );
        }
        let style = name(&application)
            .and_then(|name| scope.symbols.get(&name))
            .and_then(|id| definitions.get(id))
            .and_then(|definition| match &definition.kind {
                DefinitionKind::Style(style) => Some(style),
                _ => None,
            });
        match style {
            None => error(
                diagnostics,
                file,
                &application,
                DiagnosticCode::UnknownName,
                "unknown named style",
            ),
            Some(style)
                if !(style.native_target.is_some()
                    && style.native_target == scope.natives.get(&target).copied()
                    || style.component_target.is_some()
                        && style.component_target == scope.symbols.get(&target).copied()) =>
            {
                error(
                    diagnostics,
                    file,
                    &application,
                    DiagnosticCode::TypeMismatch,
                    "style target does not match this element",
                );
            }
            Some(style) => supplied.extend(style.properties.iter().cloned()),
        }
    }
    supplied
}

/// Returns the direct identifier of `node`, or None when syntax is incomplete.
fn name(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens()
        .filter_map(|child| child.into_token())
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
}

/// Appends a `code` and `message` at `node` in `file` to `diagnostics`.
fn error(
    diagnostics: &mut Vec<Diagnostic>,
    file: FileId,
    node: &SyntaxNode,
    code: DiagnosticCode,
    message: impl Into<String>,
) {
    diagnostics.push(Diagnostic::error(
        code,
        message,
        Span::new(file, node.text_range()),
    ));
}
