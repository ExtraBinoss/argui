use std::collections::HashMap;

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    CallbackDefinition, ComponentDefinition, Diagnostic, DiagnosticCode, EffectDefinition,
    EffectParameterDefinition, EnumDefinition, FieldDefinition, PropertyDefinition,
    PropertyDirection, SlotDefinition, StructDefinition, StyleDefinition, ThemeDefinition,
    ThemeTokenDefinition, Type,
    lower::{LoweredKind, LoweredModule, compact_text, direct_tokens, unquote},
};

use super::super::{Scope, expression, is_type_kind, lowered_definition};

/// Extracts typed struct fields and duplicate-member diagnostics.
pub(super) fn struct_definition(
    syntax: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    modules: &[LoweredModule],
    diagnostics: &mut Vec<Diagnostic>,
) -> StructDefinition {
    let mut fields = Vec::new();
    let mut names = HashMap::new();
    for field in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::FieldDecl)
    {
        let Some(name) = direct_ident(&field) else {
            continue;
        };
        let span = Span::new(file, field.text_range());
        duplicate_member(&name, span, &mut names, diagnostics);
        let value_type = field
            .children()
            .find(|node| node.kind() == SyntaxKind::TypeRef)
            .map_or(Type::Unknown, |node| {
                resolve_type(&node, file, scope, modules, diagnostics)
            });
        fields.push(FieldDefinition {
            name,
            value_type,
            span,
        });
    }
    StructDefinition { fields }
}

/// Extracts enum variants and duplicate-member diagnostics.
pub(super) fn enum_definition(
    syntax: &SyntaxNode,
    file: FileId,
    diagnostics: &mut Vec<Diagnostic>,
) -> EnumDefinition {
    let mut variants = Vec::new();
    let mut names = HashMap::new();
    for variant in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::VariantDecl)
    {
        let Some(name) = direct_ident(&variant) else {
            continue;
        };
        let span = Span::new(file, variant.text_range());
        duplicate_member(&name, span, &mut names, diagnostics);
        variants.push((name, span));
    }
    EnumDefinition { variants }
}

/// Extracts component API, binding dependencies, and visual feature counts.
pub(super) fn component_definition(
    syntax: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    modules: &[LoweredModule],
    diagnostics: &mut Vec<Diagnostic>,
) -> ComponentDefinition {
    let mut properties = Vec::new();
    let mut callbacks = Vec::new();
    let mut slots = Vec::new();
    let mut names = HashMap::new();
    for property in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::PropertyDecl)
    {
        let Some(name) = identifier_after(&property, SyntaxKind::PropertyKw) else {
            continue;
        };
        let span = Span::new(file, property.text_range());
        duplicate_member(&name, span, &mut names, diagnostics);
        let value_type = property
            .children()
            .find(|node| node.kind() == SyntaxKind::TypeRef)
            .map_or(Type::Unknown, |node| {
                resolve_type(&node, file, scope, modules, diagnostics)
            });
        let direction = property_direction(&property);
        properties.push(PropertyDefinition {
            name,
            value_type,
            direction,
            required: direction == PropertyDirection::Input
                && !has_direct_token(&property, SyntaxKind::Eq),
            span,
            dependencies: Vec::new(),
        });
    }
    let property_index = properties
        .iter()
        .cloned()
        .map(|property| (property.name.clone(), property))
        .collect::<HashMap<_, _>>();
    for (definition, syntax) in properties.iter_mut().zip(
        syntax
            .children()
            .filter(|node| node.kind() == SyntaxKind::PropertyDecl),
    ) {
        if let Some(expression) = syntax
            .children()
            .find(|node| node.kind() == SyntaxKind::Expr)
        {
            definition.dependencies =
                expression::property_dependencies(&expression, &property_index);
        }
    }
    for callback in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::CallbackDecl)
    {
        let Some(name) = identifier_after(&callback, SyntaxKind::CallbackKw) else {
            continue;
        };
        let span = Span::new(file, callback.text_range());
        duplicate_member(&name, span, &mut names, diagnostics);
        let parameters = callback
            .children()
            .filter(|node| node.kind() == SyntaxKind::CallbackParameter)
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
            .collect();
        let result = callback
            .children()
            .filter(|node| node.kind() == SyntaxKind::TypeRef)
            .last()
            .filter(|_| has_direct_token(&callback, SyntaxKind::Arrow))
            .map_or(Type::Void, |node| {
                resolve_type(&node, file, scope, modules, diagnostics)
            });
        callbacks.push(CallbackDefinition {
            name,
            parameters,
            result,
            span,
        });
    }
    for slot in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::SlotDecl)
    {
        if let Some(name) = identifier_after(&slot, SyntaxKind::SlotKw) {
            let span = Span::new(file, slot.text_range());
            duplicate_member(&name, span, &mut names, diagnostics);
            let kind = direct_tokens(&slot)
                .filter(|token| token.kind() == SyntaxKind::Ident)
                .nth(1);
            if let Some(kind) = &kind
                && kind.text() != "template"
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("unknown slot kind `{}`; expected `template`", kind.text()),
                    Span::new(file, kind.text_range()),
                ));
            }
            slots.push(SlotDefinition {
                name,
                template: kind.is_some_and(|token| token.text() == "template"),
                span,
            });
        }
    }
    if slots.iter().any(|slot| slot.template) && slots.len() != 1 {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            "a template slot must be the component's only slot until named slot arguments are supported",
            Span::new(file, syntax.text_range()),
        ));
    }
    let assets = asset_dependencies(syntax);
    ComponentDefinition {
        properties,
        callbacks,
        slots,
        visual_sites: syntax
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::Element)
            .count(),
        repeaters: syntax
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::ForExpr)
            .count(),
        states: syntax
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::StateDecl)
            .count(),
        animations: syntax
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::AnimateDecl)
            .count(),
        asset_dependencies: assets,
    }
}

