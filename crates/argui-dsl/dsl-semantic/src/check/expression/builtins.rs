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
