//! Gallery string, collection, and color helpers at the public bytecode boundary.

use super::*;

/// Evaluates one scalar `function` with typed `arguments` and declared `result_type`.
fn scalar(
    function: BuiltinFunction,
    arguments: Vec<IrExpression>,
    result_type: IrType,
) -> Result<DslValue, RuntimeError> {
    evaluate(
        expression(
            2000,
            result_type,
            IrExpressionKind::BuiltinCall {
                function,
                arguments,
            },
        ),
        &mut Context::default(),
    )
}

/// Search normalization, previews and range-backed galleries preserve Unicode and bounds.
#[test]
fn gallery_collection_and_unicode_preview_helpers() {
    assert_eq!(
        scalar(
            BuiltinFunction::Lower,
            vec![string(2001, "ÉCOLE Σ")],
            IrType::String
        )
        .unwrap(),
        DslValue::String("école σ".into())
    );
    for (start, count, expected) in [(1, 2, "🙂中"), (-5, 2, "é🙂"), (0, -1, ""), (99, 3, "")]
    {
        assert_eq!(
            scalar(
                BuiltinFunction::Slice,
                vec![string(2002, "é🙂中Z"), int(2003, start), int(2004, count)],
                IrType::String
            )
            .unwrap(),
            DslValue::String(expected.into())
        );
    }
    for (count, length) in [(-1, 0), (4, 4), (100_001, 100_000)] {
        let value = scalar(
            BuiltinFunction::Range,
            vec![int(2005, count)],
            IrType::Array(Box::new(IrType::Int)),
        )
        .unwrap();
        let DslValue::Array(items) = value else {
            panic!("range should produce an array")
        };
        assert_eq!(items.len(), length);
        if length > 0 {
            assert_eq!(items.last(), Some(&DslValue::Int(length as i64 - 1)));
        }
    }
}

/// The color picker can construct HSV colors and display exact RGBA channel values.
#[test]
fn gallery_color_picker_helpers_preserve_channels_and_alpha() {
    let value = scalar(
        BuiltinFunction::Hsv,
        vec![
            int(2010, 120),
            float(2011, 1.0),
            int(2012, 1),
            float(2013, 0.5),
        ],
        IrType::Color,
    )
    .unwrap();
    let DslValue::Color(color) = value else {
        panic!("hsv should produce a color")
    };
    assert_eq!(color.to_srgba8(), [0, 255, 0, 128]);
    let color_arg = || constant(2014, IrType::Color, IrValue::Color(0x12345678));
    for (function, expected) in [
        (BuiltinFunction::ColorRed, 0x12),
        (BuiltinFunction::ColorGreen, 0x34),
        (BuiltinFunction::ColorBlue, 0x56),
    ] {
        assert_eq!(
            scalar(function, vec![color_arg()], IrType::Int).unwrap(),
            DslValue::Int(expected)
        );
    }
    assert_eq!(
        scalar(BuiltinFunction::ColorHex, vec![color_arg()], IrType::String).unwrap(),
        DslValue::String("#12345678".into())
    );
}

/// Malformed live color bytecode is rejected before converting nonnumeric operands.
#[test]
fn color_picker_rejects_mistyped_and_missing_arguments() {
    for (function, arguments) in [
        (
            BuiltinFunction::Hsv,
            vec![
                string(2020, "red"),
                int(2021, 1),
                int(2022, 1),
                int(2023, 1),
            ],
        ),
        (BuiltinFunction::Hsv, vec![int(2024, 0)]),
        (BuiltinFunction::ColorRed, vec![string(2025, "#ff0000")]),
        (
            BuiltinFunction::Slice,
            vec![string(2026, "preview"), int(2027, 0)],
        ),
    ] {
        assert!(
            matches!(scalar(function, arguments, IrType::Color), Err(RuntimeError::InvalidBytecode(message)) if message.contains("invalid arguments"))
        );
    }
}
