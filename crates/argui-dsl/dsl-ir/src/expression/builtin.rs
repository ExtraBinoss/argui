//! Lowering of scalar collection, string, gradient, and color functions.

use argui_dsl_syntax::SyntaxNode;

use crate::{BuiltinFunction, IrExpression, IrExpressionKind, IrType, SourceInfo};

use super::{Context, expression_id, lower};

/// Lowers a recognized built-in call to an ID-only expression.
///
/// `name` selects the function, `arguments` are checked syntax expressions,
/// `source` retains its location, and `context` resolves nested expressions.
/// Returns `None` when `name` belongs to another call family.
pub(super) fn lower_call(
    name: &str,
    arguments: &[SyntaxNode],
    source: &SourceInfo,
    context: &mut Context<'_>,
) -> Option<IrExpression> {
    use BuiltinFunction as Function;
    let (function, value_type) = match name {
        "solid" => (Function::Solid, IrType::Brush),
        "linear_gradient" => (Function::LinearGradient, IrType::Brush),
        "radial_gradient" => (Function::RadialGradient, IrType::Brush),
        "conic_gradient" => (Function::ConicGradient, IrType::Brush),
        "contains" => (Function::Contains, IrType::Bool),
        "lower" => (Function::Lower, IrType::String),
        "range" => (Function::Range, IrType::Array(Box::new(IrType::Int))),
        "slice" => (Function::Slice, IrType::String),
        "hsv" => (Function::Hsv, IrType::Color),
        "color_hex" => (Function::ColorHex, IrType::String),
        "color_red" => (Function::ColorRed, IrType::Int),
        "color_green" => (Function::ColorGreen, IrType::Int),
        "color_blue" => (Function::ColorBlue, IrType::Int),
        "str" => (Function::Stringify, IrType::String),
        "tr" => (Function::Translate, IrType::String),
        _ => return None,
    };
    Some(IrExpression {
        id: expression_id(source),
        value_type,
        kind: IrExpressionKind::BuiltinCall {
            function,
            arguments: arguments
                .iter()
                .map(|argument| lower(argument, context))
                .collect(),
        },
        source: source.clone(),
    })
}
