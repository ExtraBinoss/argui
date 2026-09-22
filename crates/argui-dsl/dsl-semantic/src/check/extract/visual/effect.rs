//! Semantic validation of visual effect instances and their typed arguments.

use std::collections::{HashMap, HashSet};

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    CallbackDefinition, Definition, DefinitionKind, Diagnostic, DiagnosticCode, PropertyDefinition,
    SymbolId, Type,
};

use super::super::super::{Scope, expression};

/// Validates a named effect application against its declaration.
///
/// `node` is the parsed application; `file` identifies diagnostic spans;
/// `scope` and `definitions` resolve the effect; the remaining maps type-check
/// parameter expressions in the containing component's lexical scope.
#[allow(clippy::too_many_arguments)]
pub(super) fn validate(
    node: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    references: &HashMap<String, HashMap<String, Type>>,
    properties: &HashMap<String, PropertyDefinition>,
    callbacks: &HashMap<String, CallbackDefinition>,
    locals: &HashMap<String, Type>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(name) = node
        .children_with_tokens()
        .filter_map(|item| item.into_token())
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
    else {
        return;
    };
    let Some(effect) = scope
        .symbols
        .get(&name)
        .and_then(|id| definitions.get(id))
        .and_then(|definition| match &definition.kind {
            DefinitionKind::Effect(effect) => Some(effect),
            _ => None,
        })
    else {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidEffect,
            format!("unknown effect `{name}`"),
            Span::new(file, node.text_range()),
        ));
        return;
    };
    for parameter in &effect.parameters {
        if parameter.value_type == Type::Transform {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidEffect,
                format!(
                    "effect parameter `{}` has type `transform`, which cannot yet be used in a visual effect instance",
                    parameter.name
                ),
                Span::new(file, node.text_range()),
            ));
        }
    }
    let mut provided = HashSet::new();
    for assignment in node
        .children()
        .filter(|child| child.kind() == SyntaxKind::PropertyAssignment)
    {
        let Some(parameter_name) = assignment
            .children_with_tokens()
            .filter_map(|item| item.into_token())
            .find(|token| token.kind() == SyntaxKind::Ident)
            .map(|token| token.text().to_string())
        else {
            continue;
        };
        if !provided.insert(parameter_name.clone()) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateMember,
                format!("effect parameter `{parameter_name}` is assigned more than once"),
                Span::new(file, assignment.text_range()),
            ));
        }
        if parameter_name == "scope" {
            let value = assignment
                .children()
                .find(|child| child.kind() == SyntaxKind::Expr)
                .map(|expr| expr.text().to_string());
            if !matches!(
                value.as_deref().map(str::trim),
                Some(
                    "\"whole\""
                        | "\"background\""
                        | "\"border\""
                        | "\"content\""
                        | "\"text\""
                        | "\"backdrop\""
                )
            ) {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidEffect,
                    "effect scope must be a literal: \"whole\", \"background\", \"border\", \"content\", \"text\", or \"backdrop\"",
                    Span::new(file, assignment.text_range()),
                ));
            }
            continue;
        }
        let Some(parameter) = effect
            .parameters
            .iter()
            .find(|candidate| candidate.name == parameter_name)
        else {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidEffect,
                format!("effect `{name}` has no parameter `{parameter_name}`"),
                Span::new(file, assignment.text_range()),
            ));
            continue;
        };
        if let Some(value) = assignment
            .children()
            .find(|child| child.kind() == SyntaxKind::Expr)
        {
            let mut context = expression::Context {
                file,
                properties,
                callbacks,
                locals,
                definitions,
                theme_tokens,
                references: Some(references),
                event_handler: false,
                diagnostics,
            };
            let actual = expression::infer(&value, &mut context);
            if !parameter.value_type.accepts(&actual) {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!(
                        "effect parameter `{parameter_name}` expects `{}`, found `{actual}`",
                        parameter.value_type
                    ),
                    Span::new(file, value.text_range()),
                ));
            }
        }
    }
    for parameter in &effect.parameters {
        if !parameter.has_default && !provided.contains(&parameter.name) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidEffect,
                format!("effect `{name}` requires parameter `{}`", parameter.name),
                Span::new(file, node.text_range()),
            ));
        }
    }
}
