use std::collections::{HashMap, HashSet};

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    CallbackDefinition, ComponentDefinition, Definition, DefinitionKind, Diagnostic,
    DiagnosticCode, PropertyDefinition, PropertyDirection, SymbolId, ThemeDefinition, Type,
    lower::direct_tokens, types::from_schema,
};

use super::super::{Scope, expression};

/// Validates component defaults and recursively validates visual trees.
#[allow(clippy::too_many_arguments)]
pub(super) fn validate_component(
    file: FileId,
    syntax: &SyntaxNode,
    component: &ComponentDefinition,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    schema: &argui_schema::SchemaRegistry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let properties = component
        .properties
        .iter()
        .cloned()
        .map(|property| (property.name.clone(), property))
        .collect::<HashMap<_, _>>();
    let callbacks = component
        .callbacks
        .iter()
        .cloned()
        .map(|callback| (callback.name.clone(), callback))
        .collect::<HashMap<_, _>>();
    let locals = HashMap::new();
    let mut context = expression::Context {
        file,
        properties: &properties,
        callbacks: &callbacks,
        locals: &locals,
        definitions,
        theme_tokens,
        diagnostics,
    };
    for (property, declaration) in component.properties.iter().zip(
        syntax
            .children()
            .filter(|node| node.kind() == SyntaxKind::PropertyDecl),
    ) {
        if let Some(value) = declaration
            .children()
            .find(|node| node.kind() == SyntaxKind::Expr)
        {
            let actual = expression::infer(&value, &mut context);
            if !property.value_type.accepts(&actual) {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!(
                        "property `{}` expects `{}`, found `{actual}`",
                        property.name, property.value_type
                    ),
                    Span::new(file, value.text_range()),
                ));
            }
        }
    }
    let visual_roots = syntax.children().filter(|node| {
        matches!(
            node.kind(),
            SyntaxKind::Element | SyntaxKind::ForExpr | SyntaxKind::IfExpr
        )
    });
    for root in visual_roots {
        validate_visual(
            &root,
            file,
            scope,
            definitions,
            theme_tokens,
            schema,
            &properties,
            &callbacks,
            &locals,
            context.diagnostics,
        );
    }
}

/// Recursively validates elements, repeaters, conditions, assignments, and events.
#[allow(clippy::too_many_arguments)]
fn validate_visual(
    node: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    schema: &argui_schema::SchemaRegistry,
    component_properties: &HashMap<String, PropertyDefinition>,
    callbacks: &HashMap<String, CallbackDefinition>,
    locals: &HashMap<String, Type>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match node.kind() {
        SyntaxKind::Element => validate_element(
            node,
            file,
            scope,
            definitions,
            theme_tokens,
            schema,
            component_properties,
            callbacks,
            locals,
            diagnostics,
        ),
        SyntaxKind::ForExpr => {
            let collection = node
                .children()
                .find(|child| child.kind() == SyntaxKind::Expr);
            let mut next_locals = locals.clone();
            if let Some(collection) = collection {
                let mut context = expression::Context {
                    file,
                    properties: component_properties,
                    callbacks,
                    locals,
                    definitions,
                    theme_tokens,
                    diagnostics,
                };
                let value = expression::infer(&collection, &mut context);
                let item = match value {
                    Type::Model(item) | Type::Array(item) => *item,
                    Type::Unknown => Type::Unknown,
                    other => {
                        context.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::TypeMismatch,
                            format!(
                                "repeater source must be model<T> or array<T>, found `{other}`"
                            ),
                            Span::new(file, collection.text_range()),
                        ));
                        Type::Unknown
                    }
                };
                if let Some(binding) = identifier_after(node, SyntaxKind::ForKw) {
                    next_locals.insert(binding, item);
                }
            }
            if !has_direct_token(node, SyntaxKind::KeyKw) {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticCode::MissingRepeaterKey,
                    "stateful repeater has no stable `key` expression",
                    Span::new(file, node.text_range()),
                ));
            }
            for child in visual_children(node) {
                validate_visual(
                    &child,
                    file,
                    scope,
                    definitions,
                    theme_tokens,
                    schema,
                    component_properties,
                    callbacks,
                    &next_locals,
                    diagnostics,
                );
            }
        }
        SyntaxKind::IfExpr | SyntaxKind::Block | SyntaxKind::ElseBranch => {
            if node.kind() == SyntaxKind::IfExpr
                && let Some(condition) = node
                    .children()
                    .find(|child| child.kind() == SyntaxKind::Expr)
            {
                let mut context = expression::Context {
                    file,
                    properties: component_properties,
                    callbacks,
                    locals,
                    definitions,
                    theme_tokens,
                    diagnostics,
                };
                let actual = expression::infer(&condition, &mut context);
                if !Type::Bool.accepts(&actual) {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!("condition expects `Bool`, found `{actual}`"),
                        Span::new(file, condition.text_range()),
                    ));
                }
            }
            for child in visual_children(node) {
                validate_visual(
                    &child,
                    file,
                    scope,
                    definitions,
                    theme_tokens,
                    schema,
                    component_properties,
                    callbacks,
                    locals,
                    diagnostics,
                );
            }
        }
        _ => {}
    }
}

