use std::collections::{HashMap, HashSet};

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    CallbackDefinition, ComponentDefinition, Definition, DefinitionKind, Diagnostic,
    DiagnosticCode, PropertyDefinition, PropertyDirection, SymbolId, Type, types::from_schema,
};

use super::super::{Scope, expression};
use super::animation;
pub(super) mod binding;
mod effect;
mod helpers;
mod repeater;
mod slot;
mod template_slot;
mod virtual_list;
use helpers::*;

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
    slot::references(syntax, file, component, diagnostics);
    let references =
        binding::native_references(syntax, scope, schema, definitions, file, diagnostics, false);
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
        symbols: scope.symbols.clone(),
        file,
        properties: &properties,
        callbacks: &callbacks,
        locals: locals.clone(),
        definitions,
        theme_tokens,
        references: Some(&references),
        event_handler: false,
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
    animation::validate_component(
        syntax,
        file,
        &properties,
        &callbacks,
        definitions,
        theme_tokens,
        &references,
        context.diagnostics,
    );
    template_slot::validate_usage(
        syntax,
        file,
        &component.slots,
        scope,
        schema,
        context.diagnostics,
    );
    let visual_roots = syntax.children().filter(|node| {
        matches!(
            node.kind(),
            SyntaxKind::Element | SyntaxKind::ForExpr | SyntaxKind::IfExpr | SyntaxKind::SlotDecl
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
            &references,
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
    references: &HashMap<String, HashMap<String, Type>>,
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
            references,
            component_properties,
            callbacks,
            locals,
            diagnostics,
        ),
        SyntaxKind::ForExpr => repeater::validate(
            node,
            file,
            scope,
            definitions,
            theme_tokens,
            schema,
            references,
            component_properties,
            callbacks,
            locals,
            diagnostics,
        ),
        SyntaxKind::IfExpr
        | SyntaxKind::Block
        | SyntaxKind::ElseBranch
        | SyntaxKind::SlotDecl
        | SyntaxKind::SlotContent => {
            if node.kind() == SyntaxKind::IfExpr
                && let Some(condition) = node
                    .children()
                    .find(|child| child.kind() == SyntaxKind::Expr)
            {
                let mut context = expression::Context {
                    symbols: scope.symbols.clone(),
                    file,
                    properties: component_properties,
                    callbacks,
                    locals: locals.clone(),
                    definitions,
                    theme_tokens,
                    references: Some(references),
                    event_handler: false,
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
                    references,
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
    references: &HashMap<String, HashMap<String, Type>>,
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
    slot::supplied(node, file, user, diagnostics);
    if let Some(template) =
        user.and_then(|component| component.slots.iter().find(|slot| slot.template))
    {
        virtual_list::validate_template_argument(node, file, &template.name, diagnostics);
        if let Some(parameter) = template.parameters.first() {
            let supplied = node.children().find(|child| {
                child.kind() == SyntaxKind::SlotContent
                    && direct_ident(child).as_deref() == Some(&template.name)
            });
            let body = supplied.as_ref().unwrap_or(node);
            let collection = body
                .descendants()
                .find(|child| child.kind() == SyntaxKind::ForExpr)
                .and_then(|repeater| {
                    repeater
                        .children()
                        .find(|child| child.kind() == SyntaxKind::Expr)
                });
            if let Some(collection) = collection {
                let mut context = expression::Context {
                    symbols: scope.symbols.clone(),
                    file,
                    properties: component_properties,
                    callbacks,
                    locals: locals.clone(),
                    definitions,
                    theme_tokens,
                    references: Some(references),
                    event_handler: false,
                    diagnostics,
                };
                let actual = expression::infer(&collection, &mut context);
                let item = match actual {
                    Type::Array(item) | Type::Model(item) => *item,
                    other => other,
                };
                if item != Type::Unknown && !parameter.value_type.accepts(&item) {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!(
                            "template slot `{}` expects row `{}`, found `{item}`",
                            template.name, parameter.value_type
                        ),
                        Span::new(file, collection.text_range()),
                    ));
                }
            }
        }
    }
    let styled = super::style::applications(node, file, scope, definitions, diagnostics);
    let mut provided = HashSet::new();
    let mut effect_seen = false;
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
                if native.is_some_and(|schema| {
                    schema.properties.iter().any(|property| {
                        property.name.as_str() == property_name && property.read_only
                    })
                }) || user.is_some_and(|component| {
                    component.properties.iter().any(|property| {
                        property.name == property_name
                            && matches!(
                                property.direction,
                                PropertyDirection::Private | PropertyDirection::Output
                            )
                    })
                }) {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::ReadOnlyProperty,
                        format!("`{name}.{property_name}` is read-only"),
                        Span::new(file, child.text_range()),
                    ));
                    continue;
                }
                if let Some(value) = child
                    .children()
                    .find(|value| value.kind() == SyntaxKind::Expr)
                {
                    let mut context = expression::Context {
                        symbols: scope.symbols.clone(),
                        file,
                        properties: component_properties,
                        callbacks,
                        locals: locals.clone(),
                        definitions,
                        theme_tokens,
                        references: Some(references),
                        event_handler: false,
                        diagnostics,
                    };
                    let actual = expression::infer(&value, &mut context);
                    if native.is_some()
                        && binding::drives_layout(&property_name)
                        && (binding::reads_measured_bounds(&value)
                            || expression::property_dependencies(&value, component_properties)
                                .iter()
                                .any(|name| {
                                    component_properties
                                        .get(name)
                                        .is_some_and(|property| property.reads_measured)
                                }))
                    {
                        context.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::BindingCycle,
                            "measured dimensions cannot drive layout geometry; use a container query",
                            Span::new(file, value.text_range()),
                        ));
                    }
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
                        binding::validate_two_way(
                            &value,
                            file,
                            component_properties,
                            &expected,
                            native.is_some_and(|native| {
                                native.properties.iter().any(|property| {
                                    property.name.as_str() == property_name
                                        && property.change_event.is_some()
                                })
                            }) || user.is_some_and(|component| {
                                component.properties.iter().any(|property| {
                                    property.name == property_name
                                        && property.direction == PropertyDirection::InputOutput
                                })
                            }),
                            context.diagnostics,
                        );
                    }
                }
            }
            SyntaxKind::EffectApplication => {
                if effect_seen {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::InvalidEffect,
                        "only one effect may be applied to an element",
                        Span::new(file, child.text_range()),
                    ));
                }
                effect_seen = true;
                effect::validate(
                    &child,
                    file,
                    scope,
                    definitions,
                    theme_tokens,
                    references,
                    component_properties,
                    callbacks,
                    locals,
                    diagnostics,
                );
            }
            SyntaxKind::EventBlock => {
                let event_name = identifier_after(&child, SyntaxKind::OnKw).unwrap_or_default();
                let native_event = native.and_then(|native| {
                    native
                        .events
                        .iter()
                        .find(|event| event.name.as_str() == event_name)
                });
                let component_event = user.and_then(|component| {
                    component
                        .callbacks
                        .iter()
                        .find(|callback| callback.name == event_name)
                });
                if native_event.is_none() && component_event.is_none() {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::UnknownEvent,
                        format!("`{name}` has no event `{event_name}`"),
                        Span::new(file, child.text_range()),
                    ));
                }
                let parameters = event_parameters(&child);
                let expected = native_event
                    .map(|event| {
                        event
                            .payload
                            .map(from_schema)
                            .into_iter()
                            .collect::<Vec<_>>()
                    })
                    .or_else(|| {
                        component_event.map(|callback| {
                            callback
                                .parameters
                                .iter()
                                .map(|parameter| parameter.value_type.clone())
                                .collect::<Vec<_>>()
                        })
                    });
                if let Some(expected) = &expected
                    && parameters.len() != expected.len()
                    && !parameters.is_empty()
                {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!(
                            "event `{event_name}` expects {} bound parameters, found {}",
                            expected.len(),
                            parameters.len()
                        ),
                        Span::new(file, child.text_range()),
                    ));
                }
                let mut event_locals = locals.clone();
                let mut seen = HashSet::new();
                for (index, parameter) in parameters.iter().enumerate() {
                    if !seen.insert(parameter.clone()) {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticCode::DuplicateMember,
                            format!("event parameter `{parameter}` is repeated"),
                            Span::new(file, child.text_range()),
                        ));
                    }
                    event_locals.insert(
                        parameter.clone(),
                        expected
                            .as_ref()
                            .and_then(|types| types.get(index))
                            .cloned()
                            .unwrap_or(Type::Unknown),
                    );
                }
                let mut context = expression::Context {
                    symbols: scope.symbols.clone(),
                    file,
                    properties: component_properties,
                    callbacks,
                    locals: event_locals,
                    definitions,
                    theme_tokens,
                    references: Some(references),
                    event_handler: true,
                    diagnostics,
                };
                let result = component_event.map_or(&Type::Void, |callback| &callback.result);
                crate::check::handler::check(&child, result, &mut context);
            }
            SyntaxKind::Element
            | SyntaxKind::ForExpr
            | SyntaxKind::IfExpr
            | SyntaxKind::SlotContent => validate_visual(
                &child,
                file,
                scope,
                definitions,
                theme_tokens,
                schema,
                references,
                component_properties,
                callbacks,
                locals,
                diagnostics,
            ),
            _ => {}
        }
    }
    animation::validate_element(
        node,
        file,
        &name,
        scope,
        definitions,
        theme_tokens,
        schema,
        references,
        component_properties,
        callbacks,
        locals,
        diagnostics,
    );
    provided.extend(styled);
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
