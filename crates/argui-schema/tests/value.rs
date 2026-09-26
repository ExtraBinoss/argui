use argui_core::{Color, Transform2D};
use argui_paint::{Border, CornerRadii, Fill, ImageId, Shadow};
use argui_schema::{AssetHandle, SchemaValue, ValueType};
use argui_ui::{LayoutInsets, LengthPercentageAuto, PositionInsets, length};

#[test]
fn each_runtime_value_reports_its_canonical_schema_type() {
    let cases = [
        (SchemaValue::Bool(true), ValueType::Bool),
        (SchemaValue::Int(12), ValueType::Int),
        (SchemaValue::Float(2.5), ValueType::Float),
        (SchemaValue::String("text".into()), ValueType::String),
        (SchemaValue::Name("field".into()), ValueType::Name),
        (SchemaValue::Color(Color::BLACK), ValueType::Color),
        (
            SchemaValue::Brush(Fill::Solid(Color::BLACK)),
            ValueType::Brush,
        ),
        (SchemaValue::Dimension(length(24.0)), ValueType::Dimension),
        (
            SchemaValue::Constraint(LengthPercentageAuto::percent(0.5)),
            ValueType::Constraint,
        ),
        (
            SchemaValue::Insets(LayoutInsets::default()),
            ValueType::Insets,
        ),
        (
            SchemaValue::PositionInsets(PositionInsets::default()),
            ValueType::PositionInsets,
        ),
        (SchemaValue::Radii(CornerRadii::all(3.0)), ValueType::Radii),
        (
            SchemaValue::Border(Border::all(1.0, Color::BLACK)),
            ValueType::Border,
        ),
        (
            SchemaValue::Shadow(Shadow::drop([1.0, 2.0], 3.0, Color::BLACK)),
            ValueType::Shadow,
        ),
        (
            SchemaValue::Transform(Transform2D::IDENTITY),
            ValueType::Transform,
        ),
        (
            SchemaValue::Asset(AssetHandle::Image(ImageId::fresh())),
            ValueType::Asset,
        ),
    ];
    for (value, expected) in cases {
        assert_eq!(value.value_type(), expected);
    }
}
