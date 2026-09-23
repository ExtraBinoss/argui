//! Scalar Rust source fragments emitted by DSL expression codegen.

use argui_dsl_ir::IrValue;

/// Emits a typed constant value.
pub(super) fn constant(value: &IrValue) -> String {
    match value {
        IrValue::Null => "None".into(),
        IrValue::Bool(value) => value.to_string(),
        IrValue::Int(value) => format!("{value}_i64"),
        IrValue::Float(value) => format!("{value:?}_f32"),
        IrValue::String(value) => format!("String::from(\"{}\")", value.escape_default()),
        IrValue::Color(value) => {
            let [red, green, blue, alpha] = value.to_be_bytes();
            format!("::argui::core::Color::from_srgba8({red}, {green}, {blue}, {alpha})")
        }
    }
}

/// Returns the direct Rust operator for one IR binary operator.
pub(super) fn binary_operator(operator: argui_dsl_ir::BinaryOperator) -> &'static str {
    match operator {
        argui_dsl_ir::BinaryOperator::Add => "+",
        argui_dsl_ir::BinaryOperator::Subtract => "-",
        argui_dsl_ir::BinaryOperator::Multiply => "*",
        argui_dsl_ir::BinaryOperator::Divide => "/",
        argui_dsl_ir::BinaryOperator::Remainder => "%",
        argui_dsl_ir::BinaryOperator::Equal => "==",
        argui_dsl_ir::BinaryOperator::NotEqual => "!=",
        argui_dsl_ir::BinaryOperator::Less => "<",
        argui_dsl_ir::BinaryOperator::LessEqual => "<=",
        argui_dsl_ir::BinaryOperator::Greater => ">",
        argui_dsl_ir::BinaryOperator::GreaterEqual => ">=",
        argui_dsl_ir::BinaryOperator::And => "&&",
        argui_dsl_ir::BinaryOperator::Or => "||",
    }
}
