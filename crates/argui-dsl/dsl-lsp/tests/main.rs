use std::process::{Command, Stdio};

use argui_dsl_lsp::write_message;
use serde_json::json;

/// Drives the packaged binary through framed stdio until an exit notification.
#[test]
fn stdio_server_processes_requests_until_exit() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_argui-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    write_message(
        &mut input,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
    )
    .unwrap();
    write_message(
        &mut input,
        &json!({"jsonrpc": "2.0", "method": "exit", "params": {}}),
    )
    .unwrap();
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("capabilities"));
}
