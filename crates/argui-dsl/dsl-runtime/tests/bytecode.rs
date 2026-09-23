use std::collections::{BTreeMap, HashMap};

use argui_dsl_ir::{
    BinaryOperator, BuiltinFunction, CallbackId, ExpressionId, FieldId, IrExpression,
    IrExpressionKind, IrObservation, IrType, IrValue, LocalId, PropertyId, SiteId, SourceInfo,
    TokenId, UnaryOperator,
};
use argui_dsl_runtime::{DslValue, EvaluationContext, Instruction, Program, RuntimeError};
use argui_dsl_syntax::{FileId, Span, TextRange, TextSize};

#[path = "bytecode/arithmetic.rs"]
mod arithmetic;
#[path = "bytecode/builtin.rs"]
mod builtin;

#[path = "bytecode/gradient.rs"]
mod gradient;

#[path = "bytecode/short_circuit.rs"]
mod short_circuit;

fn source() -> SourceInfo {
    SourceInfo::new(
        Span::new(FileId::from_raw(1), TextRange::empty(TextSize::from(0))),
        None,
        None,
    )
}

fn expression(id: u64, value_type: IrType, kind: IrExpressionKind) -> IrExpression {
    IrExpression {
        id: ExpressionId::from_raw(id),
        value_type,
        kind,
        source: source(),
    }
}

fn constant(id: u64, value_type: IrType, value: IrValue) -> IrExpression {
    expression(id, value_type, IrExpressionKind::Constant(value))
}

fn int(id: u64, value: i64) -> IrExpression {
    constant(id, IrType::Int, IrValue::Int(value))
}

fn float(id: u64, value: f64) -> IrExpression {
    constant(id, IrType::Float, IrValue::Float(value))
}

fn string(id: u64, value: &str) -> IrExpression {
    constant(id, IrType::String, IrValue::String(value.into()))
}

fn bool_value(id: u64, value: bool) -> IrExpression {
    constant(id, IrType::Bool, IrValue::Bool(value))
}

fn binary(
    id: u64,
    value_type: IrType,
    operator: BinaryOperator,
    left: IrExpression,
    right: IrExpression,
) -> IrExpression {
    expression(
        id,
        value_type,
        IrExpressionKind::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        },
    )
}

#[derive(Default)]
struct Context {
    properties: HashMap<PropertyId, DslValue>,
    locals: HashMap<LocalId, DslValue>,
    tokens: HashMap<TokenId, DslValue>,
    translated: HashMap<String, String>,
    callbacks: HashMap<CallbackId, DslValue>,
    callback_arguments: Vec<(CallbackId, Vec<DslValue>)>,
    observations: HashMap<SiteId, DslValue>,
}

impl EvaluationContext for Context {
    fn property(&self, id: PropertyId) -> Option<DslValue> {
        self.properties.get(&id).cloned()
    }

    fn observed(&self, site: SiteId, _observation: IrObservation) -> Option<DslValue> {
        self.observations.get(&site).cloned()
    }

    fn local(&self, id: LocalId) -> Option<DslValue> {
        self.locals.get(&id).cloned()
    }

    fn token(&self, id: TokenId) -> Option<DslValue> {
        self.tokens.get(&id).cloned()
    }

    fn callback(&mut self, id: CallbackId, arguments: Vec<DslValue>) -> Option<DslValue> {
        self.callback_arguments.push((id, arguments));
        self.callbacks.get(&id).cloned()
    }

    fn translate(&self, id: &str) -> Option<String> {
        self.translated.get(id).cloned()
    }
}

fn evaluate(expression: IrExpression, context: &mut Context) -> Result<DslValue, RuntimeError> {
    Program::compile(&expression).evaluate(context)
}

fn malformed(instructions: Vec<Instruction>, result_type: IrType) -> Program {
    let mut program = Program::compile(&constant(900, result_type.clone(), IrValue::Null));
    program.result_type = result_type;
    program.instructions = instructions;
    program
}

