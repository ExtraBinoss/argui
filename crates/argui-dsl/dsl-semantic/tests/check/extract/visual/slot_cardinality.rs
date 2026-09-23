//! Required and single-child slot contracts.

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode, Severity};

/// Checks one component source and returns its semantic diagnostics.
fn diagnostics(source: &str) -> Vec<argui_dsl_semantic::Diagnostic> {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "main.argui",
        format!("import {{ Text }} from \"@argui/native\"\n{source}"),
    );
    database.check().diagnostics.clone()
}

/// Required slots diagnose omitted content, while optional and populated slots pass.
#[test]
fn required_and_optional_slot_contracts_accept_valid_content() {
    for source in [
        "component Panel { slot body: required } export component Main { Panel { slot body { Text {} Text {} } } }",
        "component Panel { slot header: single } export component Main { Panel {} }",
        "component Panel { slot header: required single } export component Main { Panel { slot header { Text {} } } }",
    ] {
        let issues = diagnostics(source);
        assert!(
            issues.is_empty(),
            "unexpected diagnostics for {source}: {issues:#?}"
        );
    }

    for (source, expected) in [
        (
            "component Panel { slot body: required } export component Main { Panel {} }",
            "required slot `body` is missing",
        ),
        (
            "component Panel { slot header: required single } export component Main { Panel {} }",
            "required slot `header` is missing",
        ),
    ] {
        let issues = diagnostics(source);
        assert!(
            issues.iter().any(|issue| {
                issue.code == DiagnosticCode::TypeMismatch
                    && issue.severity == Severity::Error
                    && issue.message == expected
            }),
            "expected {expected:?} for {source}: {issues:#?}"
        );
    }
}

/// Single-child slots reject multiple named or implicit children.
#[test]
fn single_slots_reject_multiple_children() {
    for source in [
        "component Panel { slot header: single } export component Main { Panel { slot header { Text {} Text {} } } }",
        "component Panel { slot body: required single } export component Main { Panel { Text {} Text {} } }",
    ] {
        let issues = diagnostics(source);
        assert!(
            issues.iter().any(|issue| {
                issue.code == DiagnosticCode::TypeMismatch
                    && issue.severity == Severity::Error
                    && issue.message.contains("accepts one visual child")
            }),
            "expected a single-child diagnostic for {source}: {issues:#?}"
        );
    }
}

/// Typed row templates accept matching item types and reject a mismatched collection.
#[test]
fn template_row_parameter_checks_caller_item_type() {
    let prefix = r#"import { VirtualWindow } from "@argui/native"
component List {
    private property offset: float = 0.0
    slot rows(item: string): template
    VirtualWindow { row_height: 20.0 offset <=> offset rows }
}
export component Main {
"#;
    let suffix = r#"
    List { for item in items key item { Text { content: "cell" } } }
}"#;
    let valid = format!("{prefix}    private property items: array<string> = [\"a\"]{suffix}");
    let issues = diagnostics(&valid);
    assert!(issues.is_empty(), "{issues:#?}");

    let invalid = format!("{prefix}    private property items: array<int> = [1]{suffix}");
    let issues = diagnostics(&invalid);
    assert!(
        issues.iter().any(|issue| issue
            .message
            .contains("template slot `rows` expects row `String`, found `Int`")),
        "{issues:#?}"
    );
}

/// Single-child slots reject content whose rendered count cannot be proven statically.
#[test]
fn single_slot_rejects_dynamic_cardinality() {
    let source = "component Panel { slot header: single } export component Main { Panel { slot header { for item in [1, 2] key item { Text {} } } } }";
    let issues = diagnostics(source);
    assert!(
        issues.iter().any(|issue| issue
            .message
            .contains("cannot prove single-child cardinality")),
        "{issues:#?}"
    );
}
