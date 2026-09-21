//! Type checking for property animations, independent of visual element kind.

use std::collections::{HashMap, HashSet};

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    CallbackDefinition, Definition, Diagnostic, DiagnosticCode, PropertyDefinition, SymbolId, Type,
    check::expression, lower::direct_tokens, types::from_schema,
};

use super::super::Scope;
mod keyframes;
use keyframes::validate_keyframes;

/// Resolved base and state metadata for an animation on one owner.
struct AnimationTarget<'a> {
    name: &'a str,
    has_binding: bool,
    has_schema_default: bool,
    state_target: bool,
    schema_animatable: bool,
}

/// Validates animations declared directly on a user component.
///
/// * `syntax` — parsed component declaration owning the animations.
/// * `file` — source file used for diagnostic spans.
/// * `properties` — declared target properties and expression bindings.
/// * `callbacks` — callbacks visible in animation expressions.
/// * `definitions` — resolved project definitions.
/// * `theme_tokens` — theme token types visible to expressions.
/// * `diagnostics` — output receiving animation errors.
#[allow(clippy::too_many_arguments)]
pub(super) fn validate_component(
    syntax: &SyntaxNode,
    file: FileId,
    properties: &HashMap<String, PropertyDefinition>,
    callbacks: &HashMap<String, CallbackDefinition>,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let locals = HashMap::new();
    let mut seen = HashSet::new();
    let target_name = direct_tokens(syntax)
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
        .unwrap_or_else(|| "component".to_string());
    for animation in syntax
        .children()
        .filter(|node| node.kind() == SyntaxKind::AnimateDecl)
    {
        validate_unique(&animation, &mut seen, file, diagnostics);
        let mut context = expression::Context {
            file,
            properties,
            callbacks,
            locals: &locals,
            definitions,
            theme_tokens,
            diagnostics,
        };
        let state_target =
            animation_name(&animation).is_some_and(|name| has_state_assignment(syntax, &name));
        validate(
            &animation,
            AnimationTarget {
                name: &target_name,
                has_binding: true,
                has_schema_default: false,
                state_target,
                schema_animatable: true,
            },
            |name| {
                properties
                    .get(name)
                    .map(|property| property.value_type.clone())
            },
            &mut context,
        );
    }
}

