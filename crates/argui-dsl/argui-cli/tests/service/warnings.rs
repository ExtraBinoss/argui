//! Accepted packages retain non-fatal semantic diagnostics for CLI output.

use argui_cli::DevCompilerService;
use argui_dsl_protocol::{LiveMessage, Severity};

/// Warnings are preserved without rejecting a valid source generation.
#[test]
fn successful_compile_keeps_source_warnings() {
    let directory = tempfile::tempdir().unwrap();
    let source = "import { TouchArea } from \"@argui/native\" export component Main { private property count: int = 0 TouchArea { on click { return; count = 1 } } }";
    std::fs::write(directory.path().join("main.argui"), source).unwrap();
    let mut service = DevCompilerService::open(directory.path(), "main.argui").unwrap();
    let result = service.compile();
    assert!(matches!(result.message, LiveMessage::Package(_)));
    let warning = result
        .warnings
        .iter()
        .find(|warning| warning.code == "UnreachableStatement")
        .unwrap();
    assert_eq!(warning.severity, Severity::Warning);
    assert_eq!(warning.path.as_deref(), Some("main.argui"));
    assert_eq!(
        &source[warning.start.unwrap() as usize..warning.end.unwrap() as usize],
        "count = 1"
    );
}

/// Rejected programs retain warnings alongside fatal semantic diagnostics.
#[test]
fn service_preserves_warning_severity_in_semantic_diagnostics() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    std::fs::write(
        directory.path().join("ui/main.argui"),
        r#"import { Column, Text, TouchArea } from "@argui/native"
export component Main {
    in property items: model<string>
    Column {
        for item in items { Text { content: item } }
        TouchArea { on click { return; str(1) } }
        Missing {}
    }
}"#,
    )
    .unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let LiveMessage::Diagnostics { diagnostics, .. } = service.compile().message else {
        panic!("invalid source should preserve warnings alongside errors");
    };
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == argui_dsl_protocol::Severity::Warning
            && diagnostic.code == "UnreachableStatement"
    }));
}
