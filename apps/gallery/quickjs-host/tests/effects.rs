use argui_gallery_quickjs::registry_from_json;
use serde_json::json;

#[test]
fn imported_gallery_wgsl_validates_and_invalid_source_is_atomic() {
    let source = include_str!("../../src/prism.wgsl");
    let valid = json!([{
        "id": "gallery.examples.prism",
        "source": source,
        "parameters": [
            {"name":"strength","type":"f32"},
            {"name":"frequency","type":"f32"}
        ]
    }]);
    let registry = registry_from_json(&valid.to_string()).expect("imported WGSL registry");
    assert!(registry.get(&argui_effects::EDGE_SHADOW_ID).is_some());
    assert!(registry.get(&argui_effects::EDGE_FADE_ID).is_some());
    assert_eq!(registry.definitions().len(), 3);
    let invalid = json!([valid[0], {
        "id": "gallery.examples.invalid",
        "source": "fn argui_effect( {",
        "parameters": []
    }]);
    assert!(registry_from_json(&invalid.to_string()).is_err());
    assert_eq!(
        registry.definitions().len(),
        3,
        "existing engine and imported registry remains valid"
    );
}
