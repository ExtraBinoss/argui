use argui_dsl_ir::{
    BinaryOperator, BuiltinFunction, CallbackId, FieldId, IrExpression, IrExpressionKind, IrType,
    LocalId, PropertyId, TokenId, UnaryOperator,
};

use crate::{DslValue, RuntimeError};

/// Pre-resolved stack instruction used only by the development runtime.
#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Constant(DslValue),
    Property(PropertyId),
    Local(LocalId),
    Field(FieldId),
    Token(TokenId),
    Asset(argui_dsl_ir::AssetId),
    Builtin {
        function: BuiltinFunction,
        arguments: usize,
    },
    Callback {
        callback: CallbackId,
        arguments: usize,
    },
    Unary(UnaryOperator),
    Binary(BinaryOperator),
    JumpIfFalse(usize),
    Jump(usize),
    Array(usize),
}

/// Immutable typed expression bytecode compiled once per accepted package.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub result_type: IrType,
    pub instructions: Vec<Instruction>,
    property_dependencies: Vec<PropertyId>,
    token_dependencies: Vec<TokenId>,
    contextual: bool,
}

/// Runtime values reachable by stable IDs while evaluating a program.
pub trait EvaluationContext {
    /// Reads a component property by resolved ID.
    fn property(&self, id: PropertyId) -> Option<DslValue>;
    /// Reads a repeater or handler local by resolved ID.
    fn local(&self, id: LocalId) -> Option<DslValue>;
    /// Reads the active theme value by resolved token ID.
    fn token(&self, id: TokenId) -> Option<DslValue>;
    /// Invokes a bound component callback by resolved ID.
    fn callback(&mut self, id: CallbackId, arguments: Vec<DslValue>) -> Option<DslValue>;
    /// Resolves one Fluent message key through the host localization bridge.
    fn translate(&self, id: &str) -> Option<String>;
}

