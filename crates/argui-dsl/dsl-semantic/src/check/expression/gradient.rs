//! Shared semantic validation for arbitrary-stop GPU fill constructors.

use super::*;

/// Checks one gradient constructor and returns its brush result type.
///
/// * `name` — linear, radial, or conic built-in name.
/// * `node` — source call used for diagnostics.
/// * `arguments` — parsed, ordered call arguments.
/// * `context` — module symbols and mutable diagnostics.
pub(super) fn check(
    name: &str,
    node: &SyntaxNode,
    arguments: &[SyntaxNode],
    context: &mut Context<'_, '_>,
) -> Type {
    let numeric_count = match name {
        "linear_gradient" => 1,
        "radial_gradient" => 4,
        "conic_gradient" => 3,
        _ => unreachable!("only gradient built-ins reach the gradient checker"),
    };
    let expected_count = numeric_count + 3;
    if arguments.len() != expected_count {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("{name}() expects color and offset arrays, {numeric_count} geometry value(s), and a color space"),
            Span::new(context.file, node.text_range()),
        ));
    }
    for (index, argument) in arguments.iter().enumerate() {
        let expected = match index {
            0 => Type::Array(Box::new(Type::Color)),
            1 => Type::Array(Box::new(Type::Float)),
            index if index == expected_count - 1 => Type::String,
            _ => Type::Float,
        };
        let actual = infer(argument, context);
        if !expected.accepts(&actual) {
            type_mismatch(context, argument, &expected, &actual);
        }
    }
    if let Some(space) = arguments.last().and_then(static_string)
        && !matches!(space.as_str(), "oklab" | "linear-srgb" | "srgb")
    {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("unknown gradient color space `{space}`; expected oklab, linear-srgb, or srgb"),
            Span::new(context.file, node.text_range()),
        ));
    }
    if let (Some(colors), Some(offsets)) = (
        arguments.first().and_then(array_elements),
        arguments.get(1).and_then(array_elements),
    ) {
        if colors.len() != offsets.len() || colors.len() < 2 {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                "gradient colors and offsets must have matching lengths of at least two",
                Span::new(context.file, node.text_range()),
            ));
        }
        let positions = offsets
            .iter()
            .map(|value| value.to_string().trim().parse::<f32>())
            .collect::<Result<Vec<_>, _>>();
        if let Ok(positions) = positions
            && (positions.iter().any(|offset| !(0.0..=1.0).contains(offset))
                || positions.windows(2).any(|pair| pair[0] > pair[1]))
        {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                "gradient offsets must be ordered values between 0 and 1",
                Span::new(context.file, node.text_range()),
            ));
        }
    }
    Type::Brush
}

/// Returns expression children of a literal array, if the argument is one.
fn array_elements(node: &SyntaxNode) -> Option<Vec<SyntaxNode>> {
    let array = node
        .children()
        .find(|child| child.kind() == SyntaxKind::ArrayExpr)?;
    Some(
        array
            .children()
            .filter(|child| child.kind() == SyntaxKind::Expr)
            .collect(),
    )
}