/// Scalar formatting and substring checks evaluate identically in live bytecode.
#[test]
fn builtin_string_functions_evaluate_and_reject_invalid_values() {
    let mut context = Context::default();
    for (argument, expected) in [
        (int(901, 42), "42"),
        (float(902, 2.5), "2.5"),
        (bool_value(903, true), "true"),
        (string(904, "ready"), "ready"),
    ] {
        let result = evaluate(
            expression(
                905,
                IrType::String,
                IrExpressionKind::BuiltinCall {
                    function: BuiltinFunction::Stringify,
                    arguments: vec![argument],
                },
            ),
            &mut context,
        )
        .unwrap();
        assert_eq!(result, DslValue::String(expected.into()));
    }
    assert_eq!(
        evaluate(
            expression(
                906,
                IrType::Bool,
                IrExpressionKind::BuiltinCall {
                    function: BuiltinFunction::Contains,
                    arguments: vec![string(907, "ada@example.com"), string(908, "@")],
                },
            ),
            &mut context,
        )
        .unwrap(),
        DslValue::Bool(true)
    );
    assert!(matches!(
        evaluate(
            expression(
                909,
                IrType::Bool,
                IrExpressionKind::BuiltinCall {
                    function: BuiltinFunction::Contains,
                    arguments: vec![string(910, "abc"), int(911, 1)],
                },
            ),
            &mut context,
        ),
        Err(RuntimeError::InvalidBytecode(_))
    ));
}

#[test]
fn compiles_and_evaluates_all_expression_shapes() {
    let property = PropertyId::from_raw(3);
    let token = TokenId::from_raw(4);
    let local = LocalId::from_raw(5);
    let field = FieldId::from_raw(6);
    let callback = CallbackId::from_raw(7);
    let mut context = Context {
        properties: HashMap::from([(property, DslValue::Int(9))]),
        locals: HashMap::from([(local, DslValue::String("local".into()))]),
        tokens: HashMap::from([(token, DslValue::Color(argui_core::Color::WHITE))]),
        translated: HashMap::from([(String::from("hello"), String::from("bonjour"))]),
        callbacks: HashMap::from([(callback, DslValue::String("called".into()))]),
        ..Context::default()
    };

    assert_eq!(evaluate(int(1, 1), &mut context).unwrap(), DslValue::Int(1));
    assert_eq!(
        evaluate(
            expression(2, IrType::Int, IrExpressionKind::PropertyRead(property)),
            &mut context,
        )
        .unwrap(),
        DslValue::Int(9)
    );
    assert_eq!(
        evaluate(
            expression(3, IrType::String, IrExpressionKind::LocalRead(local)),
            &mut context,
        )
        .unwrap(),
        DslValue::String("local".into())
    );
    assert_eq!(
        evaluate(
            expression(4, IrType::Color, IrExpressionKind::TokenRead(token)),
            &mut context,
        )
        .unwrap(),
        DslValue::Color(argui_core::Color::WHITE)
    );
    assert_eq!(
        evaluate(
            expression(
                5,
                IrType::String,
                IrExpressionKind::FieldRead {
                    base: Box::new(expression(
                        6,
                        IrType::Struct {
                            symbol: argui_dsl_semantic::SymbolId::derive("m", "struct", "S"),
                            fields: vec![field],
                        },
                        IrExpressionKind::Constant(IrValue::Null),
                    )),
                    field,
                },
            ),
            &mut Context::default(),
        )
        .unwrap_err(),
        RuntimeError::TypeMismatch {
            expected: "struct".into(),
            actual: "null".into(),
        }
    );
    let mut direct = malformed(
        vec![
            Instruction::Constant(DslValue::Struct(BTreeMap::from([(
                field,
                DslValue::String("field".into()),
            )]))),
            Instruction::Field(field),
        ],
        IrType::String,
    );
    assert_eq!(
        direct.evaluate(&mut context).unwrap(),
        DslValue::String("field".into())
    );
    direct.instructions[1] = Instruction::Field(FieldId::from_raw(99));
    assert!(matches!(
        direct.evaluate(&mut context),
        Err(RuntimeError::InvalidBytecode(message)) if message.contains("field 99")
    ));
    assert_eq!(
        evaluate(
            expression(
                8,
                IrType::Asset,
                IrExpressionKind::Asset(argui_dsl_ir::AssetId::from_raw(12)),
            ),
            &mut context,
        )
        .unwrap(),
        DslValue::Asset(argui_dsl_ir::AssetId::from_raw(12))
    );
    assert_eq!(
        evaluate(
            expression(
                9,
                IrType::String,
                IrExpressionKind::BuiltinCall {
                    function: BuiltinFunction::Translate,
                    arguments: vec![string(10, "hello")],
                },
            ),
            &mut context,
        )
        .unwrap(),
        DslValue::String("bonjour".into())
    );
    assert_eq!(
        evaluate(
            expression(
                11,
                IrType::String,
                IrExpressionKind::CallbackCall {
                    callback,
                    arguments: vec![int(12, 3), string(13, "x")],
                },
            ),
            &mut context,
        )
        .unwrap(),
        DslValue::String("called".into())
    );
    assert_eq!(context.callback_arguments.len(), 1);
    assert_eq!(
        evaluate(
            expression(
                14,
                IrType::Array(Box::new(IrType::Int)),
                IrExpressionKind::Array(vec![int(15, 1), int(16, 2)]),
            ),
            &mut context,
        )
        .unwrap(),
        DslValue::Array(vec![DslValue::Int(1), DslValue::Int(2)])
    );
    assert_eq!(
        evaluate(
            expression(
                17,
                IrType::Int,
                IrExpressionKind::Conditional {
                    condition: Box::new(bool_value(18, false)),
                    then_value: Box::new(int(19, 1)),
                    else_value: Box::new(int(20, 2)),
                },
            ),
            &mut context,
        )
        .unwrap(),
        DslValue::Int(2)
    );
}

