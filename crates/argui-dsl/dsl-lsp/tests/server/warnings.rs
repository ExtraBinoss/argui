//! The LSP retains warning severity and source locations for accepted programs.

use super::*;

/// An unreachable handler statement produces a warning notification.
#[test]
fn lsp_publishes_non_fatal_source_warnings() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("main.argui");
    let source = "import { TouchArea } from \"@argui/native\" export component Main { private property count: int = 0 TouchArea { on click { return; count = 1 } } }";
    fs::write(&path, source).unwrap();
    let mut server = LanguageServer::new(root.path().into()).unwrap();
    let notifications = server.handle_message(&json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": file_uri(&path), "languageId": "argui", "version": 1, "text": source}},
    }));
    let diagnostics = notifications[0]["params"]["diagnostics"]
        .as_array()
        .unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|issue| issue["code"] == "UnreachableStatement" && issue["severity"] == 2)
    );
}