impl Program {
    /// Compiles a fully resolved expression to compact stack instructions.
    ///
    /// * `expression` — typed IR expression containing no unresolved names.
    #[must_use]
    pub fn compile(expression: &IrExpression) -> Self {
        let mut instructions = Vec::new();
        compile_expression(expression, &mut instructions);
        let mut property_dependencies = instructions
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::Property(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        property_dependencies.sort_unstable();
        property_dependencies.dedup();
        let mut token_dependencies = instructions
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::Token(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        token_dependencies.sort_unstable();
        token_dependencies.dedup();
        let contextual = instructions.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::Local(_)
                    | Instruction::Callback { .. }
                    | Instruction::Builtin {
                        function: BuiltinFunction::Translate,
                        ..
                    }
            )
        });
        Self {
            result_type: expression.value_type.clone(),
            instructions,
            property_dependencies,
            token_dependencies,
            contextual,
        }
    }

    /// Returns property IDs whose revisions determine a cacheable result.
    #[must_use]
    pub fn property_dependencies(&self) -> &[PropertyId] {
        &self.property_dependencies
    }

    /// Returns token IDs whose revisions determine a cacheable result.
    #[must_use]
    pub fn token_dependencies(&self) -> &[TokenId] {
        &self.token_dependencies
    }

    /// Returns whether locals or callback side effects forbid persistent caching.
    #[must_use]
    pub const fn is_contextual(&self) -> bool {
        self.contextual
    }

    /// Evaluates this program against one component/local environment.
    ///
    /// * `context` — stable-ID value and callback provider.
    ///
    /// # Errors
    ///
    /// Returns deterministic bytecode/type errors; it never panics on a package.
    pub fn evaluate(&self, context: &mut impl EvaluationContext) -> Result<DslValue, RuntimeError> {
        let mut stack = Vec::<DslValue>::new();
        let mut cursor = 0_usize;
        while let Some(instruction) = self.instructions.get(cursor) {
            cursor += 1;
            match instruction {
                Instruction::Constant(value) => stack.push(value.clone()),
                Instruction::Property(id) => stack.push(
                    context
                        .property(*id)
                        .ok_or(RuntimeError::MissingProperty(id.raw()))?,
                ),
                Instruction::Local(id) => stack.push(context.local(*id).ok_or_else(|| {
                    RuntimeError::InvalidBytecode(format!("local {} is unavailable", id.raw()))
                })?),
                Instruction::Token(id) => stack.push(context.token(*id).ok_or_else(|| {
                    RuntimeError::InvalidBytecode(format!("token {} is unavailable", id.raw()))
                })?),
                Instruction::Asset(id) => stack.push(DslValue::Asset(*id)),
                Instruction::Field(id) => {
                    let base = pop(&mut stack)?;
                    let DslValue::Struct(fields) = base else {
                        return Err(type_error("struct", &base));
                    };
                    stack.push(fields.get(id).cloned().ok_or_else(|| {
                        RuntimeError::InvalidBytecode(format!("field {} is unavailable", id.raw()))
                    })?);
                }
                Instruction::Builtin {
                    function: BuiltinFunction::Translate,
                    arguments,
                } => {
                    let arguments = arguments_from(&mut stack, *arguments)?;
                    let value = arguments
                        .into_iter()
                        .next()
                        .unwrap_or(DslValue::String(String::new()));
                    let DslValue::String(value) = value else {
                        return Err(type_error("string", &value));
                    };
                    let translated = context.translate(&value).unwrap_or(value);
                    stack.push(DslValue::String(translated));
                }
                Instruction::Callback {
                    callback,
                    arguments,
                } => {
                    let arguments = arguments_from(&mut stack, *arguments)?;
                    stack.push(
                        context
                            .callback(*callback, arguments)
                            .unwrap_or(DslValue::Null),
                    );
                }
                Instruction::Unary(operator) => {
                    let value = pop(&mut stack)?;
                    stack.push(unary(*operator, value)?);
                }
                Instruction::Binary(operator) => {
                    let right = pop(&mut stack)?;
                    let left = pop(&mut stack)?;
                    stack.push(binary(*operator, left, right)?);
                }
                Instruction::JumpIfFalse(target) => {
                    let value = pop(&mut stack)?;
                    let DslValue::Bool(value) = value else {
                        return Err(type_error("bool", &value));
                    };
                    if !value {
                        cursor = checked_target(*target, self.instructions.len())?;
                    }
                }
                Instruction::Jump(target) => {
                    cursor = checked_target(*target, self.instructions.len())?;
                }
                Instruction::Array(length) => {
                    let values = arguments_from(&mut stack, *length)?;
                    stack.push(DslValue::Array(values));
                }
            }
        }
        if stack.len() != 1 {
            return Err(RuntimeError::InvalidBytecode(format!(
                "program completed with {} stack values",
                stack.len()
            )));
        }
        stack
            .pop()
            .ok_or_else(|| RuntimeError::InvalidBytecode("empty result stack".into()))
    }
}

/// Emits bytecode recursively and patches conditional jump destinations.
fn compile_expression(expression: &IrExpression, output: &mut Vec<Instruction>) {
    match &expression.kind {
        IrExpressionKind::Constant(value) => {
            output.push(Instruction::Constant(DslValue::constant(value)));
        }
        IrExpressionKind::PropertyRead(id) => output.push(Instruction::Property(*id)),
        IrExpressionKind::LocalRead(id) => output.push(Instruction::Local(*id)),
        IrExpressionKind::FieldRead { base, field } => {
            compile_expression(base, output);
            output.push(Instruction::Field(*field));
        }
        IrExpressionKind::TokenRead(id) => output.push(Instruction::Token(*id)),
        IrExpressionKind::Asset(id) => output.push(Instruction::Asset(*id)),
        IrExpressionKind::BuiltinCall {
            function,
            arguments,
        } => {
            compile_arguments(arguments, output);
            output.push(Instruction::Builtin {
                function: *function,
                arguments: arguments.len(),
            });
        }
        IrExpressionKind::CallbackCall {
            callback,
            arguments,
        } => {
            compile_arguments(arguments, output);
            output.push(Instruction::Callback {
                callback: *callback,
                arguments: arguments.len(),
            });
        }
        IrExpressionKind::Unary { operator, operand } => {
            compile_expression(operand, output);
            output.push(Instruction::Unary(*operator));
        }
        IrExpressionKind::Binary {
            operator,
            left,
            right,
        } => {
            compile_expression(left, output);
            compile_expression(right, output);
            output.push(Instruction::Binary(*operator));
        }
        IrExpressionKind::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            compile_expression(condition, output);
            let false_jump = output.len();
            output.push(Instruction::JumpIfFalse(0));
            compile_expression(then_value, output);
            let end_jump = output.len();
            output.push(Instruction::Jump(0));
            let false_target = output.len();
            compile_expression(else_value, output);
            let end_target = output.len();
            output[false_jump] = Instruction::JumpIfFalse(false_target);
            output[end_jump] = Instruction::Jump(end_target);
        }
        IrExpressionKind::Array(values) => {
            compile_arguments(values, output);
            output.push(Instruction::Array(values.len()));
        }
    }
}

