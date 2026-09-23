//! Checks the sendable protocol used by the Solid and React native host.

use argui_runtime::{HostOperation, WireOperation};
use argui_schema::{AssetHandle, SchemaValue};

#[test]
fn wire_commit_decodes_all_gallery_operation_shapes() {
    let operations: Vec<WireOperation> = serde_json::from_value(serde_json::json!([
        {"kind":"create","id":{"slot":1,"generation":1},"nativeType":1},
        {"kind":"setProperty","id":{"slot":1,"generation":1},"property":3,"value":{"type":"Color","value":"#223344"}},
        {"kind":"setListener","id":{"slot":1,"generation":1},"event":1,"callback":7},
        {"kind":"insert","parent":{"slot":1,"generation":1},"child":{"slot":2,"generation":1},"before":null},
        {"kind":"setRoot","id":{"slot":1,"generation":1}},
        {"kind":"remove","id":{"slot":2,"generation":1}}
    ])).expect("valid transport batch");
    assert_eq!(operations.len(), 6);
    let native = operations
        .into_iter()
        .map(WireOperation::into_native)
        .collect::<Result<Vec<_>, _>>()
        .expect("typed operations");
    assert!(matches!(native[1], HostOperation::SetProperty { .. }));
    assert!(matches!(native[5], HostOperation::Remove { .. }));
}

#[test]
fn invalid_wire_value_rejects_entire_decode() {
    let operation: WireOperation = serde_json::from_value(serde_json::json!({
        "kind":"setProperty","id":{"slot":1,"generation":1},"property":3,
        "value":{"type":"Color","value":"red"}
    }))
    .expect("transport shape");
    assert!(operation.into_native().is_err());
}

#[test]
fn dimension_and_semantic_values_decode_without_cross_thread_schema_objects() {
    for value in [
        serde_json::json!({"type":"Dimension","value":"75%"}),
        serde_json::json!({"type":"Dimension","value":"auto"}),
        serde_json::json!({"type":"Dimension","value":"fill"}),
        serde_json::json!({"type":"Name","value":"page"}),
        serde_json::json!({"type":"Bool","value":true}),
    ] {
        let wire = serde_json::from_value(value).expect("wire value");
        assert!(argui_runtime::WireValue::into_native(wire).is_ok());
    }
}

#[test]
fn imported_raster_and_vector_handles_decode_and_reject_unknown_kinds() {
    for (kind, expected) in [
        ("image", AssetHandle::Image(argui_paint::ImageId(17))),
        ("svg", AssetHandle::Vector(argui_paint::VectorId(17))),
    ] {
        let wire = serde_json::from_value(serde_json::json!({
            "type": "Asset", "value": {"kind": kind, "id": 17}
        }))
        .expect("asset wire shape");
        assert_eq!(
            argui_runtime::WireValue::into_native(wire).expect("asset handle"),
            SchemaValue::Asset(expected)
        );
    }
    for value in [
        serde_json::json!({"kind":"video", "id":17}),
        serde_json::json!({"kind":"svg", "id":0}),
        serde_json::json!({"kind":"image", "id":9007199254740992u64}),
    ] {
        let wire = serde_json::from_value(serde_json::json!({"type":"Asset", "value":value}))
            .expect("asset wire shape");
        assert!(argui_runtime::WireValue::into_native(wire).is_err());
    }
}

#[test]
fn all_typed_wire_families_accept_valid_payloads() {
    for (kind, value) in [
        ("Bool", serde_json::json!(false)),
        ("Int", serde_json::json!(-7)),
        ("Float", serde_json::json!(1.25)),
        ("String", serde_json::json!("native text")),
        ("Name", serde_json::json!("gallery")),
        ("Color", serde_json::json!("#123456")),
        ("Brush", serde_json::json!("#123456")),
        ("Dimension", serde_json::json!(24)),
        ("Dimension", serde_json::json!("24px")),
        ("Insets", serde_json::json!(8)),
        (
            "Insets",
            serde_json::json!({"top":1,"right":2,"bottom":3,"left":4}),
        ),
        ("Radii", serde_json::json!(8)),
        (
            "Radii",
            serde_json::json!({"topLeft":1,"topRight":2,"bottomRight":3,"bottomLeft":4}),
        ),
        (
            "Transform",
            serde_json::json!({"x":1,"y":2,"scaleX":2,"scaleY":3,"rotation":0.5}),
        ),
        ("Transform", serde_json::json!({})),
    ] {
        let wire = serde_json::from_value(serde_json::json!({"type":kind,"value":value})).unwrap();
        assert!(
            argui_runtime::WireValue::into_native(wire).is_ok(),
            "{kind}: {value}"
        );
    }
}

#[test]
fn malformed_wire_families_fail_without_partial_decoding() {
    for (kind, value) in [
        ("Int", serde_json::json!("7")),
        ("Float", serde_json::json!(1e300)),
        ("String", serde_json::json!(7)),
        ("Name", serde_json::json!(null)),
        ("Color", serde_json::json!("red")),
        ("Brush", serde_json::json!("red")),
        ("Dimension", serde_json::json!("bad")),
        ("Insets", serde_json::json!({"top":1,"right":2,"bottom":3})),
        (
            "Radii",
            serde_json::json!({"topLeft":1,"topRight":2,"bottomRight":3}),
        ),
        ("Transform", serde_json::json!([])),
        ("Unsupported", serde_json::json!(1)),
    ] {
        let wire = serde_json::from_value(serde_json::json!({"type":kind,"value":value})).unwrap();
        assert!(
            argui_runtime::WireValue::into_native(wire).is_err(),
            "{kind}: {value}"
        );
    }
}
