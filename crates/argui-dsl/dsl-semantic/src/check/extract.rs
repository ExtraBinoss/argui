use std::collections::HashMap;

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    Definition, DefinitionKind, Diagnostic, DiagnosticCode, SymbolId, Type,
    lower::{LoweredDefinition, LoweredKind, LoweredModule},
};

use super::Scope;
use declaration::{
    component_definition, effect_definition, enum_definition, function_definition,
    struct_definition, style_definition, theme_definition,
};
use named_cycle::report_named_cycles;
use theme::validate_theme_values;
use visual::validate_component;

mod animation;
mod declaration;
mod named_cycle;
mod style;
mod theme;
mod visual;

pub(super) fn definition(
    lowered: &LoweredDefinition,
    file: FileId,
    scope: &Scope,
    modules: &[LoweredModule],
    diagnostics: &mut Vec<Diagnostic>,
) -> Definition {
    let kind = match lowered.kind {
        LoweredKind::Struct => DefinitionKind::Struct(struct_definition(
            &lowered.syntax,
            file,
            scope,
            modules,
            diagnostics,
        )),
        LoweredKind::Enum => {
            DefinitionKind::Enum(enum_definition(&lowered.syntax, file, diagnostics))
        }
        LoweredKind::Component => DefinitionKind::Component(component_definition(
            &lowered.syntax,
            file,
            scope,
            modules,
            diagnostics,
        )),
        LoweredKind::Theme => DefinitionKind::Theme(theme_definition(
            &lowered.syntax,
            file,
            scope,
            modules,
            diagnostics,
        )),
        LoweredKind::Style => DefinitionKind::Style(style_definition(&lowered.syntax, scope)),
        LoweredKind::Effect => DefinitionKind::Effect(effect_definition(
            &lowered.syntax,
            file,
            scope,
            modules,
            diagnostics,
        )),
        LoweredKind::Function => DefinitionKind::Function(function_definition(
            &lowered.syntax,
            file,
            scope,
            modules,
            diagnostics,
        )),
    };
    Definition {
        id: lowered.id,
        name: lowered.name.clone(),
        exported: lowered.exported,
        span: lowered.span,
        kind,
    }
}

/// Builds the cross-theme token type index used by expressions and styles.
pub(super) fn theme_token_index(
    definitions: &HashMap<SymbolId, Definition>,
) -> HashMap<String, Type> {
    definitions
        .values()
        .filter_map(|definition| match &definition.kind {
            DefinitionKind::Theme(theme) => Some(&theme.tokens),
            _ => None,
        })
        .flatten()
        .map(|token| (token.name.clone(), token.value_type.clone()))
        .collect()
}

/// Runs declaration-body and visual-tree validation for one resolved module.
pub(super) fn validate_module(
    module: &LoweredModule,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    schema: &argui_schema::SchemaRegistry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for lowered in &module.definitions {
        let Some(definition) = definitions.get(&lowered.id) else {
            continue;
        };
        match &definition.kind {
            DefinitionKind::Component(component) => validate_component(
                module.file,
                &lowered.syntax,
                component,
                scope,
                definitions,
                theme_tokens,
                schema,
                diagnostics,
            ),
            DefinitionKind::Style(style) => style::validate(
                &lowered.syntax,
                module.file,
                style,
                definitions,
                theme_tokens,
                schema,
                diagnostics,
            ),
            DefinitionKind::Theme(theme) => validate_theme_values(
                module.file,
                &lowered.syntax,
                theme,
                definitions,
                theme_tokens,
                diagnostics,
            ),
            DefinitionKind::Function(function) => validate_function(
                &lowered.syntax,
                module.file,
                function,
                scope,
                definitions,
                theme_tokens,
                diagnostics,
            ),
            DefinitionKind::Effect(effect)
                if effect
                    .shader
                    .as_ref()
                    .is_none_or(|path| !path.ends_with(".wgsl")) =>
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidEffect,
                    "effect shader must reference a .wgsl file",
                    lowered.span,
                ));
            }
            _ => {}
        }
    }
}