/// Validates an animation targeting any native or imported user component.
///
/// * `element` — parsed element owning animation clauses.
/// * `file` — source file used for diagnostic spans.
/// * `target_name` — imported element or component name.
/// * `scope` — resolved names of the containing module.
/// * `definitions` — resolved project definitions.
/// * `theme_tokens` — theme token types visible to expressions.
/// * `schema` — canonical native property schemas.
/// * `component_properties` — local expression bindings.
/// * `callbacks` — callbacks visible to expressions.
/// * `locals` — active repeater bindings.
/// * `diagnostics` — output receiving animation errors.
#[allow(clippy::too_many_arguments)]
pub(super) fn validate_element(
    element: &SyntaxNode,
    file: FileId,
    target_name: &str,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    schema: &argui_schema::SchemaRegistry,
    component_properties: &HashMap<String, PropertyDefinition>,
    callbacks: &HashMap<String, CallbackDefinition>,
    locals: &HashMap<String, Type>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let native = scope
        .natives
        .get(target_name)
        .and_then(|id| schema.schema(*id));
    let user = scope
        .symbols
        .get(target_name)
        .and_then(|id| definitions.get(id))
        .and_then(|definition| match &definition.kind {
            crate::DefinitionKind::Component(component) => Some(component),
            _ => None,
        });
    if native.is_none() && user.is_none() {
        return;
    }
    let mut context = expression::Context {
        file,
        properties: component_properties,
        callbacks,
        locals,
        definitions,
        theme_tokens,
        diagnostics,
    };
    let mut seen = HashSet::new();
    for animation in element
        .children()
        .filter(|node| node.kind() == SyntaxKind::AnimateDecl)
    {
        validate_unique(&animation, &mut seen, file, context.diagnostics);
        let animation_target = animation_name(&animation);
        let has_binding = animation_target.as_ref().is_some_and(|name| {
            user.is_some()
                || element.children().any(|child| {
                    matches!(
                        child.kind(),
                        SyntaxKind::PropertyAssignment | SyntaxKind::TwoWayBinding
                    ) && direct_tokens(&child)
                        .find(|token| token.kind() == SyntaxKind::Ident)
                        .is_some_and(|token| token.text() == name.as_str())
                })
        });
        let has_schema_default = animation_target.as_ref().is_some_and(|name| {
            native.is_some_and(|native| {
                native.properties.iter().any(|property| {
                    property.name.as_str() == name.as_str() && property.default.is_some()
                })
            })
        });
        let state_target = animation_target
            .as_ref()
            .is_some_and(|name| has_state_assignment(element, name));
        let schema_animatable = animation_target.as_ref().is_none_or(|name| {
            native
                .and_then(|native| {
                    native
                        .properties
                        .iter()
                        .find(|property| property.name.as_str() == name.as_str())
                })
                .is_none_or(|property| property.animatable)
        });
        validate(
            &animation,
            AnimationTarget {
                name: target_name,
                has_binding,
                has_schema_default,
                state_target,
                schema_animatable,
            },
            |name| {
                native
                    .and_then(|native| {
                        native
                            .properties
                            .iter()
                            .find(|property| property.name.as_str() == name)
                            .map(|property| from_schema(property.value_type))
                    })
                    .or_else(|| {
                        user.and_then(|component| {
                            component
                                .properties
                                .iter()
                                .find(|property| property.name == name)
                                .map(|property| property.value_type.clone())
                        })
                    })
            },
            &mut context,
        );
    }
}

/// Returns whether a state on this owner assigns the named property.
///
/// * `owner` — component or visual element containing `states` and `animate`.
/// * `name` — property targeted by the animation.
fn has_state_assignment(owner: &SyntaxNode, name: &str) -> bool {
    owner
        .children()
        .filter(|node| node.kind() == SyntaxKind::StatesBlock)
        .flat_map(|states| states.descendants())
        .filter(|node| node.kind() == SyntaxKind::PropertyAssignment)
        .any(|assignment| {
            direct_tokens(&assignment)
                .find(|token| token.kind() == SyntaxKind::Ident)
                .is_some_and(|token| token.text() == name)
        })
}

/// Reports a second animation clause targeting the same property on one owner.
///
/// * `animation` — current animation clause.
/// * `seen` — property names already animated on the owner.
/// * `file` — source file for the duplicate diagnostic.
/// * `diagnostics` — output receiving a duplicate error.
fn validate_unique(
    animation: &SyntaxNode,
    seen: &mut HashSet<String>,
    file: FileId,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(name) = animation_name(animation)
        && !seen.insert(name.clone())
    {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("property `{name}` has more than one animation on the same element"),
            Span::new(file, animation.text_range()),
        ));
    }
}

/// Extracts the property named immediately after `animate`.
///
/// * `animation` — parsed animation clause.
///
/// Returns the target name, if syntax recovery left one available.
fn animation_name(animation: &SyntaxNode) -> Option<String> {
    direct_tokens(animation)
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
}