/// Extracts typed theme tokens, derived dependencies, and modes.
pub(super) fn theme_definition(
    syntax: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    modules: &[LoweredModule],
    diagnostics: &mut Vec<Diagnostic>,
) -> ThemeDefinition {
    let mut tokens = Vec::new();
    let mut names = HashMap::new();
    for token in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::ThemeTokenDecl)
    {
        let Some(name) = direct_theme(&token) else {
            continue;
        };
        let span = Span::new(file, token.text_range());
        duplicate_member(&name, span, &mut names, diagnostics);
        let value_type = token
            .children()
            .find(|node| node.kind() == SyntaxKind::TypeRef)
            .map_or(Type::Unknown, |node| {
                resolve_type(&node, file, scope, modules, diagnostics)
            });
        let dependencies = token
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .filter(|token| token.kind() == SyntaxKind::ThemeName)
            .map(|token| token.text().to_string())
            .filter(|dependency| dependency != &name)
            .collect();
        tokens.push(ThemeTokenDefinition {
            name,
            value_type,
            dependencies,
            span,
        });
    }
    let modes = syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::ThemeModeDecl)
        .filter_map(|mode| direct_ident(&mode))
        .collect();
    ThemeDefinition { tokens, modes }
}

/// Extracts a style target and assigned property names.
pub(super) fn style_definition(syntax: &SyntaxNode) -> StyleDefinition {
    StyleDefinition {
        target: syntax
            .children()
            .find(|node| node.kind() == SyntaxKind::TypeRef)
            .and_then(|node| direct_ident(&node))
            .unwrap_or_default(),
        properties: syntax
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::PropertyAssignment)
            .filter_map(|assignment| direct_ident_or_theme(&assignment))
            .collect(),
    }
}

/// Extracts an external shader path and typed effect parameter schema.
pub(super) fn effect_definition(
    syntax: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    modules: &[LoweredModule],
    diagnostics: &mut Vec<Diagnostic>,
) -> EffectDefinition {
    let mut shader = None;
    let mut bounded_damage = false;
    let mut seen_damage = false;
    for assignment in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::PropertyAssignment)
    {
        let setting = assignment
            .children_with_tokens()
            .filter_map(|element| element.into_token())
            .find(|token| !token.kind().is_trivia());
        match setting.as_ref().map(|token| token.text()) {
            Some("shader") => {
                shader = assignment
                    .descendants_with_tokens()
                    .filter_map(|element| element.into_token())
                    .find(|token| token.kind() == SyntaxKind::String)
                    .map(|token| unquote(token.text()));
            }
            Some("damage") => {
                if seen_damage {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::DuplicateMember,
                        "effect damage is declared more than once",
                        Span::new(file, assignment.text_range()),
                    ));
                    continue;
                }
                seen_damage = true;
                let value = assignment
                    .descendants_with_tokens()
                    .filter_map(|element| element.into_token())
                    .find(|token| token.kind() == SyntaxKind::String)
                    .map(|token| unquote(token.text()));
                match value.as_deref() {
                    Some("bounded") => bounded_damage = true,
                    Some("unbounded") => {}
                    _ => diagnostics.push(Diagnostic::error(
                        DiagnosticCode::InvalidEffect,
                        "effect damage must be `\"bounded\"` or `\"unbounded\"`",
                        Span::new(file, assignment.text_range()),
                    )),
                }
            }
            _ => {}
        }
    }
    let mut names = HashMap::new();
    let parameters = syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::EffectParameterDecl)
        .filter_map(|parameter| {
            let name = identifier_after(&parameter, SyntaxKind::ParameterKw)?;
            let span = Span::new(file, parameter.text_range());
            if name == "scope" {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidEffect,
                    "`scope` is reserved for selecting an effect's paint region",
                    span,
                ));
            }
            duplicate_member(&name, span, &mut names, diagnostics);
            let value_type = parameter
                .children()
                .find(|node| node.kind() == SyntaxKind::TypeRef)
                .map_or(Type::Unknown, |node| {
                    resolve_type(&node, file, scope, modules, diagnostics)
                });
            Some(EffectParameterDefinition {
                name,
                value_type,
                has_default: parameter
                    .children()
                    .any(|child| child.kind() == SyntaxKind::Expr),
                span,
            })
        })
        .collect();
    EffectDefinition {
        shader,
        bounded_damage,
        parameters,
    }
}

