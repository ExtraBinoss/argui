//! Application-owned native primitive exercised through AOT and live compilation.

use argui_testing::TestApp;

/// The generated AOT renderer builds the registered primitive, its slot, and event.
#[test]
fn aot_extension_renders_and_activates() {
    let registry = argui_example_dsl_live_demo::native_extension::registry().unwrap();
    argui_example_dsl_live_demo::extension::install_native_registry(registry).unwrap();
    let mut app = TestApp::new(argui_example_dsl_live_demo::extension::ExtensionProof::new());
    app.assert_text("Ready");
    app.assert_text("Slot child");
    app.get_by_text("Ready").click().unwrap();
    app.assert_text("Activated");
}

/// The live renderer accepts the same extension contract and activates its event.
#[cfg(feature = "argui-live")]
#[test]
fn live_extension_renders_and_activates() {
    let registry = argui_example_dsl_live_demo::native_extension::registry().unwrap();
    let (package, root) = live_package(registry.clone());
    let mut runtime = argui_dsl_runtime::LiveRuntime::new_with_registry(package, registry).unwrap();
    runtime.mount(root, []).unwrap();
    let mut app = TestApp::new(runtime);
    app.assert_text("Ready");
    app.assert_text("Slot child");
    app.get_by_text("Ready").click().unwrap();
    app.assert_text("Activated");
}

/// An extension-compiled package rejects a runtime with a different native ABI.
#[cfg(feature = "argui-live")]
#[test]
fn live_extension_rejects_builtin_only_registry() {
    let (package, _) =
        live_package(argui_example_dsl_live_demo::native_extension::registry().unwrap());
    assert!(argui_dsl_runtime::LiveRuntime::new(package).is_err());
}

/// Compiles the extension source and its asset with a caller-owned registry.
#[cfg(feature = "argui-live")]
fn live_package(
    registry: argui::schema::SchemaRegistry,
) -> (argui_dsl_runtime::LivePackage, argui_dsl_ir::ComponentId) {
    use argui_dsl_compiler::{Compiler, SourceModule};
    let compiled = Compiler::compile_with_registry(
        [SourceModule::new(
            "tests/native_extension/extension.argui",
            include_str!("native_extension/extension.argui"),
        )],
        "tests/native_extension/extension.argui",
        registry,
        |_| Ok(include_bytes!("native_extension/assets/spark.svg").to_vec()),
    )
    .unwrap();
    let root_symbol = compiled
        .semantic
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == "ExtensionProof")
        .unwrap()
        .id
        .raw();
    let root = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == root_symbol)
        .unwrap()
        .id;
    let assets = compiled
        .ir
        .assets
        .iter()
        .map(|asset| {
            (
                asset.id,
                argui_dsl_runtime::AssetPayload::new(
                    1,
                    include_bytes!("native_extension/assets/spark.svg").to_vec(),
                ),
            )
        })
        .collect();
    (
        argui_dsl_runtime::LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, assets)
            .unwrap(),
        root,
    )
}

/// The language server checks the external native name, property, event, and slot.
#[test]
fn lsp_recognizes_application_extension() {
    use serde_json::json;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("tests/native_extension/extension.argui");
    let uri = format!("file://{}", path.display());
    let source = include_str!("native_extension/extension.argui");
    let mut server = argui_dsl_lsp::LanguageServer::with_registry(
        root.clone(),
        argui_example_dsl_live_demo::native_extension::registry().unwrap(),
    )
    .unwrap();
    server.handle_message(&json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"rootUri": format!("file://{}", root.display())},
    }));
    let notifications = server.handle_message(&json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": {"textDocument": {
            "uri": uri, "languageId": "argui", "version": 1, "text": source,
        }},
    }));
    let diagnostics = notifications[0]["params"]["diagnostics"]
        .as_array()
        .unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}
