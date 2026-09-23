//! Evaluation of scalar collection, string, and color helpers.

use argui_dsl_ir::BuiltinFunction;

use crate::{DslValue, RuntimeError};

/// Evaluates a scalar helper with already evaluated `arguments`.
///
/// `function` selects the helper and `arguments` are its typed operands.
/// Returns one runtime value. Malformed bytecode or mismatched operands return
/// [`RuntimeError::InvalidBytecode`].
pub(super) fn evaluate(
    function: BuiltinFunction,
    arguments: &[DslValue],
) -> Result<DslValue, RuntimeError> {
    use BuiltinFunction as Function;
    use DslValue as Value;
    let result = match (function, arguments) {
        (Function::IsSome, [value]) => Value::Bool(*value != Value::Null),
        (Function::UnwrapOr, [value, fallback]) => {
            if *value == Value::Null {
                fallback.clone()
            } else {
                value.clone()
            }
        }
        (Function::Contains, [Value::String(text), Value::String(fragment)]) => {
            Value::Bool(text.contains(fragment))
        }
        (Function::Lower, [Value::String(value)]) => Value::String(value.to_lowercase()),
        (Function::Range, [Value::Int(count)]) => {
            Value::Array((0..(*count).clamp(0, 100_000)).map(Value::Int).collect())
        }
        (Function::Slice, [Value::String(value), Value::Int(start), Value::Int(count)]) => {
            Value::String(
                value
                    .chars()
                    .skip(*start.max(&0) as usize)
                    .take(*count.max(&0) as usize)
                    .collect(),
            )
        }
        (Function::Hsv, [hue, saturation, value, alpha]) => {
            let (Some(hue), Some(saturation), Some(value), Some(alpha)) = (
                numeric(hue),
                numeric(saturation),
                numeric(value),
                numeric(alpha),
            ) else {
                return Err(invalid(function));
            };
            Value::Color(argui_core::Color::hsva(hue, saturation, value, alpha))
        }
        (Function::ColorHex, [Value::Color(value)]) => Value::String(value.to_hex_rgba()),
        (
            Function::ColorRed | Function::ColorGreen | Function::ColorBlue,
            [Value::Color(value)],
        ) => {
            let index = match function {
                Function::ColorRed => 0,
                Function::ColorGreen => 1,
                _ => 2,
            };
            Value::Int(i64::from(value.to_srgba8()[index]))
        }
        _ => return Err(invalid(function)),
    };
    Ok(result)
}

/// Converts a DSL float or integer `value` into an f32 color coordinate.
/// Returns `None` for other runtime types.
fn numeric(value: &DslValue) -> Option<f32> {
    match value {
        DslValue::Float(value) => Some(*value as f32),
        DslValue::Int(value) => Some(*value as f32),
        _ => None,
    }
}

/// Reports malformed operands for `function` as invalid bytecode.
fn invalid(function: BuiltinFunction) -> RuntimeError {
    RuntimeError::InvalidBytecode(format!("invalid arguments for {function:?}"))
}
