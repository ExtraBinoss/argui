//! Numeric boundary behavior shared with compiled AOT fixtures.
use super::*;

/// Checked integer evaluation never wraps and preserves comparisons beyond f64 precision.
#[test]
fn checked_integer_boundaries() {
    let mut context = Context::default();
    for (operator, left, right) in [
        (BinaryOperator::Add, i64::MAX, 1),
        (BinaryOperator::Subtract, i64::MIN, 1),
        (BinaryOperator::Multiply, i64::MAX, 2),
        (BinaryOperator::Divide, i64::MIN, -1),
        (BinaryOperator::Remainder, i64::MIN, -1),
        (BinaryOperator::Divide, 1, 0),
        (BinaryOperator::Remainder, 1, 0),
    ] {
        assert!(
            evaluate(
                binary(3, IrType::Int, operator, int(1, left), int(2, right)),
                &mut context
            )
            .is_err()
        );
    }
    assert!(
        evaluate(
            expression(
                3,
                IrType::Int,
                IrExpressionKind::Unary {
                    operator: UnaryOperator::Negate,
                    operand: Box::new(int(1, i64::MIN)),
                }
            ),
            &mut context
        )
        .is_err()
    );
    for (operator, expected) in [
        (BinaryOperator::Less, true),
        (BinaryOperator::LessEqual, true),
        (BinaryOperator::Greater, false),
        (BinaryOperator::GreaterEqual, false),
    ] {
        assert_eq!(
            evaluate(
                binary(
                    3,
                    IrType::Bool,
                    operator,
                    int(1, i64::MAX - 1),
                    int(2, i64::MAX)
                ),
                &mut context
            )
            .unwrap(),
            DslValue::Bool(expected)
        );
    }
}

/// Floats use the renderer's f32 precision, mixed operands widen, and zero divisors fail.
#[test]
fn floats_match_generated_rust_precision() {
    let mut context = Context::default();
    let sum = binary(
        3,
        IrType::Float,
        BinaryOperator::Add,
        int(1, 16_777_216),
        float(2, 1.0),
    );
    assert_eq!(
        evaluate(sum, &mut context).unwrap(),
        DslValue::Float(16_777_216.0)
    );
    let sum = binary(
        3,
        IrType::Float,
        BinaryOperator::Add,
        float(1, 0.5),
        int(2, 1),
    );
    assert_eq!(evaluate(sum, &mut context).unwrap(), DslValue::Float(1.5));
    for operator in [BinaryOperator::Divide, BinaryOperator::Remainder] {
        assert!(
            evaluate(
                binary(3, IrType::Float, operator, float(1, 1.0), float(2, 0.0)),
                &mut context
            )
            .is_err()
        );
    }
}
