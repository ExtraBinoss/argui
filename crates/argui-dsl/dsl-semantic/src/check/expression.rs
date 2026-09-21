use std::collections::HashMap;

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    CallbackDefinition, Definition, DefinitionKind, Diagnostic, DiagnosticCode, PropertyDefinition,
    Type,
};

mod gradient;

pub(super) struct Context<'a, 'd> {
    pub file: FileId,
    pub properties: &'a HashMap<String, PropertyDefinition>,
    pub callbacks: &'a HashMap<String, CallbackDefinition>,
    pub locals: &'a HashMap<String, Type>,
    pub definitions: &'a HashMap<crate::SymbolId, Definition>,
    pub theme_tokens: &'a HashMap<String, Type>,
    pub diagnostics: &'d mut Vec<Diagnostic>,
}

/// Infers and validates one parsed expression tree.
pub(super) fn infer(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    match node.kind() {
        SyntaxKind::Expr => {
            expression_child(node).map_or(Type::Unknown, |child| infer(&child, context))
        }
        SyntaxKind::LiteralExpr => literal(node, context),
        SyntaxKind::PathExpr => path(node, context),
        SyntaxKind::MemberExpr => member(node, context),
        SyntaxKind::CallExpr => call(node, context),
        SyntaxKind::UnaryExpr => unary(node, context),
        SyntaxKind::BinaryExpr => binary(node, context),
        SyntaxKind::ConditionalExpr => conditional(node, context),
        SyntaxKind::ArrayExpr => array(node, context),
        _ => expression_child(node).map_or(Type::Unknown, |child| infer(&child, context)),
    }
}

/// Collects component-property paths referenced by an expression.
pub(super) fn property_dependencies(
    node: &SyntaxNode,
    properties: &HashMap<String, PropertyDefinition>,
) -> Vec<String> {
    let mut dependencies = node
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::PathExpr)
        .filter_map(|path| direct_ident(&path))
        .filter(|name| properties.contains_key(name))
        .collect::<Vec<_>>();
    dependencies.sort();
    dependencies.dedup();
    dependencies
}

/// Infers a literal type and reports malformed numeric spelling at its source span.
fn literal(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let tokens = node
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia())
        .collect::<Vec<_>>();
    let Some(first) = tokens.first() else {
        return Type::Unknown;
    };
    match first.kind() {
        SyntaxKind::String => Type::String,
        SyntaxKind::TrueKw | SyntaxKind::FalseKw => Type::Bool,
        SyntaxKind::NullKw => Type::Optional(Box::new(Type::Unknown)),
        SyntaxKind::Hash => Type::Color,
        SyntaxKind::Number => match numeric_type(first.text()) {
            Some(value_type) => value_type,
            None => {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidNumber,
                    format!("invalid numeric literal `{}`", first.text()),
                    Span::new(context.file, node.text_range()),
                ));
                Type::Unknown
            }
        },
        _ => Type::Unknown,
    }
}

/// Resolves a local, property, callback, or imported symbol path.
fn path(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let Some(name) = direct_ident_or_theme(node) else {
        return Type::Unknown;
    };
    if let Some(value) = context.locals.get(&name) {
        return value.clone();
    }
    if let Some(property) = context.properties.get(&name) {
        return property.value_type.clone();
    }
    if let Some(callback) = context.callbacks.get(&name) {
        return Type::Callback {
            parameters: callback
                .parameters
                .iter()
                .map(|parameter| parameter.value_type.clone())
                .collect(),
            result: Box::new(callback.result.clone()),
        };
    }
    if let Some(value) = context.theme_tokens.get(&name) {
        return value.clone();
    }
    context.diagnostics.push(Diagnostic::error(
        DiagnosticCode::UnknownName,
        format!("unknown name `{name}`"),
        Span::new(context.file, node.text_range()),
    ));
    Type::Unknown
}

/// Resolves a struct member after inferring its base expression.
fn member(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let base = node
        .children()
        .find(|child| is_expression_kind(child.kind()))
        .map_or(Type::Unknown, |child| infer(&child, context));
    let member = node
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() == SyntaxKind::Ident)
        .last()
        .map(|token| token.text().to_string());
    let Some(member) = member else {
        return Type::Unknown;
    };
    let Type::Struct(id) = base else {
        if base != Type::Unknown {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("type `{base}` has no member `{member}`"),
                Span::new(context.file, node.text_range()),
            ));
        }
        return Type::Unknown;
    };
    let Some(Definition {
        kind: DefinitionKind::Struct(definition),
        ..
    }) = context.definitions.get(&id)
    else {
        return Type::Unknown;
    };
    if let Some(field) = definition.fields.iter().find(|field| field.name == member) {
        field.value_type.clone()
    } else {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnknownName,
            format!("struct has no field `{member}`"),
            Span::new(context.file, node.text_range()),
        ));
        Type::Unknown
    }
}

