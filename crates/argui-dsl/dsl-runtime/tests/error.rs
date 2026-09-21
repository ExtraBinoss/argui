//! Public and bytecode error contracts.

mod public_errors {
    use std::collections::HashMap;

    use argui_dsl_ir::{ComponentId, PropertyId, ThemeModeId};
    #[cfg(not(target_arch = "wasm32"))]
    use argui_dsl_runtime::ClientEvent;
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime, RuntimeError};

    fn empty_runtime() -> LiveRuntime {
        let package = LivePackage::prepare(
            0,
            17,
            argui_dsl_ir::IrProject {
                modules: Vec::new(),
                structs: Vec::new(),
                enums: Vec::new(),
                components: Vec::new(),
                themes: Vec::new(),
                styles: Vec::new(),
                effects: Vec::new(),
                assets: Vec::new(),
            },
            HashMap::new(),
        )
        .unwrap();
        LiveRuntime::new(package).unwrap()
    }

    #[test]
    fn public_runtime_reports_unmounted_and_missing_state_without_panicking() {
        let mut runtime = empty_runtime();
        assert_eq!(runtime.root(), None);
        assert_eq!(runtime.generation(), 1);
        assert_eq!(runtime.public_api_hash(), 17);
        assert!(runtime.ir().components.is_empty());
        assert!(runtime.assets().records().next().is_none());
        assert_eq!(runtime.last_error(), None);
        assert!(matches!(
            runtime.render(),
            Err(RuntimeError::InvalidBytecode(message)) if message.contains("no live root")
        ));
        assert!(matches!(
            runtime.mount(ComponentId::from_raw(99), []),
            Err(RuntimeError::MissingComponent(99))
        ));
        assert!(matches!(
            runtime.set_property(
                argui_dsl_runtime::InstanceId::from_raw(1),
                PropertyId::from_raw(2),
                DslValue::Int(1),
            ),
            Err(RuntimeError::MissingComponent(1))
        ));
        assert!(matches!(
            runtime.bind_callback(
                argui_dsl_runtime::InstanceId::from_raw(1),
                argui_dsl_ir::CallbackId::from_raw(2),
                |_| DslValue::Null,
            ),
            Err(RuntimeError::MissingComponent(1))
        ));
        assert!(matches!(
            runtime.invoke_callback(
                argui_dsl_runtime::InstanceId::from_raw(1),
                argui_dsl_ir::CallbackId::from_raw(2),
                Vec::new(),
            ),
            Err(RuntimeError::MissingComponent(1))
        ));
        assert!(matches!(
            runtime.set_theme_mode(ThemeModeId::from_raw(1)),
            Err(RuntimeError::InvalidBytecode(message)) if message.contains("unknown theme mode")
        ));
        runtime.set_translator(|_| None);
        runtime.clear_translator();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_package_client_events_are_preserved_by_the_transaction_boundary() {
        let mut runtime = empty_runtime();
        for event in [
            ClientEvent::Diagnostics {
                generation: 1,
                diagnostics: Vec::new(),
            },
            ClientEvent::RestartRequired {
                generation: 1,
                previous_api_hash: 1,
                next_api_hash: 2,
            },
            ClientEvent::Disconnected("closed".into()),
            ClientEvent::Committed(argui_dsl_runtime::ReloadOutcome {
                previous_generation: 1,
                generation: 2,
                migrated_instances: 0,
            }),
        ] {
            let copy = event.clone();
            assert_eq!(
                argui_dsl_runtime::LiveClient::apply(&mut runtime, event).unwrap(),
                copy
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn failed_native_connection_is_reported_as_a_bounded_runtime_error() {
        let error = argui_dsl_runtime::LiveClient::connect(("127.0.0.1", 0));
        assert!(matches!(error, Err(RuntimeError::IncompatiblePackage(_))));
        let error = LiveRuntime::connect(("127.0.0.1", 0), std::time::Duration::from_millis(1));
        assert!(matches!(error, Err(RuntimeError::IncompatiblePackage(_))));
    }
}

mod bytecode_edges {
    use argui_dsl_ir::{
        BinaryOperator, BuiltinFunction, CallbackId, ExpressionId, IrExpression, IrExpressionKind,
        IrType, IrValue, SourceInfo, UnaryOperator,
    };
    use argui_dsl_runtime::{DslValue, EvaluationContext, Instruction, Program, RuntimeError};
    use argui_dsl_syntax::{FileId, Span, TextRange, TextSize};

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

    fn bool_value(id: u64, value: bool) -> IrExpression {
        constant(id, IrType::Bool, IrValue::Bool(value))
    }

    fn string(id: u64, value: &str) -> IrExpression {
        constant(id, IrType::String, IrValue::String(value.into()))
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
    struct Context;

    impl EvaluationContext for Context {
        fn property(&self, _id: argui_dsl_ir::PropertyId) -> Option<DslValue> {
            None
        }

        fn local(&self, _id: argui_dsl_ir::LocalId) -> Option<DslValue> {
            None
        }

        fn token(&self, _id: argui_dsl_ir::TokenId) -> Option<DslValue> {
            None
        }

        fn callback(&mut self, _id: CallbackId, _arguments: Vec<DslValue>) -> Option<DslValue> {
            None
        }

        fn translate(&self, _id: &str) -> Option<String> {
            None
        }
    }

    fn malformed(instructions: Vec<Instruction>, result_type: IrType) -> Program {
        let mut program = Program::compile(&constant(1, result_type.clone(), IrValue::Null));
        program.result_type = result_type;
        program.instructions = instructions;
        program
    }

    #[test]
    fn zero_division_remainder_and_incompatible_binary_values_are_rejected() {
        for operator in [BinaryOperator::Divide, BinaryOperator::Remainder] {
            let result =
                Program::compile(&binary(10, IrType::Int, operator, int(11, 4), int(12, 0)))
                    .evaluate(&mut Context);
            assert!(matches!(result, Err(RuntimeError::TypeMismatch { .. })));
        }
        let result = Program::compile(&binary(
            20,
            IrType::Int,
            BinaryOperator::Add,
            bool_value(21, true),
            bool_value(22, false),
        ))
        .evaluate(&mut Context);
        assert!(matches!(result, Err(RuntimeError::TypeMismatch { .. })));
    }

    #[test]
    fn malformed_stack_paths_report_each_bounded_failure() {
        for instructions in [
            vec![Instruction::Unary(UnaryOperator::Not)],
            vec![Instruction::Binary(BinaryOperator::Add)],
            vec![Instruction::Array(1)],
            vec![Instruction::Field(argui_dsl_ir::FieldId::from_raw(1))],
        ] {
            assert!(matches!(
                malformed(instructions, IrType::Int).evaluate(&mut Context),
                Err(RuntimeError::InvalidBytecode(_))
            ));
        }
        assert!(matches!(
            malformed(
                vec![
                    Instruction::Constant(DslValue::Int(1)),
                    Instruction::Field(argui_dsl_ir::FieldId::from_raw(1)),
                ],
                IrType::Int,
            )
            .evaluate(&mut Context),
            Err(RuntimeError::TypeMismatch { .. })
        ));
        assert!(matches!(
            malformed(
                vec![Instruction::Jump(1)],
                IrType::Int,
            )
            .evaluate(&mut Context),
            Err(RuntimeError::InvalidBytecode(message)) if message.contains("0 stack values")
        ));
        assert!(matches!(
            malformed(
                vec![
                    Instruction::Constant(DslValue::Bool(true)),
                    Instruction::JumpIfFalse(2),
                    Instruction::Constant(DslValue::Int(1)),
                ],
                IrType::Int,
            )
            .evaluate(&mut Context),
            Ok(DslValue::Int(1))
        ));
    }

    #[test]
    fn zero_argument_builtins_and_callbacks_have_deterministic_values() {
        let translated = Program::compile(&expression(
            30,
            IrType::String,
            IrExpressionKind::BuiltinCall {
                function: BuiltinFunction::Translate,
                arguments: Vec::new(),
            },
        ))
        .evaluate(&mut Context)
        .unwrap();
        assert_eq!(translated, DslValue::String(String::new()));

        let callback = Program::compile(&expression(
            31,
            IrType::Unknown,
            IrExpressionKind::CallbackCall {
                callback: CallbackId::from_raw(5),
                arguments: Vec::new(),
            },
        ))
        .evaluate(&mut Context)
        .unwrap();
        assert_eq!(callback, DslValue::Null);

        let array = malformed(
            vec![Instruction::Array(0)],
            IrType::Array(Box::new(IrType::Int)),
        )
        .evaluate(&mut Context)
        .unwrap();
        assert_eq!(array, DslValue::Array(Vec::new()));
    }

    #[test]
    fn unary_type_errors_and_unused_context_maps_are_stable() {
        let result = Program::compile(&expression(
            40,
            IrType::Bool,
            IrExpressionKind::Unary {
                operator: UnaryOperator::Positive,
                operand: Box::new(string(41, "not numeric")),
            },
        ))
        .evaluate(&mut Context);
        assert!(matches!(result, Err(RuntimeError::TypeMismatch { .. })));
    }
}
