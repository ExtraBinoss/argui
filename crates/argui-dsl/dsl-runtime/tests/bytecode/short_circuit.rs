//! Boolean evaluation skips unreachable operands, including their side effects.

use super::*;

/// Logical operators call their right operand exactly when its value is needed.
#[test]
fn boolean_operators_short_circuit_callback_effects() {
    for operator in [BinaryOperator::And, BinaryOperator::Or] {
        for left in [false, true] {
            for right in [false, true] {
                let callback = CallbackId::from_raw(12);
                let mut context = Context::default();
                context.callbacks.insert(callback, DslValue::Bool(right));
                let rhs = expression(
                    2,
                    IrType::Bool,
                    IrExpressionKind::CallbackCall {
                        callback,
                        arguments: vec![],
                    },
                );
                let expected = if operator == BinaryOperator::And {
                    left && right
                } else {
                    left || right
                };
                let evaluated = if operator == BinaryOperator::And {
                    left
                } else {
                    !left
                };
                assert_eq!(
                    evaluate(
                        binary(3, IrType::Bool, operator, bool_value(1, left), rhs),
                        &mut context
                    )
                    .unwrap(),
                    DslValue::Bool(expected)
                );
                assert_eq!(context.callback_arguments.len(), usize::from(evaluated));
            }
        }
    }
}

/// Unreachable reads do not fail; reachable invalid reads still report errors.
#[test]
fn short_circuit_suppresses_only_unreachable_errors() {
    for operator in [BinaryOperator::And, BinaryOperator::Or] {
        for left in [false, true] {
            let missing = expression(
                2,
                IrType::Bool,
                IrExpressionKind::PropertyRead(PropertyId::from_raw(99)),
            );
            let program = Program::compile(&binary(
                3,
                IrType::Bool,
                operator,
                bool_value(1, left),
                missing,
            ));
            assert_eq!(program.property_dependencies(), &[PropertyId::from_raw(99)]);
            let result = program.evaluate(&mut Context::default());
            if (operator == BinaryOperator::And && !left)
                || (operator == BinaryOperator::Or && left)
            {
                assert_eq!(result.unwrap(), DslValue::Bool(left));
            } else {
                assert!(matches!(result, Err(RuntimeError::MissingProperty(99))));
            }
        }
    }
}