/// Emits arguments in source order.
fn compile_arguments(values: &[IrExpression], output: &mut Vec<Instruction>) {
    for value in values {
        compile_expression(value, output);
    }
}

/// Removes one stack operand or reports malformed bytecode.
fn pop(stack: &mut Vec<DslValue>) -> Result<DslValue, RuntimeError> {
    stack
        .pop()
        .ok_or_else(|| RuntimeError::InvalidBytecode("stack underflow".into()))
}

/// Removes `count` ordered arguments from the stack.
fn arguments_from(stack: &mut Vec<DslValue>, count: usize) -> Result<Vec<DslValue>, RuntimeError> {
    if stack.len() < count {
        return Err(RuntimeError::InvalidBytecode(
            "argument stack underflow".into(),
        ));
    }
    Ok(stack.split_off(stack.len() - count))
}

/// Validates a bytecode jump destination.
fn checked_target(target: usize, length: usize) -> Result<usize, RuntimeError> {
    (target <= length)
        .then_some(target)
        .ok_or_else(|| RuntimeError::InvalidBytecode(format!("jump {target} exceeds {length}")))
}

/// Evaluates a unary operation.
fn unary(operator: UnaryOperator, value: DslValue) -> Result<DslValue, RuntimeError> {
    match (operator, value) {
        (UnaryOperator::Not, DslValue::Bool(value)) => Ok(DslValue::Bool(!value)),
        (UnaryOperator::Negate, DslValue::Int(value)) => Ok(DslValue::Int(-value)),
        (UnaryOperator::Negate, DslValue::Float(value)) => Ok(DslValue::Float(-value)),
        (UnaryOperator::Positive, value @ (DslValue::Int(_) | DslValue::Float(_))) => Ok(value),
        (_, value) => Err(type_error("unary-compatible number/bool", &value)),
    }
}

/// Evaluates one closed binary operation.
fn binary(
    operator: BinaryOperator,
    left: DslValue,
    right: DslValue,
) -> Result<DslValue, RuntimeError> {
    use BinaryOperator as Op;
    match (operator, left, right) {
        (Op::Add, DslValue::Int(left), DslValue::Int(right)) => Ok(DslValue::Int(left + right)),
        (Op::Subtract, DslValue::Int(left), DslValue::Int(right)) => {
            Ok(DslValue::Int(left - right))
        }
        (Op::Multiply, DslValue::Int(left), DslValue::Int(right)) => {
            Ok(DslValue::Int(left * right))
        }
        (Op::Divide, DslValue::Int(left), DslValue::Int(right)) if right != 0 => {
            Ok(DslValue::Int(left / right))
        }
        (Op::Remainder, DslValue::Int(left), DslValue::Int(right)) if right != 0 => {
            Ok(DslValue::Int(left % right))
        }
        (Op::Add, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(left + right))
        }
        (Op::Subtract, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(left - right))
        }
        (Op::Multiply, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(left * right))
        }
        (Op::Divide, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(left / right))
        }
        (Op::Remainder, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(left % right))
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
        ) => Ok(DslValue::Bool(compare(operator, left as f64, right as f64))),
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

/// Creates a consistent type mismatch diagnostic.
fn type_error(expected: &str, actual: &DslValue) -> RuntimeError {
    RuntimeError::TypeMismatch {
        expected: expected.into(),
        actual: actual.type_name().into(),
    }
}
