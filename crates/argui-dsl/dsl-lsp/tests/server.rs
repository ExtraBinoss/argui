use std::{fs, path::Path};

use argui_dsl_lsp::LanguageServer;
use serde_json::{Value, json};

#[path = "server/warnings.rs"]
mod warnings;

fn file_uri(path: &Path) -> String {
    let text = path
        .to_string_lossy()
        .replace('\\', "/")
        .replace(' ', "%20");
    if text.starts_with('/') {
        format!("file://{text}")
    } else {
        format!("file:///{text}")
    }
}

fn request(server: &mut LanguageServer, id: u32, method: &str, parameters: Value) -> Value {
    server
        .handle_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": parameters,
        }))
        .into_iter()
        .next()
        .unwrap()
}

#[test]
fn server_exposes_the_shared_semantic_authoring_surface() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("main.argui");
    let source = r#"import { Button } from "@argui/ui"
import { Container } from "@argui/native"
export theme AppTheme { --accent: color = #369c }
export component Main {
    Button {
        text: "Save"
    }
    Container { backgroun: var(--accent) }
}
"#;
    fs::write(&path, source).unwrap();
    let uri = file_uri(&path);
    let mut server = LanguageServer::new(root.path().into()).unwrap();

    let initialized = request(
        &mut server,
        1,
        "initialize",
        json!({"rootUri": file_uri(root.path())}),
    );
    assert_eq!(initialized["result"]["capabilities"]["hoverProvider"], true);

    let notifications = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": uri, "languageId": "argui", "version": 1, "text": source}},
    }));
    assert!(
        notifications[0]["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["code"] == "UnknownProperty")
    );

    let completion = request(
        &mut server,
        2,
        "textDocument/completion",
        json!({"textDocument": {"uri": uri}, "position": {"line": 5, "character": 8}}),
    );
    assert!(
        completion["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "enabled")
    );

    let actions = request(
        &mut server,
        3,
        "textDocument/codeAction",
        json!({"textDocument": {"uri": uri}, "range": {"start": {"line": 7, "character": 16}, "end": {"line": 7, "character": 25}}, "context": {"diagnostics": []}}),
    );
    assert!(
        actions["result"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| action["title"].as_str().unwrap().contains("background"))
    );

    let colors = request(
        &mut server,
        4,
        "textDocument/documentColor",
        json!({"textDocument": {"uri": uri}}),
    );
    assert_eq!(colors["result"].as_array().unwrap().len(), 1);
    let alpha = colors["result"][0]["color"]["alpha"].as_f64().unwrap();
    assert!((alpha - 0.8).abs() < 0.000_001);

    let tokens = request(
        &mut server,
        5,
        "textDocument/semanticTokens/full",
        json!({"textDocument": {"uri": uri}}),
    );
    assert!(!tokens["result"]["data"].as_array().unwrap().is_empty());

    let symbols = request(
        &mut server,
        6,
        "textDocument/documentSymbol",
        json!({"textDocument": {"uri": uri}}),
    );
    assert!(
        symbols["result"]
            .as_array()
            .unwrap()
            .iter()
            .any(|symbol| symbol["name"] == "Main")
    );
}

#[test]
fn shutdown_rejects_requests_until_exit() {
    let root = tempfile::tempdir().unwrap();
    let mut server = LanguageServer::new(root.path().into()).unwrap();
    let response = request(&mut server, 1, "shutdown", Value::Null);
    assert!(response["result"].is_null());
    let rejected = request(&mut server, 2, "workspace/symbol", Value::Null);
    assert_eq!(rejected["error"]["code"], -32600);
    let _ = server.handle_message(&json!({"jsonrpc": "2.0", "method": "exit"}));
    assert!(server.exited());
}

/// Recurses into source folders while excluding generated/dependency folders.
#[test]
fn workspace_indexes_nested_argui_files_and_skips_generated_trees() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("nested")).unwrap();
    for directory in [".git", ".codex", "target", "node_modules"] {
        fs::create_dir_all(root.path().join(directory)).unwrap();
        fs::write(
            root.path().join(directory).join("ignored.argui"),
            "export component Ignored {}",
        )
        .unwrap();
    }
    fs::write(
        root.path().join("nested/visible.argui"),
        "export component Visible {}",
    )
    .unwrap();
    fs::write(root.path().join("notes.txt"), "not a DSL module").unwrap();

    let mut server = LanguageServer::new(root.path().into()).unwrap();
    let symbols = request(&mut server, 1, "workspace/symbol", Value::Null);
    let symbols = symbols["result"].as_array().unwrap();
    assert!(symbols.iter().any(|symbol| symbol["name"] == "Visible"));
    assert!(!symbols.iter().any(|symbol| symbol["name"] == "Ignored"));
}