/// Validates one element against a native or user-component schema.
#[allow(clippy::too_many_arguments)]
fn validate_element(
    node: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    schema: &argui_schema::SchemaRegistry,
    component_properties: &HashMap<String, PropertyDefinition>,
    callbacks: &HashMap<String, CallbackDefinition>,
    locals: &HashMap<String, Type>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(name) = direct_ident(node) else {
        return;
    };
    let native = scope.natives.get(&name).and_then(|id| schema.schema(*id));
    let user = scope
        .symbols
        .get(&name)
        .and_then(|id| definitions.get(id))
        .and_then(|definition| match &definition.kind {
            DefinitionKind::Component(component) => Some(component),
            _ => None,
        });
    if native.is_none() && user.is_none() {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnknownComponent,
            format!("unknown component or native primitive `{name}`"),
            Span::new(file, node.text_range()),
        ));
    }
    let mut provided = HashSet::new();
    for child in node.children() {
        match child.kind() {
            SyntaxKind::PropertyAssignment | SyntaxKind::TwoWayBinding => {
                let Some(property_name) = direct_ident_or_theme(&child) else {
                    continue;
                };
                if !provided.insert(property_name.clone()) {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::DuplicateMember,
                        format!("property `{property_name}` is assigned more than once"),
                        Span::new(file, child.text_range()),
                    ));
                }
                let expected = native
                    .and_then(|native| {
                        native
                            .properties
                            .iter()
                            .find(|property| property.name.as_str() == property_name)
                            .map(|property| from_schema(property.value_type))
                    })
                    .or_else(|| {
                        user.and_then(|component| {
                            component
                                .properties
                                .iter()
                                .find(|property| property.name == property_name)
                                .map(|property| property.value_type.clone())
                        })
                    });
                let Some(expected) = expected else {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::UnknownProperty,
                        format!("`{name}` has no property `{property_name}`"),
                        Span::new(file, child.text_range()),
                    ));
                    continue;
                };
                if let Some(value) = child
                    .children()
                    .find(|value| value.kind() == SyntaxKind::Expr)
                {
                    let mut context = expression::Context {
                        file,
                        properties: component_properties,
                        callbacks,
                        locals,
                        definitions,
                        theme_tokens,
                        diagnostics,
                    };
                    let actual = expression::infer(&value, &mut context);
                    if !expected.accepts(&actual) {
                        context.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::TypeMismatch,
                            format!(
                                "property `{property_name}` expects `{expected}`, found `{actual}`"
                            ),
                            Span::new(file, value.text_range()),
                        ));
                    }
                    if child.kind() == SyntaxKind::TwoWayBinding {
                        validate_two_way(&value, file, component_properties, context.diagnostics);
                    }
                }
            }
            SyntaxKind::EventBlock => {
                let event_name = identifier_after(&child, SyntaxKind::OnKw).unwrap_or_default();
                let known = native.is_some_and(|native| {
                    native
                        .events
                        .iter()
                        .any(|event| event.name.as_str() == event_name)
                }) || user.is_some_and(|component| {
                    component
                        .callbacks
                        .iter()
                        .any(|callback| callback.name == event_name)
                });
                if !known {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::UnknownEvent,
                        format!("`{name}` has no event `{event_name}`"),
                        Span::new(file, child.text_range()),
                    ));
                }
                for statement in child
                    .children()
                    .filter(|statement| statement.kind() == SyntaxKind::Statement)
                {
                    for value in statement
                        .children()
                        .filter(|value| value.kind() == SyntaxKind::Expr)
                    {
                        let mut context = expression::Context {
                            file,
                            properties: component_properties,
                            callbacks,
                            locals,
                            definitions,
                            theme_tokens,
                            diagnostics,
                        };
                        let _ = expression::infer(&value, &mut context);
                    }
                }
            }
            SyntaxKind::Element | SyntaxKind::ForExpr | SyntaxKind::IfExpr => validate_visual(
                &child,
                file,
                scope,
                definitions,
                theme_tokens,
                schema,
                component_properties,
                callbacks,
                locals,
                diagnostics,
            ),
            _ => {}
        }
    }
    if let Some(native) = native {
        for property in &native.properties {
            if property.required
                && property.default.is_none()
                && !provided.contains(property.name.as_str())
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::MissingProperty,
                    format!("`{name}` requires property `{}`", property.name),
                    Span::new(file, node.text_range()),
                ));
            }
        }
    }
    if let Some(user) = user {
        for property in &user.properties {
            if property.required && !provided.contains(&property.name) {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::MissingProperty,
                    format!("`{name}` requires property `{}`", property.name),
                    Span::new(file, node.text_range()),
                ));
            }
        }
    }
}

