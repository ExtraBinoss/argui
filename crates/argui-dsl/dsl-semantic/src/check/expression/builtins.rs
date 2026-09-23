//! Type checking for scalar collection, string, and color helpers.

use argui_dsl_syntax::{Span, SyntaxNode};

use crate::{Diagnostic, DiagnosticCode, Type};

use super::{Context, infer, type_mismatch};

/// Checks a supported scalar built-in call.
///
/// `name` identifies the call, `node` gives the diagnostic span, `arguments`
/// supplies its expressions, and `context` collects type errors. Returns the
/// result type for a known built-in or `None` for another callable name.
pub(super) fn check(
    name: &str,
    node: &SyntaxNode,
    arguments: &[SyntaxNode],
    context: &mut Context<'_, '_>,
) -> Option<Type> {
    if name == "is_some" || name == "unwrap_or" {
        return Some(optional(name, node, arguments, context));
    }
    let (expected, result, usage): (Vec<Type>, Type, &str) = match name {
        "contains" => (
            vec![Type::String, Type::String],
            Type::Bool,
            "contains() expects a string and a string fragment",
        ),
        "lower" => (
            vec![Type::String],
            Type::String,
            "lower() expects exactly one string",
        ),
        "range" => (
            vec![Type::Int],
            Type::Array(Box::new(Type::Int)),
            "range() expects exactly one integer count",
        ),
        "slice" => (
            vec![Type::String, Type::Int, Type::Int],
            Type::String,
            "slice() expects a string, start index, and length",
        ),
        "hsv" => (
            vec![Type::Float; 4],
            Type::Color,
            "hsv() expects hue, saturation, value, and alpha",
        ),
        "color_hex" => (
            vec![Type::Color],
            Type::String,
            "color_hex() expects exactly one color",
        ),
        "color_red" | "color_green" | "color_blue" => (
            vec![Type::Color],
            Type::Int,
            "color channel functions expect exactly one color",
        ),
        _ => return None,
    };
    if arguments.len() != expected.len() {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            usage,
            Span::new(context.file, node.text_range()),
        ));
    }
    for (argument, expected) in arguments.iter().zip(expected.iter()) {
        let actual = infer(argument, context);
        if !expected.accepts(&actual) {
            type_mismatch(context, argument, expected, &actual);
        }
    }
    for argument in arguments.iter().skip(expected.len()) {
        infer(argument, context);
    }
    Some(result)
}

/// Checks explicit optional inspection or fallback, including the fallback type.
/// Returns `Unknown` after a malformed call so later checks can continue.
fn optional(
    name: &str,
    node: &SyntaxNode,
    arguments: &[SyntaxNode],
    context: &mut Context<'_, '_>,
) -> Type {
    let expected_count = if name == "is_some" { 1 } else { 2 };
    if arguments.len() != expected_count {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("{name}() expects {expected_count} arguments"),
            Span::new(context.file, node.text_range()),
        ));
    }
    let first = arguments
        .first()
        .map(|argument| infer(argument, context))
        .unwrap_or(Type::Unknown);
    let inner = match first {
        Type::Optional(inner) => *inner,
        Type::Unknown => Type::Unknown,
        other => {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("{name}() requires an optional, found `{other}`"),
                Span::new(context.file, node.text_range()),
            ));
            Type::Unknown
        }
    };
    if name == "is_some" {
        return Type::Bool;
    }
    if let Some(fallback) = arguments.get(1) {
        let actual = infer(fallback, context);
        if !inner.accepts(&actual) {
            type_mismatch(context, fallback, &inner, &actual);
        }
    }
    inner
}