/// Restores an on-disk overlay and exercises warning, URI, color, and UTF-16 paths.
#[test]
fn server_restores_disk_overlay_and_reports_warning_and_position_errors() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("space dir");
    fs::create_dir(&root).unwrap();
    let path = root.join("main.argui");
    let source = r##"import { Text } from "@argui/native"
export component Main {
    private property values: array<string> = ["one"]
    for item in values { Text { content: item } }
}
"##;
    fs::write(&path, source).unwrap();
    let uri = file_uri(&path);
    let mut server = LanguageServer::new(root.clone()).unwrap();

    let initialized = request(&mut server, 1, "initialize", json!({}));
    assert!(initialized["result"]["capabilities"].is_object());
    let overlay = "/* 😀 same-line\ncontinued */\nexport component Main { Text { content: #112233 } }\n// #12\n";
    let opened = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": uri, "text": overlay}}
    }));
    assert_eq!(opened.len(), 1);

    let invalid_position = request(
        &mut server,
        2,
        "textDocument/hover",
        json!({"textDocument": {"uri": file_uri(&path)}, "position": {"line": 0, "character": 4}}),
    );
    assert_eq!(invalid_position["error"]["code"], -32602);
    assert!(
        invalid_position["error"]["message"]
            .as_str()
            .unwrap()
            .contains("surrogate")
    );

    let colors = request(
        &mut server,
        3,
        "textDocument/documentColor",
        json!({"textDocument": {"uri": file_uri(&path)}}),
    );
    assert_eq!(colors["result"].as_array().unwrap().len(), 1);
    let tokens = request(
        &mut server,
        4,
        "textDocument/semanticTokens/full",
        json!({"textDocument": {"uri": file_uri(&path)}}),
    );
    assert!(!tokens["result"]["data"].as_array().unwrap().is_empty());

    let closed = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didClose",
        "params": {"textDocument": {"uri": file_uri(&path)}}
    }));
    let diagnostics = closed[0]["params"]["diagnostics"].as_array().unwrap();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["code"] == "MissingRepeaterKey" && diagnostic["severity"] == 1
    }));

    let malformed = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": format!("file://{}/%ZZ.argui", root.display()), "text": ""}}
    }));
    assert_eq!(malformed[0]["method"], "window/logMessage");
}

#[test]
fn server_reports_malformed_requests_and_shutdown_notifications() {
    let root = tempfile::tempdir().unwrap();
    let mut server = LanguageServer::new(root.path().into()).unwrap();
    assert!(
        server
            .handle_message(&json!({"jsonrpc": "2.0", "id": 1}))
            .is_empty()
    );

    let notification = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/unknown",
        "params": {}
    }));
    assert_eq!(notification[0]["method"], "window/logMessage");
    assert_eq!(notification[0]["params"]["type"], 1);

    let invalid_root = request(
        &mut server,
        2,
        "initialize",
        json!({"rootUri": "https://example.test/project"}),
    );
    assert_eq!(invalid_root["error"]["code"], -32602);

    let initialized = request(
        &mut server,
        3,
        "initialize",
        json!({"rootPath": root.path().to_string_lossy()}),
    );
    assert!(initialized["result"]["capabilities"].is_object());

    let missing_open = request(
        &mut server,
        4,
        "textDocument/didOpen",
        json!({"textDocument": {"text": ""}}),
    );
    assert_eq!(missing_open["error"]["code"], -32602);
    let missing_change = request(
        &mut server,
        5,
        "textDocument/didChange",
        json!({"textDocument": {"uri": file_uri(&root.path().join("missing.argui"))}}),
    );
    assert_eq!(missing_change["error"]["code"], -32602);
    let missing_close = request(
        &mut server,
        6,
        "textDocument/didClose",
        json!({"textDocument": {}}),
    );
    assert_eq!(missing_close["error"]["code"], -32602);
}