/// Checks a resolved animation target and its driver parameters.
///
/// * `animation` — parsed animation clause.
/// * `target` — resolved owner name, base-value availability, and state metadata.
/// * `property_type` — resolves a target property name to its semantic type.
/// * `context` — expression bindings and diagnostic output.
fn validate(
    animation: &SyntaxNode,
    target: AnimationTarget<'_>,
    property_type: impl Fn(&str) -> Option<Type>,
    context: &mut expression::Context<'_, '_>,
) {
    let Some(name) = animation_name(animation) else {
        return;
    };
    let Some(value_type) = property_type(&name) else {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnknownProperty,
            format!("`{}` has no animatable property `{name}`", target.name),
            Span::new(context.file, animation.text_range()),
        ));
        return;
    };
    if !value_type.is_interpolable() {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("cannot animate `{name}` of type `{value_type}`; use an interpolable numeric, dimension, angle, or color property"),
            Span::new(context.file, animation.text_range()),
        ));
    }
    if !target.schema_animatable {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("native property `{name}` is structural and cannot be animated"),
            Span::new(context.file, animation.text_range()),
        ));
    }

    let mut seen = HashSet::new();
    let mut infinite = false;
    let mut has_spring = false;
    let mut has_keyframes = false;
    let mut from_unit = None;
    let mut to_unit = None;
    for child in animation.descendants() {
        match child.kind() {
            SyntaxKind::StyleStateDecl => {
                let driver = direct_tokens(&child)
                    .find(|token| token.kind() == SyntaxKind::Ident)
                    .map(|token| token.text().to_string());
                if driver.as_deref() == Some("spring") {
                    has_spring = true;
                } else if let Some(driver) = driver {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::InvalidAnimation,
                        format!("unknown animation driver `{driver}`; expected `spring`"),
                        Span::new(context.file, child.text_range()),
                    ));
                }
            }
            SyntaxKind::KeyframesDecl => {
                if has_keyframes {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::InvalidAnimation,
                        "an animation can declare only one `keyframes` block",
                        Span::new(context.file, child.text_range()),
                    ));
                }
                has_keyframes = true;
                validate_keyframes(&child, &value_type, context);
            }
            SyntaxKind::PropertyAssignment => {
                let parameter = direct_tokens(&child)
                    .find(|token| matches!(token.kind(), SyntaxKind::Ident | SyntaxKind::FromKw))
                    .map(|token| token.text().to_string());
                let Some(parameter) = parameter else { continue };
                if !seen.insert(parameter.clone()) {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::InvalidAnimation,
                        format!("animation parameter `{parameter}` is assigned more than once"),
                        Span::new(context.file, child.text_range()),
                    ));
                }
                if parameter == "transition" {
                    let spelling = child
                        .children()
                        .find(|node| node.kind() == SyntaxKind::Expr)
                        .map(|value| value.text().to_string());
                    if !matches!(
                        spelling.as_deref(),
                        Some(
                            "enter" | "leave" | "in-out" | "\"enter\"" | "\"leave\"" | "\"in-out\""
                        )
                    ) {
                        context.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::InvalidAnimation,
                            "animation `transition` must be the static policy `enter`, `leave`, or `in-out`",
                            Span::new(context.file, child.text_range()),
                        ));
                    }
                    continue;
                }
                let expected = match parameter.as_str() {
                    "from" | "to" => Some(&value_type),
                    "duration" => Some(&Type::Duration),
                    "iterations" | "easing" => Some(&Type::String),
                    "stiffness" | "damping" => Some(&Type::Float),
                    _ => None,
                };
                let Some(expected) = expected else {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::InvalidAnimation,
                        format!("unknown animation parameter `{parameter}`"),
                        Span::new(context.file, child.text_range()),
                    ));
                    continue;
                };
                if let Some(value) = child
                    .children()
                    .find(|node| node.kind() == SyntaxKind::Expr)
                {
                    let spelling = value.text().to_string();
                    let css_easing = matches!(
                        spelling.as_str(),
                        "linear" | "ease" | "ease-in" | "ease-out" | "ease-in-out"
                    );
                    let bare_easing = parameter == "easing"
                        && value
                            .children()
                            .next()
                            .is_some_and(|node| node.kind() == SyntaxKind::PathExpr)
                        && (css_easing
                            || (!context.properties.contains_key(&spelling)
                                && !context.locals.contains_key(&spelling)
                                && !context.theme_tokens.contains_key(&spelling)));
                    let actual = if bare_easing {
                        Type::String
                    } else {
                        expression::infer(&value, context)
                    };
                    if value_type == Type::Dimension {
                        match parameter.as_str() {
                            "from" => from_unit = Some(actual.clone()),
                            "to" => to_unit = Some(actual.clone()),
                            _ => {}
                        }
                    }
                    if !expected.accepts(&actual) {
                        context.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::TypeMismatch,
                            format!("animation `{parameter}` for `{name}` expects `{expected}`, found `{actual}`"),
                            Span::new(context.file, value.text_range()),
                        ));
                    }
                    if parameter == "iterations" {
                        if spelling == "\"infinite\"" {
                            infinite = true;
                        } else if spelling.starts_with('"') && spelling != "\"once\"" {
                            context.diagnostics.push(Diagnostic::error(
                                DiagnosticCode::InvalidAnimation,
                                "animation `iterations` must be `\"once\"` or `\"infinite\"`",
                                Span::new(context.file, value.text_range()),
                            ));
                        }
                    }
                    if parameter == "easing"
                        && (spelling.starts_with('"') || bare_easing)
                        && !matches!(
                            spelling.as_str(),
                            "\"linear\""
                                | "\"ease\""
                                | "\"ease-in\""
                                | "\"ease-out\""
                                | "\"ease-in-out\""
                                | "linear"
                                | "ease"
                                | "ease-in"
                                | "ease-out"
                                | "ease-in-out"
                        )
                    {
                        context.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::InvalidAnimation,
                            "animation `easing` must be `linear`, `ease`, `ease-in`, `ease-out`, or `ease-in-out`",
                            Span::new(context.file, value.text_range()),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    if infinite && !has_keyframes && !(seen.contains("from") && seen.contains("to")) {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("infinite animation of `{name}` requires both `from` and `to`"),
            Span::new(context.file, animation.text_range()),
        ));
    }
    if let (Some(from), Some(to)) = (&from_unit, &to_unit)
        && matches!(from, Type::Length | Type::Percentage)
        && matches!(to, Type::Length | Type::Percentage)
        && from != to
    {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!(
                "animation of `{name}` cannot interpolate between pixel and percentage dimensions"
            ),
            Span::new(context.file, animation.text_range()),
        ));
    }
    let is_spring = has_spring || seen.contains("stiffness") || seen.contains("damping");
    let has_transition = seen.contains("transition");
    if has_transition && !target.state_target {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!(
                "transition of `{name}` requires a state on the same owner assigning that property"
            ),
            Span::new(context.file, animation.text_range()),
        ));
    }
    if has_transition
        && (seen.contains("from")
            || seen.contains("to")
            || has_keyframes
            || seen.contains("iterations"))
    {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!(
                "state transition of `{name}` cannot use `from`, `to`, `keyframes`, or `iterations`"
            ),
            Span::new(context.file, animation.text_range()),
        ));
    }
    if has_transition && target.state_target && !target.has_binding && !target.has_schema_default {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("transition of `{name}` requires a base assignment or schema default"),
            Span::new(context.file, animation.text_range()),
        ));
    }
    if !target.has_binding && !has_keyframes && !seen.contains("to") && !has_transition {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("animation of unbound `{name}` requires a `to` endpoint or `keyframes`"),
            Span::new(context.file, animation.text_range()),
        ));
    }
    if has_keyframes && (seen.contains("from") || seen.contains("to") || is_spring) {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("keyframes of `{name}` cannot be combined with `from`, `to`, or `spring`"),
            Span::new(context.file, animation.text_range()),
        ));
    }
    if is_spring && seen.contains("easing") {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("spring animation of `{name}` cannot use timeline `easing`"),
            Span::new(context.file, animation.text_range()),
        ));
    }
    if is_spring && seen.contains("duration") {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("spring animation of `{name}` cannot use `duration`; spring timing is controlled by `stiffness` and `damping`"),
            Span::new(context.file, animation.text_range()),
        ));
    }
    if (seen.contains("from") || seen.contains("to") || has_keyframes || has_transition)
        && !seen.contains("duration")
        && !is_spring
    {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            format!("timeline animation of `{name}` requires `duration`"),
            Span::new(context.file, animation.text_range()),
        ));
    }
}
