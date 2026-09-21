//! Gradient bytecode coverage at the public program boundary.

use super::*;

/// Live bytecode preserves dynamic stop arrays for every GPU gradient kind.
#[test]
fn builtin_gradients_accept_variable_stops_and_report_errors() {
    let colors = expression(
        920,
        IrType::Array(Box::new(IrType::Color)),
        IrExpressionKind::Array(vec![
            constant(921, IrType::Color, IrValue::Color(0xff0000ff)),
            constant(922, IrType::Color, IrValue::Color(0x00ff00ff)),
            constant(923, IrType::Color, IrValue::Color(0x0000ffff)),
        ]),
    );
    let offsets = expression(
        924,
        IrType::Array(Box::new(IrType::Float)),
        IrExpressionKind::Array(vec![float(925, 0.0), float(926, 0.4), float(927, 1.0)]),
    );
    for (function, geometry) in [
        (BuiltinFunction::LinearGradient, vec![float(928, 45.0)]),
        (
            BuiltinFunction::RadialGradient,
            vec![
                float(929, 0.5),
                float(930, 0.5),
                float(931, 0.7),
                float(932, 0.7),
            ],
        ),
        (
            BuiltinFunction::ConicGradient,
            vec![float(933, 0.5), float(934, 0.5), float(935, 90.0)],
        ),
    ] {
        let mut arguments = vec![colors.clone(), offsets.clone()];
        arguments.extend(geometry);
        arguments.push(string(936, "oklab"));
        let value = evaluate(
            expression(
                937,
                IrType::Brush,
                IrExpressionKind::BuiltinCall {
                    function,
                    arguments,
                },
            ),
            &mut Context::default(),
        )
        .unwrap();
        assert!(matches!(value, DslValue::Brush(_)));
    }
    let invalid = evaluate(
        expression(
            938,
            IrType::Brush,
            IrExpressionKind::BuiltinCall {
                function: BuiltinFunction::LinearGradient,
                arguments: vec![
                    colors,
                    expression(
                        939,
                        IrType::Array(Box::new(IrType::Float)),
                        IrExpressionKind::Array(vec![float(940, 0.0)]),
                    ),
                    float(941, 0.0),
                    string(942, "oklab"),
                ],
            },
        ),
        &mut Context::default(),
    );
    assert!(
        matches!(invalid, Err(RuntimeError::InvalidBytecode(message)) if message.contains("MismatchedStops"))
    );
}

/// Malformed gradient operands report the failing boundary instead of panicking.
#[test]
fn gradient_bytecode_rejects_missing_and_mistyped_operands() {
    let red = DslValue::Color(argui_core::Color::from_srgba8(255, 0, 0, 255));
    let blue = DslValue::Color(argui_core::Color::from_srgba8(0, 0, 255, 255));
    let colors = DslValue::Array(vec![red, blue]);
    let offsets = DslValue::Array(vec![DslValue::Float(0.0), DslValue::Float(1.0)]);
    let number = DslValue::Float(45.0);
    let space = DslValue::String("oklab".into());
    let type_error = |expected: &str, actual: &str| RuntimeError::TypeMismatch {
        expected: expected.into(),
        actual: actual.into(),
    };
    let cases = [
        (
            "missing colors",
            vec![],
            RuntimeError::InvalidBytecode("gradient colors are missing".into()),
        ),
        (
            "colors not an array",
            vec![DslValue::Bool(true)],
            type_error("array<color>", "bool"),
        ),
        (
            "invalid color element",
            vec![DslValue::Array(vec![DslValue::Bool(true)])],
            type_error("color", "bool"),
        ),
        (
            "missing offsets",
            vec![colors.clone()],
            RuntimeError::InvalidBytecode("gradient offsets are missing".into()),
        ),
        (
            "offsets not an array",
            vec![colors.clone(), DslValue::Bool(true)],
            type_error("array<float>", "bool"),
        ),
        (
            "invalid offset element",
            vec![colors.clone(), DslValue::Array(vec![DslValue::Bool(true)])],
            type_error("float", "bool"),
        ),
        (
            "missing color space",
            vec![colors.clone(), offsets.clone(), number.clone()],
            RuntimeError::InvalidBytecode("gradient color space is missing".into()),
        ),
        (
            "invalid color space type",
            vec![
                colors.clone(),
                offsets.clone(),
                number.clone(),
                DslValue::Bool(true),
            ],
            type_error("string", "bool"),
        ),
        (
            "invalid geometry type",
            vec![
                colors.clone(),
                offsets.clone(),
                DslValue::Bool(true),
                space.clone(),
            ],
            type_error("float", "bool"),
        ),
    ];
    for (name, arguments, expected) in cases {
        let count = arguments.len();
        let instructions = arguments
            .into_iter()
            .map(Instruction::Constant)
            .chain([Instruction::Builtin {
                function: BuiltinFunction::LinearGradient,
                arguments: count,
            }])
            .collect();
        let result = malformed(instructions, IrType::Brush).evaluate(&mut Context::default());
        assert_eq!(result.unwrap_err(), expected, "{name}");
    }

    let valid_integer_operands = [
        Instruction::Constant(colors),
        Instruction::Constant(DslValue::Array(vec![DslValue::Int(0), DslValue::Int(1)])),
        Instruction::Constant(DslValue::Int(45)),
        Instruction::Constant(space),
        Instruction::Builtin {
            function: BuiltinFunction::LinearGradient,
            arguments: 4,
        },
    ];
    assert!(matches!(
        malformed(valid_integer_operands.to_vec(), IrType::Brush).evaluate(&mut Context::default()),
        Ok(DslValue::Brush(_))
    ));
}