#[test]
fn binary_and_unary_operators_cover_numeric_boolean_and_string_paths() {
    let integer_cases = [
        (BinaryOperator::Add, 7),
        (BinaryOperator::Subtract, 3),
        (BinaryOperator::Multiply, 10),
        (BinaryOperator::Divide, 2),
        (BinaryOperator::Remainder, 1),
    ];
    for (index, (operator, expected)) in integer_cases.into_iter().enumerate() {
        assert_eq!(
            evaluate(
                binary(
                    index as u64 + 1,
                    IrType::Int,
                    operator,
                    int(100, 5),
                    int(101, 2)
                ),
                &mut Context::default(),
            )
            .unwrap(),
            DslValue::Int(expected)
        );
    }
    for (operator, expected) in [
        (BinaryOperator::Add, 7.5),
        (BinaryOperator::Subtract, 2.5),
        (BinaryOperator::Multiply, 12.5),
        (BinaryOperator::Divide, 2.0),
        (BinaryOperator::Remainder, 0.0),
    ] {
        assert_eq!(
            evaluate(
                binary(20, IrType::Float, operator, float(21, 5.0), float(22, 2.5)),
                &mut Context::default(),
            )
            .unwrap(),
            DslValue::Float(expected)
        );
    }
    assert_eq!(
        evaluate(
            binary(
                23,
                IrType::String,
                BinaryOperator::Add,
                string(24, "a"),
                string(25, "b"),
            ),
            &mut Context::default(),
        )
        .unwrap(),
        DslValue::String("ab".into())
    );
    for (operator, expected) in [
        (BinaryOperator::Equal, false),
        (BinaryOperator::NotEqual, true),
        (BinaryOperator::Less, true),
        (BinaryOperator::LessEqual, true),
        (BinaryOperator::Greater, false),
        (BinaryOperator::GreaterEqual, false),
    ] {
        assert_eq!(
            evaluate(
                binary(30, IrType::Bool, operator, int(31, 1), int(32, 2)),
                &mut Context::default(),
            )
            .unwrap(),
            DslValue::Bool(expected)
        );
    }
    for (operator, left, right, expected) in [
        (BinaryOperator::And, true, false, false),
        (BinaryOperator::And, true, true, true),
        (BinaryOperator::Or, false, false, false),
        (BinaryOperator::Or, false, true, true),
    ] {
        assert_eq!(
            evaluate(
                binary(
                    40,
                    IrType::Bool,
                    operator,
                    bool_value(41, left),
                    bool_value(42, right),
                ),
                &mut Context::default(),
            )
            .unwrap(),
            DslValue::Bool(expected)
        );
    }
    for (operator, operand, expected) in [
        (
            UnaryOperator::Not,
            bool_value(50, false),
            DslValue::Bool(true),
        ),
        (UnaryOperator::Negate, int(51, 3), DslValue::Int(-3)),
        (UnaryOperator::Negate, float(52, 3.5), DslValue::Float(-3.5)),
        (UnaryOperator::Positive, int(53, 3), DslValue::Int(3)),
        (
            UnaryOperator::Positive,
            float(54, 3.5),
            DslValue::Float(3.5),
        ),
    ] {
        let value_type = expected.type_name();
        let value_type = match value_type {
            "bool" => IrType::Bool,
            "int" => IrType::Int,
            _ => IrType::Float,
        };
        assert_eq!(
            evaluate(
                expression(
                    60,
                    value_type,
                    IrExpressionKind::Unary {
                        operator,
                        operand: Box::new(operand),
                    },
                ),
                &mut Context::default(),
            )
            .unwrap(),
            expected
        );
    }
}

