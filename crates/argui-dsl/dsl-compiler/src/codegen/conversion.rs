//! Explicit emission of the widening conversions accepted by the semantic checker.

use crate::{
    CompilerError,
    codegen::{Context, expression::Scope},
};
use argui_dsl_ir::{IrExpression, IrType};

impl Context<'_> {
    /// Emits `value` in `scope` converted to its checked destination `expected`.
    /// Returns typed Rust, or the underlying expression's generation error.
    pub(super) fn expression_as(
        &self,
        value: &IrExpression,
        expected: &IrType,
        scope: &Scope,
    ) -> Result<String, CompilerError> {
        let code = self.expression(value, scope)?;
        Ok(convert(code, &value.value_type, expected))
    }
}

/// Wraps Rust `code` converting semantic `actual` into compatible `expected`.
/// Identical types retain the original expression; containers convert elements.
fn convert(code: String, actual: &IrType, expected: &IrType) -> String {
    if actual == expected {
        return code;
    }
    match (actual, expected) {
        (IrType::Int, IrType::Float) => format!("({code}) as f32"),
        (IrType::Optional(inner), IrType::Optional(_)) if **inner == IrType::Unknown => code,
        (IrType::Optional(actual), IrType::Optional(expected)) => {
            let inner = convert("value".into(), actual, expected);
            format!("({code}).map(|value| {inner})")
        }
        (actual, IrType::Optional(expected)) => {
            format!("Some({})", convert(code, actual, expected))
        }
        (IrType::Array(actual), IrType::Array(expected)) => {
            let inner = convert("value".into(), actual, expected);
            format!("({code}).into_iter().map(|value| {inner}).collect::<Vec<_>>()")
        }
        _ => code,
    }
}