/// Resolves a parsed type reference including generic collection/domain types.
fn resolve_type(
    node: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    modules: &[LoweredModule],
    diagnostics: &mut Vec<Diagnostic>,
) -> Type {
    let text = compact_text(node);
    resolve_type_text(&text, file, node, scope, modules, diagnostics)
}

/// Resolves a compact type spelling recursively.
fn resolve_type_text(
    text: &str,
    file: FileId,
    node: &SyntaxNode,
    scope: &Scope,
    modules: &[LoweredModule],
    diagnostics: &mut Vec<Diagnostic>,
) -> Type {
    if let Some(inner) = text.strip_suffix('?') {
        return Type::Optional(Box::new(resolve_type_text(
            inner,
            file,
            node,
            scope,
            modules,
            diagnostics,
        )));
    }
    if let Some(open) = text.find('<')
        && let Some(inner) = text.strip_suffix('>')
    {
        let base = &text[..open];
        let inner = &inner[open + 1..];
        let value = resolve_type_text(inner, file, node, scope, modules, diagnostics);
        return match base {
            "optional" => Type::Optional(Box::new(value)),
            "array" => Type::Array(Box::new(value)),
            "model" => Type::Model(Box::new(value)),
            _ => {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnknownType,
                    format!("unknown generic type `{base}`"),
                    Span::new(file, node.text_range()),
                ));
                Type::Unknown
            }
        };
    }
    if let Some(builtin) = Type::builtin(text) {
        return builtin;
    }
    if let Some(id) = scope.symbols.get(text).copied()
        && let Some(definition) = lowered_definition(modules, id)
        && is_type_kind(definition.kind)
    {
        return match definition.kind {
            LoweredKind::Struct => Type::Struct(id),
            LoweredKind::Enum => Type::Enum(id),
            _ => Type::Unknown,
        };
    }
    diagnostics.push(Diagnostic::error(
        DiagnosticCode::UnknownType,
        format!("unknown type `{text}`"),
        Span::new(file, node.text_range()),
    ));
    Type::Unknown
}

/// Emits a duplicate-member diagnostic while retaining the first source span.
fn duplicate_member(
    name: &str,
    span: Span,
    names: &mut HashMap<String, Span>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(previous) = names.insert(name.into(), span) {
        diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::DuplicateMember,
                format!("`{name}` is declared more than once"),
                span,
            )
            .related(previous, "first declaration is here"),
        );
    }
}

/// Returns `asset("path")` source dependencies in stable source order.
fn asset_dependencies(node: &SyntaxNode) -> Vec<String> {
    let mut assets = node
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::CallExpr)
        .filter(|call| {
            call.descendants()
                .find(|node| node.kind() == SyntaxKind::PathExpr)
                .and_then(|path| direct_ident(&path))
                .as_deref()
                == Some("asset")
        })
        .filter_map(|call| {
            call.descendants_with_tokens()
                .filter_map(|element| element.into_token())
                .find(|token| token.kind() == SyntaxKind::String)
                .map(|token| unquote(token.text()))
        })
        .collect::<Vec<_>>();
    assets.sort();
    assets.dedup();
    assets
}

/// Returns a component property's declared direction.
fn property_direction(node: &SyntaxNode) -> PropertyDirection {
    if has_direct_token(node, SyntaxKind::InOutKw) {
        PropertyDirection::InputOutput
    } else if has_direct_token(node, SyntaxKind::InKw) {
        PropertyDirection::Input
    } else if has_direct_token(node, SyntaxKind::OutKw) {
        PropertyDirection::Output
    } else {
        PropertyDirection::Private
    }
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
