//! Checked scalar arithmetic shared by expressions and handler mutations.

use super::type_error;
use crate::{DslValue, RuntimeError};
use argui_dsl_ir::{BinaryOperator, UnaryOperator};

/// Applies `operator` to `value`, returning a scalar or a type/overflow error.
pub(super) fn unary(operator: UnaryOperator, value: DslValue) -> Result<DslValue, RuntimeError> {
    match (operator, value) {
        (UnaryOperator::Not, DslValue::Bool(value)) => Ok(DslValue::Bool(!value)),
        (UnaryOperator::Negate, DslValue::Int(value)) => checked_integer(value.checked_neg()),
        (UnaryOperator::Negate, DslValue::Float(value)) => {
            Ok(DslValue::Float(f64::from(-(value as f32))))
        }
        (UnaryOperator::Positive, value @ (DslValue::Int(_) | DslValue::Float(_))) => Ok(value),
        (_, value) => Err(type_error("unary-compatible number/bool", &value)),
    }
}

/// Applies `operator` to `left` and `right`, widening mixed numeric operands.
/// Returns a scalar, or a type, integer-overflow, or zero-divisor error.
pub(crate) fn binary(
    operator: BinaryOperator,
    left: DslValue,
    right: DslValue,
) -> Result<DslValue, RuntimeError> {
    use BinaryOperator as Op;
    match (operator, left, right) {
        (operator, DslValue::Int(left), DslValue::Float(right)) => binary(
            operator,
            DslValue::Float(f64::from(left as f32)),
            DslValue::Float(right),
        ),
        (operator, DslValue::Float(left), DslValue::Int(right)) => binary(
            operator,
            DslValue::Float(left),
            DslValue::Float(f64::from(right as f32)),
        ),
        (Op::Add, DslValue::Int(left), DslValue::Int(right)) => {
            checked_integer(left.checked_add(right))
        }
        (Op::Subtract, DslValue::Int(left), DslValue::Int(right)) => {
            checked_integer(left.checked_sub(right))
        }
        (Op::Multiply, DslValue::Int(left), DslValue::Int(right)) => {
            checked_integer(left.checked_mul(right))
        }
        (Op::Divide, DslValue::Int(left), DslValue::Int(right)) if right != 0 => {
            checked_integer(left.checked_div(right))
        }
        (Op::Remainder, DslValue::Int(left), DslValue::Int(right)) if right != 0 => {
            checked_integer(left.checked_rem(right))
        }
        (Op::Add, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(f64::from((left as f32) + (right as f32))))
        }
        (Op::Subtract, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(f64::from((left as f32) - (right as f32))))
        }
        (Op::Multiply, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(f64::from((left as f32) * (right as f32))))
        }
        (Op::Divide, DslValue::Float(left), DslValue::Float(right)) if right != 0.0 => {
            Ok(DslValue::Float(f64::from((left as f32) / (right as f32))))
        }
        (Op::Remainder, DslValue::Float(left), DslValue::Float(right)) if right != 0.0 => {
            Ok(DslValue::Float(f64::from((left as f32) % (right as f32))))
        }
        (Op::Add, DslValue::String(left), DslValue::String(right)) => {
            Ok(DslValue::String(left + &right))
        }
        (Op::Equal, left, right) => Ok(DslValue::Bool(left == right)),
        (Op::NotEqual, left, right) => Ok(DslValue::Bool(left != right)),
        (Op::And, DslValue::Bool(left), DslValue::Bool(right)) => Ok(DslValue::Bool(left && right)),
        (Op::Or, DslValue::Bool(left), DslValue::Bool(right)) => Ok(DslValue::Bool(left || right)),
        (
            operator @ (Op::Less | Op::LessEqual | Op::Greater | Op::GreaterEqual),
            DslValue::Int(left),
            DslValue::Int(right),
        ) => Ok(DslValue::Bool(match operator {
            Op::Less => left < right,
            Op::LessEqual => left <= right,
            Op::Greater => left > right,
            _ => left >= right,
        })),
        (
            operator @ (Op::Less | Op::LessEqual | Op::Greater | Op::GreaterEqual),
            DslValue::Float(left),
            DslValue::Float(right),
        ) => Ok(DslValue::Bool(compare(operator, left, right))),
        (_, left, right) => Err(RuntimeError::TypeMismatch {
            expected: "compatible binary operands".into(),
            actual: format!("{} and {}", left.type_name(), right.type_name()),
        }),
    }
}

/// Converts a checked integer `value` to a scalar, or reports arithmetic overflow.
fn checked_integer(value: Option<i64>) -> Result<DslValue, RuntimeError> {
    value
        .map(DslValue::Int)
        .ok_or_else(|| RuntimeError::InvalidBytecode("integer arithmetic overflow".into()))
}

/// Evaluates an ordered numeric comparison.
fn compare(operator: BinaryOperator, left: f64, right: f64) -> bool {
    match operator {
        BinaryOperator::Less => left < right,
        BinaryOperator::LessEqual => left <= right,
        BinaryOperator::Greater => left > right,
        BinaryOperator::GreaterEqual => left >= right,
        _ => false,
    }
}