/// Validates built-in and callback calls and returns their result type.
fn call(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let callee_name = node
        .descendants()
        .find(|child| child.kind() == SyntaxKind::PathExpr)
        .and_then(|path| direct_ident(&path));
    let arguments = node
        .children()
        .find(|child| child.kind() == SyntaxKind::ArgumentList)
        .map(|arguments| {
            arguments
                .children()
                .filter(|child| child.kind() == SyntaxKind::Expr)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    match callee_name.as_deref() {
        Some("solid") => {
            if arguments.len() != 1 {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    "solid() expects exactly one color",
                    Span::new(context.file, node.text_range()),
                ));
            }
            for argument in &arguments {
                let actual = infer(argument, context);
                if !Type::Color.accepts(&actual) {
                    type_mismatch(context, argument, &Type::Color, &actual);
                }
            }
            Type::Brush
        }
        Some(name @ ("linear_gradient" | "radial_gradient" | "conic_gradient")) => {
            gradient::check(name, node, &arguments, context)
        }
        Some("contains") => {
            if arguments.len() != 2 {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    "contains() expects a string and a string fragment",
                    Span::new(context.file, node.text_range()),
                ));
            }
            for argument in &arguments {
                let actual = infer(argument, context);
                if !Type::String.accepts(&actual) {
                    type_mismatch(context, argument, &Type::String, &actual);
                }
            }
            Type::Bool
        }
        Some("str") => {
            if arguments.len() != 1 {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    "str() expects exactly one bool, int, float, or string",
                    Span::new(context.file, node.text_range()),
                ));
            }
            for argument in &arguments {
                let actual = infer(argument, context);
                if !matches!(
                    actual,
                    Type::Bool | Type::Int | Type::Float | Type::String | Type::Unknown
                ) {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!("str() cannot convert {actual:?} to string"),
                        Span::new(context.file, argument.text_range()),
                    ));
                }
            }
            Type::String
        }
        Some("set_theme_mode") => {
            if arguments.len() != 1 {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    "set_theme_mode() expects one string mode name",
                    Span::new(context.file, node.text_range()),
                ));
            }
            for argument in &arguments {
                let actual = infer(argument, context);
                if !Type::String.accepts(&actual) {
                    type_mismatch(context, argument, &Type::String, &actual);
                }
            }
            if let Some(mode) = arguments.first().and_then(static_string)
                && !context.definitions.values().any(|definition| {
                    matches!(&definition.kind, DefinitionKind::Theme(theme) if theme.modes.contains(&mode))
                })
            {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnknownName,
                    format!("unknown theme mode `{mode}`"),
                    Span::new(context.file, node.text_range()),
                ));
            }
            Type::Void
        }
        Some("tr") => {
            if arguments.len() != 1 {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    "tr() expects exactly one Fluent message ID",
                    Span::new(context.file, node.text_range()),
                ));
            }
            for argument in arguments {
                let actual = infer(&argument, context);
                if !Type::String.accepts(&actual) {
                    type_mismatch(context, &argument, &Type::String, &actual);
                }
            }
            Type::String
        }
        Some("asset") => {
            if arguments.len() != 1 {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidAsset,
                    "asset() expects exactly one string path",
                    Span::new(context.file, node.text_range()),
                ));
            }
            for argument in arguments {
                let actual = infer(&argument, context);
                if !Type::String.accepts(&actual) {
                    type_mismatch(context, &argument, &Type::String, &actual);
                }
            }
            Type::Asset
        }
        Some("var") => {
            let token = arguments
                .first()
                .and_then(|argument| argument.descendants().find_map(|node| direct_theme(&node)));
            token
                .as_ref()
                .and_then(|name| context.theme_tokens.get(name))
                .cloned()
                .unwrap_or_else(|| {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::UnknownName,
                        format!("unknown theme token `{}`", token.unwrap_or_default()),
                        Span::new(context.file, node.text_range()),
                    ));
                    Type::Unknown
                })
        }
        Some(name) if context.callbacks.contains_key(name) => {
            let callback = context.callbacks[name].clone();
            if callback.parameters.len() != arguments.len() {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!(
                        "callback `{name}` expects {} arguments, received {}",
                        callback.parameters.len(),
                        arguments.len()
                    ),
                    Span::new(context.file, node.text_range()),
                ));
            }
            for (argument, expected) in arguments.iter().zip(&callback.parameters) {
                let actual = infer(argument, context);
                if !expected.value_type.accepts(&actual) {
                    type_mismatch(context, argument, &expected.value_type, &actual);
                }
            }
            callback.result
        }
        _ => {
            for argument in arguments {
                let _ = infer(&argument, context);
            }
            expression_child(node).map_or(Type::Unknown, |callee| infer(&callee, context))
        }
    }
}

