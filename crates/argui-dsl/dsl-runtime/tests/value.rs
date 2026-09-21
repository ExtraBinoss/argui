use std::collections::BTreeMap;

use argui_dsl_ir::IrType;
use argui_dsl_runtime::DslValue;

#[test]
fn value_compatibility_and_type_names_cover_all_dsl_values() {
    let values = [
        DslValue::Null,
        DslValue::Bool(true),
        DslValue::Int(1),
        DslValue::Float(1.0),
        DslValue::String("x".into()),
        DslValue::Color(argui_core::Color::WHITE),
        DslValue::Struct(BTreeMap::new()),
        DslValue::Enum {
            symbol: 1,
            variant: 2,
        },
        DslValue::Array(Vec::new()),
        DslValue::Asset(argui_dsl_ir::AssetId::from_raw(1)),
    ];
    assert_eq!(
        values.iter().map(DslValue::type_name).collect::<Vec<_>>(),
        vec![
            "null", "bool", "int", "float", "string", "color", "struct", "enum", "array", "asset"
        ]
    );
    assert!(DslValue::Null.compatible_with(&IrType::Optional(Box::new(IrType::String))));
    assert!(DslValue::Int(1).compatible_with(&IrType::Float));
    assert!(DslValue::Float(1.0).compatible_with(&IrType::Duration));
    assert!(DslValue::String("x".into()).compatible_with(&IrType::FontWeight));
    assert!(DslValue::Color(argui_core::Color::WHITE).compatible_with(&IrType::Color));
    assert!(
        DslValue::Struct(BTreeMap::new()).compatible_with(&IrType::Struct {
            symbol: argui_dsl_semantic::SymbolId::derive("m", "struct", "S"),
            fields: Vec::new(),
        })
    );
    assert!(
        DslValue::Enum {
            symbol: 1,
            variant: 2
        }
        .compatible_with(&IrType::Enum(argui_dsl_semantic::SymbolId::derive(
            "m", "enum", "E"
        )))
    );
    assert!(DslValue::Array(Vec::new()).compatible_with(&IrType::Model(Box::new(IrType::Int))));
    assert!(DslValue::Asset(argui_dsl_ir::AssetId::from_raw(1)).compatible_with(&IrType::Asset));
    assert!(!DslValue::Bool(true).compatible_with(&IrType::String));
}
