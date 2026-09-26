//! Guards the browser host against a generated contract with different features.

#[test]
fn embedded_contract_matches_web_host_schema() {
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../crates/argui-cli/assets/sdk/host/contract.generated.json"
    ))
    .expect("generated contract must parse");
    let host = argui_runtime::NativeHost::with_builtins().expect("built-in schema must register");
    assert_eq!(
        contract["abiHash"].as_str(),
        Some(host.abi_hash().to_string().as_str())
    );
}
