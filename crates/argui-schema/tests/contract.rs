#![cfg(feature = "serde")]

use argui_schema::{
    NativeElementInput, NativeSchema, NativeTypeId, PropertyId, PropertySchema, SchemaRegistry,
    ValueType,
};
use argui_ui::Element;

#[test]
fn contract_exports_custom_primitives_with_closed_values() {
    let mut registry = SchemaRegistry::new();
    registry
        .register(
            NativeSchema::new(NativeTypeId::from_raw(900), "Chart", "Custom chart").property(
                PropertySchema::new(
                    PropertyId::from_raw(1),
                    "renderMode",
                    ValueType::String,
                    "Chart renderer.",
                )
                .allowed_values(&["bars", "lines"]),
            ),
            |_input: &NativeElementInput| Ok(Element::container([])),
        )
        .unwrap();

    let json = serde_json::to_value(registry.contract()).unwrap();
    assert_eq!(json["natives"][0]["name"], "Chart");
    assert_eq!(
        json["natives"][0]["properties"][0]["allowedValues"],
        serde_json::json!(["bars", "lines"])
    );
    assert!(json["abiHash"].is_string());
}
