use argui_dsl_semantic::{SymbolId, Type};

#[test]
fn builtin_types_and_numeric_units_have_explicit_boundaries() {
    let names = [
        ("void", Type::Void),
        ("bool", Type::Bool),
        ("int", Type::Int),
        ("float", Type::Float),
        ("string", Type::String),
        ("color", Type::Color),
        ("brush", Type::Brush),
        ("length", Type::Length),
        ("dimension", Type::Dimension),
        ("percentage", Type::Percentage),
        ("duration", Type::Duration),
        ("angle", Type::Angle),
        ("radii", Type::Radii),
        ("insets", Type::Insets),
        ("border", Type::Border),
        ("shadow", Type::Shadow),
        ("font-family", Type::FontFamily),
        ("font-weight", Type::FontWeight),
        ("font-size", Type::FontSize),
        ("line-height", Type::LineHeight),
        ("transform", Type::Transform),
        ("asset", Type::Asset),
    ];
    for (name, expected) in names {
        assert_eq!(Type::builtin(name), Some(expected));
    }
    assert_eq!(Type::builtin("unknown"), None);
    for value in [
        Type::Int,
        Type::Float,
        Type::Length,
        Type::Percentage,
        Type::Duration,
        Type::Angle,
        Type::FontSize,
        Type::LineHeight,
    ] {
        assert!(value.is_numeric(), "{value:?} should be numeric");
    }
    for value in [
        Type::Unknown,
        Type::Void,
        Type::Bool,
        Type::String,
        Type::Color,
        Type::Dimension,
        Type::Asset,
        Type::Radii,
        Type::Transform,
    ] {
        assert!(!value.is_numeric(), "{value:?} should not be numeric");
    }
}

#[test]
fn accepts_handles_unknown_widening_units_and_nested_containers() {
    assert!(Type::Unknown.accepts(&Type::String));
    assert!(Type::String.accepts(&Type::Unknown));
    assert!(Type::Float.accepts(&Type::Int));
    assert!(!Type::Int.accepts(&Type::Float));
    assert!(Type::Dimension.accepts(&Type::Length));
    assert!(Type::Dimension.accepts(&Type::Percentage));
    assert!(!Type::Length.accepts(&Type::Percentage));
    assert!(Type::Optional(Box::new(Type::Float)).accepts(&Type::Int));
    assert!(Type::Optional(Box::new(Type::Float)).accepts(&Type::Optional(Box::new(Type::Int))));
    assert!(!Type::Optional(Box::new(Type::String)).accepts(&Type::Optional(Box::new(Type::Int))));
    assert!(Type::Array(Box::new(Type::Float)).accepts(&Type::Array(Box::new(Type::Int))));
    assert!(!Type::Array(Box::new(Type::Int)).accepts(&Type::Model(Box::new(Type::Int))));
    assert!(Type::Model(Box::new(Type::Float)).accepts(&Type::Model(Box::new(Type::Int))));
    assert!(!Type::Model(Box::new(Type::Float)).accepts(&Type::Array(Box::new(Type::Int))));
}

#[test]
fn displays_named_nested_and_callback_types() {
    let symbol = SymbolId::derive("ui/model.argui", "struct", "Project");
    assert_eq!(Type::Struct(symbol).to_string(), format!("struct#{symbol}"));
    assert_eq!(Type::Enum(symbol).to_string(), format!("enum#{symbol}"));
    assert_eq!(
        Type::Optional(Box::new(Type::String)).to_string(),
        "optional<String>"
    );
    assert_eq!(Type::Array(Box::new(Type::Int)).to_string(), "array<Int>");
    assert_eq!(
        Type::Model(Box::new(Type::String)).to_string(),
        "model<String>"
    );
    assert_eq!(
        Type::Callback {
            parameters: vec![Type::String, Type::Int],
            result: Box::new(Type::Bool),
        }
        .to_string(),
        "callback(String, Int) -> Bool"
    );
}