/// Validates unary operators.
fn unary(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let operand = expression_child(node).map_or(Type::Unknown, |child| infer(&child, context));
    let operator = direct_operator(node);
    match operator {
        Some(SyntaxKind::Bang) if Type::Bool.accepts(&operand) => Type::Bool,
        Some(SyntaxKind::Plus | SyntaxKind::Minus) if operand.is_numeric() => operand,
        _ => {
            if operand != Type::Unknown {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("invalid unary operation for `{operand}`"),
                    Span::new(context.file, node.text_range()),
                ));
            }
            Type::Unknown
        }
    }
}

/// Validates binary arithmetic, comparison, and boolean operators.
fn binary(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let operands = node
        .children()
        .filter(|child| is_expression_kind(child.kind()))
        .collect::<Vec<_>>();
    let left = operands
        .first()
        .map_or(Type::Unknown, |child| infer(child, context));
    let right = operands
        .get(1)
        .map_or(Type::Unknown, |child| infer(child, context));
    match direct_operator(node) {
        Some(SyntaxKind::AndAnd | SyntaxKind::OrOr) => {
            require(context, node, &Type::Bool, &left);
            require(context, node, &Type::Bool, &right);
            Type::Bool
        }
        Some(SyntaxKind::EqEq | SyntaxKind::BangEq) => {
            if !left.accepts(&right) && !right.accepts(&left) {
                type_mismatch(context, node, &left, &right);
            }
            Type::Bool
        }
        Some(SyntaxKind::Lt | SyntaxKind::LtEq | SyntaxKind::Gt | SyntaxKind::GtEq) => {
            if !(left.is_numeric() && (left.accepts(&right) || right.accepts(&left))) {
                type_mismatch(context, node, &left, &right);
            }
            Type::Bool
        }
        Some(SyntaxKind::Plus) if left == Type::String && right == Type::String => Type::String,
        Some(
            SyntaxKind::Plus
            | SyntaxKind::Minus
            | SyntaxKind::Star
            | SyntaxKind::Slash
            | SyntaxKind::Percent,
        ) => arithmetic(context, node, left, right),
        _ => Type::Unknown,
    }
}

/// Produces the result of unit-preserving arithmetic.
fn arithmetic(context: &mut Context<'_, '_>, node: &SyntaxNode, left: Type, right: Type) -> Type {
    if !left.is_numeric() || !right.is_numeric() {
        type_mismatch(context, node, &left, &right);
        return Type::Unknown;
    }
    if left == right {
        return left;
    }
    if matches!(
        (&left, &right),
        (Type::Float, Type::Int) | (Type::Int, Type::Float)
    ) {
        return Type::Float;
    }
    context.diagnostics.push(Diagnostic::error(
        DiagnosticCode::UnitMismatch,
        format!("cannot combine `{left}` and `{right}` without an explicit conversion"),
        Span::new(context.file, node.text_range()),
    ));
    Type::Unknown
}

/// Validates a ternary conditional expression.
fn conditional(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let values = node
        .children()
        .filter(|child| is_expression_kind(child.kind()))
        .collect::<Vec<_>>();
    let condition = values
        .first()
        .map_or(Type::Unknown, |child| infer(child, context));
    require(context, node, &Type::Bool, &condition);
    let then_type = values
        .get(1)
        .map_or(Type::Unknown, |child| infer(child, context));
    let else_type = values
        .get(2)
        .map_or(Type::Unknown, |child| infer(child, context));
    if then_type.accepts(&else_type) {
        then_type
    } else if else_type.accepts(&then_type) {
        else_type
    } else {
        type_mismatch(context, node, &then_type, &else_type);
        Type::Unknown
    }
}

