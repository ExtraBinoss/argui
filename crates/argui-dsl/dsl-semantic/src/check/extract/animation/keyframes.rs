//! Validation of ordered, typed timeline keyframe stops.

use argui_dsl_syntax::{Span, SyntaxKind, SyntaxNode};

use crate::{Diagnostic, DiagnosticCode, Type, check::expression, lower::direct_tokens};

/// Checks ordered percentage stops and typed values for one keyframe block.
///
/// * `block` — parsed keyframes declaration.
/// * `value_type` — target property's interpolable type.
/// * `context` — expression bindings and diagnostic sink.
pub(super) fn validate_keyframes(
    block: &SyntaxNode,
    value_type: &Type,
    context: &mut expression::Context<'_, '_>,
) {
    let mut previous = None;
    let mut first_offset = None;
    let mut count = 0_usize;
    let mut dimension_unit = None;
    for frame in block
        .children()
        .filter(|node| node.kind() == SyntaxKind::KeyframeDecl)
    {
        count += 1;
        let offset = direct_tokens(&frame)
            .find(|token| token.kind() == SyntaxKind::Number)
            .and_then(|token| {
                token
                    .text()
                    .strip_suffix('%')
                    .and_then(|text| text.replace('_', "").parse::<f32>().ok())
            });
        match offset {
            Some(offset) if offset.is_finite() && (0.0..=100.0).contains(&offset) => {
                if count == 1 {
                    first_offset = Some(offset);
                }
                if previous.is_some_and(|last| offset <= last) {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::InvalidAnimation,
                        "keyframe offsets must be strictly increasing percentages",
                        Span::new(context.file, frame.text_range()),
                    ));
                }
                previous = Some(offset);
            }
            _ => context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidAnimation,
                "keyframe offset must be a percentage between 0% and 100%",
                Span::new(context.file, frame.text_range()),
            )),
        }
        if let Some(value) = frame
            .children()
            .find(|node| node.kind() == SyntaxKind::Expr)
        {
            let actual = expression::infer(&value, context);
            if !value_type.accepts(&actual) {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("keyframe expects `{value_type}`, found `{actual}`"),
                    Span::new(context.file, value.text_range()),
                ));
            }
            if *value_type == Type::Dimension && matches!(actual, Type::Length | Type::Percentage) {
                if dimension_unit.as_ref().is_some_and(|unit| unit != &actual) {
                    context.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::InvalidAnimation,
                        "keyframes cannot interpolate between pixel and percentage dimensions",
                        Span::new(context.file, value.text_range()),
                    ));
                }
                dimension_unit = Some(actual);
            }
        }
    }
    if count < 2 {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            "keyframes require at least two stops at 0% and 100%",
            Span::new(context.file, block.text_range()),
        ));
    }
    if first_offset != Some(0.0) || previous != Some(100.0) {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidAnimation,
            "keyframes must start at 0% and end at 100%",
            Span::new(context.file, block.text_range()),
        ));
    }
}