#[test]
fn malformed_programs_return_stable_errors_and_dependencies_are_deduplicated() {
    let property = PropertyId::from_raw(1);
    let token = TokenId::from_raw(2);
    let root_expression = binary(
        70,
        IrType::Int,
        BinaryOperator::Add,
        expression(71, IrType::Int, IrExpressionKind::PropertyRead(property)),
        binary(
            72,
            IrType::Int,
            BinaryOperator::Add,
            expression(73, IrType::Int, IrExpressionKind::PropertyRead(property)),
            expression(74, IrType::Int, IrExpressionKind::TokenRead(token)),
        ),
    );
    let program = Program::compile(&root_expression);
    assert_eq!(program.property_dependencies(), &[property]);
    assert_eq!(program.token_dependencies(), &[token]);
    assert!(!program.is_contextual());
    assert!(matches!(
        malformed(
            vec![Instruction::Property(PropertyId::from_raw(99))],
            IrType::Int
        )
        .evaluate(&mut Context::default()),
        Err(RuntimeError::MissingProperty(99))
    ));
    for instruction in [
        Instruction::Local(LocalId::from_raw(4)),
        Instruction::Token(TokenId::from_raw(5)),
    ] {
        let result = malformed(vec![instruction], IrType::Int).evaluate(&mut Context::default());
        assert!(matches!(result, Err(RuntimeError::InvalidBytecode(_))));
    }
    for instructions in [
        vec![Instruction::Unary(UnaryOperator::Not)],
        vec![Instruction::Binary(BinaryOperator::Add)],
        vec![Instruction::Array(1)],
        vec![Instruction::Builtin {
            function: BuiltinFunction::Translate,
            arguments: 1,
        }],
        vec![Instruction::Callback {
            callback: CallbackId::from_raw(1),
            arguments: 1,
        }],
    ] {
        assert!(matches!(
            malformed(instructions, IrType::Int).evaluate(&mut Context::default()),
            Err(RuntimeError::InvalidBytecode(_))
        ));
    }
    assert!(matches!(
        malformed(vec![Instruction::Jump(10)], IrType::Int).evaluate(&mut Context::default()),
        Err(RuntimeError::InvalidBytecode(message)) if message.contains("jump 10")
    ));
    assert!(matches!(
        malformed(
            vec![
                Instruction::Constant(DslValue::Bool(false)),
                Instruction::JumpIfFalse(10),
            ],
            IrType::Int,
        )
        .evaluate(&mut Context::default()),
        Err(RuntimeError::InvalidBytecode(message)) if message.contains("jump 10")
    ));
    assert!(matches!(
        evaluate(
            binary(
                80,
                IrType::Int,
                BinaryOperator::Divide,
                int(81, 1),
                int(82, 0)
            ),
            &mut Context::default(),
        ),
        Err(RuntimeError::TypeMismatch { .. })
    ));
    assert!(matches!(
        evaluate(
            expression(
                83,
                IrType::Int,
                IrExpressionKind::Unary {
                    operator: UnaryOperator::Negate,
                    operand: Box::new(string(84, "bad")),
                },
            ),
            &mut Context::default(),
        ),
        Err(RuntimeError::TypeMismatch { .. })
    ));
    assert!(matches!(
        evaluate(
            expression(
                85,
                IrType::String,
                IrExpressionKind::BuiltinCall {
                    function: BuiltinFunction::Translate,
                    arguments: vec![int(86, 1)],
                },
            ),
            &mut Context::default(),
        ),
        Err(RuntimeError::TypeMismatch { .. })
    ));
    assert!(matches!(
        malformed(
            vec![
                Instruction::Constant(DslValue::Int(1)),
                Instruction::Constant(DslValue::Int(2)),
            ],
            IrType::Int,
        )
        .evaluate(&mut Context::default()),
        Err(RuntimeError::InvalidBytecode(message)) if message.contains("2 stack values")
    ));
}