/// Infers a homogeneous array literal.
fn array(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let mut values = node
        .children()
        .filter(|child| child.kind() == SyntaxKind::Expr);
    let Some(first) = values.next() else {
        return Type::Array(Box::new(Type::Unknown));
    };
    let element = infer(&first, context);
    for value in values {
        let actual = infer(&value, context);
        if !element.accepts(&actual) {
            type_mismatch(context, &value, &element, &actual);
        }
    }
    Type::Array(Box::new(element))
}

/// Parses a numeric spelling into its semantic unit type, returning `None` on failure.
fn numeric_type(text: &str) -> Option<Type> {
    let (value_type, suffix) = if text.ends_with("px") {
        (Type::Length, "px")
    } else if text.ends_with('%') {
        (Type::Percentage, "%")
    } else if text.ends_with("ms") {
        (Type::Duration, "ms")
    } else if text.ends_with("deg") {
        (Type::Angle, "deg")
    } else if text.ends_with("rad") {
        (Type::Angle, "rad")
    } else if text.ends_with('s') {
        (Type::Duration, "s")
    } else if text.contains(['.', 'e', 'E']) {
        (Type::Float, "")
    } else {
        return text.replace('_', "").parse::<i64>().ok().map(|_| Type::Int);
    };
    text.strip_suffix(suffix)?
        .replace('_', "")
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|_| value_type)
}

/// Returns the first nested expression child.
fn expression_child(node: &SyntaxNode) -> Option<SyntaxNode> {
    node.children()
        .find(|child| is_expression_kind(child.kind()))
}

/// Returns whether `kind` carries expression semantics.
fn is_expression_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Expr
            | SyntaxKind::LiteralExpr
            | SyntaxKind::PathExpr
            | SyntaxKind::CallExpr
            | SyntaxKind::MemberExpr
            | SyntaxKind::UnaryExpr
            | SyntaxKind::BinaryExpr
            | SyntaxKind::ConditionalExpr
            | SyntaxKind::ArrayExpr
    )
}

/// Returns the first direct identifier.
fn direct_ident(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
}

/// Returns a compile-time string only when the whole argument is a literal.
///
/// * `node` — parsed call argument expression.
///
/// Returns its unquoted value when it is exactly a string literal.
fn static_string(node: &SyntaxNode) -> Option<String> {
    let literal = expression_child(node)?;
    if literal.kind() != SyntaxKind::LiteralExpr {
        return None;
    }
    literal
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::String)
        .map(|token| token.text().trim_matches('"').to_string())
}

/// Returns the first direct theme-token name.
fn direct_theme(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::ThemeName)
        .map(|token| token.text().to_string())
}

/// Returns a direct identifier or theme-token name.
fn direct_ident_or_theme(node: &SyntaxNode) -> Option<String> {
    direct_ident(node).or_else(|| direct_theme(node))
}

/// Returns the direct operator token owned by an expression node.
fn direct_operator(node: &SyntaxNode) -> Option<SyntaxKind> {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .map(|token| token.kind())
        .find(|kind| {
            matches!(
                kind,
                SyntaxKind::Bang
                    | SyntaxKind::Plus
                    | SyntaxKind::Minus
                    | SyntaxKind::Star
                    | SyntaxKind::Slash
                    | SyntaxKind::Percent
                    | SyntaxKind::EqEq
                    | SyntaxKind::BangEq
                    | SyntaxKind::Lt
                    | SyntaxKind::LtEq
                    | SyntaxKind::Gt
                    | SyntaxKind::GtEq
                    | SyntaxKind::AndAnd
                    | SyntaxKind::OrOr
            )
        })
}

/// Emits a mismatch when `expected` does not accept `actual`.
fn require(context: &mut Context<'_, '_>, node: &SyntaxNode, expected: &Type, actual: &Type) {
    if !expected.accepts(actual) {
        type_mismatch(context, node, expected, actual);
    }
}

/// Emits a typed mismatch diagnostic.
fn type_mismatch(context: &mut Context<'_, '_>, node: &SyntaxNode, expected: &Type, actual: &Type) {
    context.diagnostics.push(Diagnostic::error(
        DiagnosticCode::TypeMismatch,
        format!("expected `{expected}`, found `{actual}`"),
        Span::new(context.file, node.text_range()),
    ));
}