/// Validates that a two-way binding targets a writable component property.
fn validate_two_way(
    value: &SyntaxNode,
    file: FileId,
    properties: &HashMap<String, PropertyDefinition>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let target = value
        .descendants()
        .find(|node| node.kind() == SyntaxKind::PathExpr)
        .and_then(|node| direct_ident(&node));
    let writable = target
        .as_ref()
        .and_then(|name| properties.get(name))
        .is_some_and(|property| {
            matches!(
                property.direction,
                PropertyDirection::Private
                    | PropertyDirection::Output
                    | PropertyDirection::InputOutput
            )
        });
    if !writable {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidTwoWayBinding,
            "two-way binding target must be a writable local property",
            Span::new(file, value.text_range()),
        ));
    }
}

/// Type-checks initial theme token values against their declarations.
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
}

/// Returns an element/block's immediate nested visual nodes.
fn visual_children(node: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> + '_ {
    node.children()
        .flat_map(|child| {
            if child.kind() == SyntaxKind::Block {
                child.children().collect::<Vec<_>>()
            } else {
                vec![child]
            }
        })
        .filter(|child| {
            matches!(
                child.kind(),
                SyntaxKind::Element
                    | SyntaxKind::ForExpr
                    | SyntaxKind::IfExpr
                    | SyntaxKind::ElseBranch
                    | SyntaxKind::Block
            )
        })
}

/// Returns whether a node directly owns a token kind.
fn has_direct_token(node: &SyntaxNode, kind: SyntaxKind) -> bool {
    direct_tokens(node).any(|token| token.kind() == kind)
}

/// Returns the first direct identifier token.
fn direct_ident(node: &SyntaxNode) -> Option<String> {
    direct_tokens(node)
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
}

/// Returns the first direct theme-token name.
fn direct_theme(node: &SyntaxNode) -> Option<String> {
    direct_tokens(node)
        .find(|token| token.kind() == SyntaxKind::ThemeName)
        .map(|token| token.text().to_string())
}

/// Returns a direct identifier or theme-token name.
fn direct_ident_or_theme(node: &SyntaxNode) -> Option<String> {
    direct_ident(node).or_else(|| direct_theme(node))
}

/// Returns the first identifier after a direct keyword.
fn identifier_after(node: &SyntaxNode, keyword: SyntaxKind) -> Option<String> {
    let mut seen = false;
    for token in direct_tokens(node) {
        if seen && token.kind() == SyntaxKind::Ident {
            return Some(token.text().to_string());
        }
        seen |= token.kind() == keyword;
    }
    None
}