/// Type-checks one pure expression body with only its typed parameters in scope.
///
/// `syntax` and `file` identify the body; `function` is its pre-extracted
/// signature. `scope`, `definitions`, and `theme_tokens` resolve named values;
/// `diagnostics` receives type and effect errors.
fn validate_function(
    syntax: &SyntaxNode,
    file: FileId,
    function: &crate::FunctionDefinition,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(body) = syntax
        .children()
        .find(|node| node.kind() == SyntaxKind::Expr)
    else {
        return;
    };
    for call in body
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::CallExpr)
    {
        let name = call
            .descendants()
            .find(|node| node.kind() == SyntaxKind::PathExpr)
            .and_then(|path| {
                path.children_with_tokens()
                    .filter_map(|element| element.into_token())
                    .find(|token| token.kind() == SyntaxKind::Ident)
            })
            .map(|token| token.text().to_string());
        if name.as_deref().is_some_and(|name| {
            matches!(
                name,
                "set_theme_mode"
                    | "focus_next"
                    | "focus_previous"
                    | "scroll_to"
                    | "prevent_default"
                    | "stop_propagation"
            )
        }) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!(
                    "effectful call `{}` is not allowed in a pure function",
                    name.unwrap()
                ),
                Span::new(file, call.text_range()),
            ));
        }
    }
    let empty_properties = HashMap::new();
    let empty_callbacks = HashMap::new();
    let mut context = super::expression::Context {
        file,
        properties: &empty_properties,
        callbacks: &empty_callbacks,
        locals: function
            .parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.value_type.clone()))
            .collect(),
        symbols: scope.symbols.clone(),
        definitions,
        theme_tokens,
        references: None,
        event_handler: false,
        diagnostics,
    };
    let actual = super::expression::infer(&body, &mut context);
    if !function.result.accepts(&actual) {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!(
                "function returns `{actual}`, expected `{}`",
                function.result
            ),
            Span::new(file, body.text_range()),
        ));
    }
}

/// Rejects direct and mutual calls among pure functions before IR expansion.
///
/// `modules` retain call syntax, `scopes` resolve aliases, and `definitions`
/// identify function declarations. `diagnostics` receives cycle reports.
pub(super) fn function_cycles(
    modules: &[LoweredModule],
    scopes: &[Scope],
    definitions: &HashMap<SymbolId, Definition>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut graph = HashMap::new();
    let mut spans = HashMap::new();
    for (module, scope) in modules.iter().zip(scopes) {
        for lowered in &module.definitions {
            if !matches!(
                definitions.get(&lowered.id).map(|value| &value.kind),
                Some(DefinitionKind::Function(_))
            ) {
                continue;
            }
            let key = lowered.id.to_string();
            let calls = lowered
                .syntax
                .descendants()
                .filter(|node| node.kind() == SyntaxKind::CallExpr)
                .filter_map(|call| {
                    call.descendants()
                        .find(|node| node.kind() == SyntaxKind::PathExpr)
                })
                .filter_map(|path| {
                    path.children_with_tokens()
                        .filter_map(|element| element.into_token())
                        .find(|token| token.kind() == SyntaxKind::Ident)
                })
                .filter_map(|token| scope.symbols.get(token.text()))
                .filter(|id| {
                    matches!(
                        definitions.get(id).map(|value| &value.kind),
                        Some(DefinitionKind::Function(_))
                    )
                })
                .map(ToString::to_string)
                .collect();
            spans.insert(key.clone(), lowered.span);
            graph.insert(key, calls);
        }
    }
    report_named_cycles(
        &graph,
        &spans,
        DiagnosticCode::BindingCycle,
        "function call cycle",
        diagnostics,
    );
}

/// Reports static cycles between component property default bindings.
pub(super) fn binding_cycles(
    definitions: &HashMap<SymbolId, Definition>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for definition in definitions.values() {
        let DefinitionKind::Component(component) = &definition.kind else {
            continue;
        };
        let graph = component
            .properties
            .iter()
            .map(|property| (property.name.clone(), property.dependencies.clone()))
            .collect::<HashMap<_, _>>();
        report_named_cycles(
            &graph,
            &component
                .properties
                .iter()
                .map(|property| (property.name.clone(), property.span))
                .collect(),
            DiagnosticCode::BindingCycle,
            "binding cycle",
            diagnostics,
        );
    }
}

/// Reports cycles between derived theme tokens.
pub(super) fn theme_cycles(
    definitions: &HashMap<SymbolId, Definition>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for definition in definitions.values() {
        let DefinitionKind::Theme(theme) = &definition.kind else {
            continue;
        };
        report_named_cycles(
            &theme
                .tokens
                .iter()
                .map(|token| (token.name.clone(), token.dependencies.clone()))
                .collect(),
            &theme
                .tokens
                .iter()
                .map(|token| (token.name.clone(), token.span))
                .collect(),
            DiagnosticCode::ThemeCycle,
            "theme token cycle",
            diagnostics,
        );
    }
}