#[test]
fn server_handles_empty_documents_noop_edits_and_missing_overlays() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("overlay.argui");
    let uri = file_uri(&path);
    let mut server = LanguageServer::new(root.path().into()).unwrap();
    let opened = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": uri.clone(), "text": "\n"}}
    }));
    assert_eq!(opened.len(), 1);
    assert!(
        opened[0]["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let formatting = request(
        &mut server,
        10,
        "textDocument/formatting",
        json!({"textDocument": {"uri": uri.clone()}}),
    );
    assert!(formatting["result"].as_array().unwrap().is_empty());
    let colors = request(
        &mut server,
        11,
        "textDocument/documentColor",
        json!({"textDocument": {"uri": uri.clone()}}),
    );
    assert!(colors["result"].as_array().unwrap().is_empty());
    let presentation = request(
        &mut server,
        12,
        "textDocument/colorPresentation",
        json!({
            "textDocument": {"uri": uri.clone()},
            "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}}
        }),
    );
    assert!(presentation["result"].as_array().unwrap().is_empty());
    let actions = request(
        &mut server,
        13,
        "textDocument/codeAction",
        json!({"textDocument": {"uri": uri.clone()}, "context": {"diagnostics": []}}),
    );
    assert!(actions["result"].as_array().unwrap().is_empty());

    let unknown_document = file_uri(&root.path().join("unknown.argui"));
    let hover = request(
        &mut server,
        14,
        "textDocument/hover",
        json!({"textDocument": {"uri": unknown_document}, "position": {"line": 0, "character": 0}}),
    );
    assert_eq!(hover["error"]["code"], -32602);

    let close = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didClose",
        "params": {"textDocument": {"uri": uri}}
    }));
    assert_eq!(close.len(), 1);
    assert!(
        close[0]["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(!path.exists());
    fs::write(&path, "export component Disk {}").unwrap();
}

fn position(source: &str, needle: &str, occurrence: usize) -> Value {
    let mut offset = 0;
    for _ in 0..=occurrence {
        offset = source[offset..]
            .find(needle)
            .map(|found| offset + found)
            .expect("test marker must exist");
        if occurrence > 0 {
            offset += needle.len();
        }
    }
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    json!({
        "line": line,
        "character": prefix[line_start..].encode_utf16().count(),
    })
}

fn setup() -> (tempfile::TempDir, String, String, LanguageServer) {
    let root = tempfile::tempdir().unwrap();
    let model = "export struct Project { name: string }\n";
    let source = r#"import { Project } from "./model.argui"
import { Button } from "@argui/ui"
import { Container } from "@argui/native"
export theme AppTheme { --accent: color = #369c }
export component Main {
    in property project: Project
    callback clicked()
    Button {
        text: "Save"
        on click { clicked() }
    }
    Container { backgroun: var(--accent) }
}
"#;
    fs::write(root.path().join("model.argui"), model).unwrap();
    fs::write(root.path().join("main.argui"), source).unwrap();
    let server = LanguageServer::new(root.path().into()).unwrap();
    (root, source.into(), model.into(), server)
}

#[test]
fn server_serves_navigation_symbols_and_all_document_edits() {
    let (root, source, _model, mut server) = setup();
    let uri = file_uri(&root.path().join("main.argui"));
    let root_uri = file_uri(root.path());
    assert!(
        request(&mut server, 1, "initialize", json!({"rootUri": root_uri}))["result"]
            ["capabilities"]["definitionProvider"]
            .as_bool()
            .unwrap()
    );

    let main_hover = request(
        &mut server,
        2,
        "textDocument/hover",
        json!({"textDocument": {"uri": uri}, "position": position(&source, "Main", 0)}),
    );
    assert!(
        main_hover["result"]["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("component Main")
    );

    let definition = request(
        &mut server,
        3,
        "textDocument/definition",
        json!({"textDocument": {"uri": uri}, "position": position(&source, "Project", 1)}),
    );
    assert!(
        definition["result"]["uri"]
            .as_str()
            .unwrap()
            .ends_with("model.argui")
    );

    let references = request(
        &mut server,
        4,
        "textDocument/references",
        json!({"textDocument": {"uri": uri}, "position": position(&source, "Project", 1)}),
    );
    assert!(references["result"].as_array().unwrap().len() >= 2);

    let rename = request(
        &mut server,
        5,
        "textDocument/rename",
        json!({"textDocument": {"uri": uri}, "position": position(&source, "Project", 1), "newName": "WorkspaceProject"}),
    );
    assert!(!rename["result"]["changes"].as_object().unwrap().is_empty());

    let document_symbols = request(
        &mut server,
        6,
        "textDocument/documentSymbol",
        json!({"textDocument": {"uri": uri}}),
    );
    assert!(
        document_symbols["result"]
            .as_array()
            .unwrap()
            .iter()
            .any(|symbol| symbol["name"] == "Main")
    );
    let workspace_symbols = request(&mut server, 7, "workspace/symbol", Value::Null);
    assert!(
        workspace_symbols["result"]
            .as_array()
            .unwrap()
            .iter()
            .any(|symbol| symbol["name"] == "Project")
    );

    let formatting = request(
        &mut server,
        8,
        "textDocument/formatting",
        json!({"textDocument": {"uri": uri}, "options": {"tabSize": 4, "insertSpaces": true}}),
    );
    assert!(formatting["result"].is_array());
    let colors = request(
        &mut server,
        9,
        "textDocument/documentColor",
        json!({"textDocument": {"uri": uri}}),
    );
    assert_eq!(colors["result"].as_array().unwrap().len(), 1);
    let color = request(
        &mut server,
        10,
        "textDocument/colorPresentation",
        json!({
            "textDocument": {"uri": uri},
            "range": {"start": position(&source, "#369c", 0), "end": position(&source, "#369c", 0)},
            "color": {"red": 0.2, "green": 0.4, "blue": 0.6, "alpha": 0.8}
        }),
    );
    assert_eq!(color["result"][0]["label"], "#336699cc");

    let unknown = request(
        &mut server,
        11,
        "textDocument/hover",
        json!({"textDocument": {"uri": uri}, "position": {"line": 99, "character": 0}}),
    );
    assert_eq!(unknown["error"]["code"], -32602);
}

#[test]
fn server_handles_overlays_diagnostics_quickfixes_and_lifecycle_errors() {
    let (root, source, _model, mut server) = setup();
    let uri = file_uri(&root.path().join("main.argui"));
    let source_with_error = source.replace("backgroun", "background");
    let opened = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": uri, "languageId": "argui", "version": 1, "text": source_with_error}}
    }));
    assert_eq!(opened.len(), 1);
    assert!(
        opened[0]["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let changed = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {"textDocument": {"uri": uri, "version": 2}, "contentChanges": [{"text": source}]}
    }));
    assert!(
        changed[0]["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["code"] == "UnknownProperty")
    );

    let actions = request(
        &mut server,
        20,
        "textDocument/codeAction",
        json!({"textDocument": {"uri": uri}, "range": {"start": {"line": 7, "character": 8}, "end": {"line": 7, "character": 16}}, "context": {"diagnostics": []}}),
    );
    assert!(
        actions["result"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| action["title"].as_str().unwrap().contains("background"))
    );

    let invalid_completion = request(
        &mut server,
        21,
        "textDocument/completion",
        json!({"textDocument": {"uri": uri}, "position": {"line": 0}}),
    );
    assert_eq!(invalid_completion["error"]["code"], -32602);
    let unknown_method = request(&mut server, 22, "textDocument/unknown", Value::Null);
    assert_eq!(unknown_method["error"]["code"], -32601);

    let close = server.handle_message(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didClose",
        "params": {"textDocument": {"uri": uri}}
    }));
    assert_eq!(close.len(), 1);
    let _ = server.handle_message(&json!({"jsonrpc": "2.0", "method": "shutdown"}));
    let rejected = request(&mut server, 23, "workspace/symbol", Value::Null);
    assert_eq!(rejected["error"]["code"], -32600);
    let _ = server.handle_message(&json!({"jsonrpc": "2.0", "method": "exit"}));
    assert!(server.exited());
}
