use argui_dsl_ir::IrType;
use argui_schema::ValueType;

#[test]
fn native_schema_value_types_map_to_backend_stable_ir_types() {
    let cases = [
        (ValueType::Bool, IrType::Bool),
        (ValueType::Int, IrType::Int),
        (ValueType::Float, IrType::Float),
        (ValueType::String, IrType::String),
        (ValueType::Name, IrType::String),
        (ValueType::Color, IrType::Color),
        (ValueType::Brush, IrType::Brush),
        (ValueType::Dimension, IrType::Dimension),
        (ValueType::Insets, IrType::Insets),
        (ValueType::Radii, IrType::Radii),
        (ValueType::Border, IrType::Border),
        (ValueType::Shadow, IrType::Shadow),
        (ValueType::Transform, IrType::Transform),
        (ValueType::Asset, IrType::Asset),
    ];

    for (schema_type, expected) in cases {
        assert_eq!(IrType::from_schema(schema_type), expected);
    }
}
