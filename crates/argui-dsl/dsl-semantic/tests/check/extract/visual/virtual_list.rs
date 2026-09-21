use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode, Severity};

/// Checks one module with the built-in native schemas loaded.
fn diagnostics(source: &str) -> Vec<argui_dsl_semantic::Diagnostic> {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("ui/main.argui", source);
    database.check().diagnostics.clone()
}

#[test]
fn keyed_virtual_repeater_accepts_typed_rows_and_two_way_scroll() {
    let diagnostics = diagnostics(
        r#"import { VList, Text } from "@argui/native"
export component Main {
    private property items: array<string> = ["one", "two"]
    private property offset: float = 0.0
    VList { row_height: 32.0 height: 100% offset <=> offset opacity: 1.0
        states { dim when false { opacity: 0.5 } }
        for item in items key item { Text { content: item } }
    }
}"#,
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != Severity::Error),
        "{diagnostics:#?}"
    );
}

#[test]
fn virtual_list_requires_reactive_scroll_offset() {
    for assignment in ["", "offset: 0.0"] {
        let source = format!(
            "import {{ VList, Text }} from \"@argui/native\"\nexport component Main {{\n    private property items: array<string> = [\"one\"]\n    private property scroll: float = 0.0\n    VList {{ row_height: 32.0 {assignment}\n        for item in items key item {{ Text {{ content: item }} }}\n    }}\n}}"
        );
        let issues = diagnostics(&source);
        assert!(
            issues.iter().any(|issue| {
                issue.code == DiagnosticCode::InvalidTwoWayBinding
                    && issue.message.contains("offset <=> writable_property")
            }),
            "{assignment:?}: {issues:#?}"
        );
    }
}

#[test]
fn virtual_list_requires_one_keyed_repeater_and_owns_window_metadata() {
    for (body, expected) in [
        (
            "Text { content: \"wrong\" }",
            "visual child must be a keyed `for`",
        ),
        (
            "for item in items { Text { content: item } }",
            "stable `key` expression",
        ),
        (
            "for item in items key item { Text { content: item } } Text { content: \"extra\" }",
            "exactly one keyed `for`",
        ),
        (
            "__item_count: 42 for item in items key item { Text { content: item } }",
            "compiler-owned",
        ),
        (
            "states { compact when true { row_height: 24.0 } } for item in items key item { Text { content: item } }",
            "cannot be overridden by `states`",
        ),
    ] {
        let source = format!(
            "import {{ VList, Text }} from \"@argui/native\"\nexport component Main {{ private property items: array<string> = [\"one\"] VList {{ row_height: 32.0 {body} }} }}"
        );
        let diagnostics = diagnostics(&source);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error
                    && diagnostic.message.contains(expected)),
            "{body}: {diagnostics:#?}"
        );
    }
    let missing =
        diagnostics("import { VList } from \"@argui/native\"\nexport component Main { VList { } }");
    assert!(
        missing
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MissingProperty)
    );
}

#[test]
fn template_slot_forwards_one_caller_scoped_repeater() {
    let issues = diagnostics(
        r#"import { VList, Text } from "@argui/native"
export struct Item { id: int label: string }
component VirtualList {
    in property row_height: float
    in-out property scroll: float = 0.0
    slot rows: template
    VList { row_height: row_height offset <=> scroll rows }
}
export component Main {
    private property items: array<Item>
    private property offset: float = 0.0
    VirtualList { row_height: 32.0 scroll <=> offset
        for item in items key item.id { Text { content: item.label } }
    }
}"#,
    );
    assert!(
        issues.iter().all(|issue| issue.severity != Severity::Error),
        "{issues:#?}"
    );
}

#[test]
fn template_slot_rejects_eager_children_unkeyed_rows_and_wrong_forwarding() {
    let wrapper = r#"import { VList, Text } from "@argui/native"
component VirtualList {
    in property row_height: float
    in-out property scroll: float = 0.0
    slot rows: template
    VList { row_height: row_height offset <=> scroll rows }
}
export component Main {
    private property items: array<string> = ["one"]
    private property offset: float = 0.0
    VirtualList { row_height: 32.0 scroll <=> offset BODY }
}"#;
    for (body, expected) in [
        (
            "Text { content: \"eager\" }",
            "requires a keyed `for` repeater",
        ),
        (
            "for item in items { Text { content: item } }",
            "stable `key` expression",
        ),
        ("", "exactly one keyed `for` repeater"),
        (
            "for item in items key item { Text { content: item } } for item in items key item { Text { content: item } }",
            "exactly one keyed `for` repeater",
        ),
    ] {
        let issues = diagnostics(&wrapper.replace("BODY", body));
        assert!(
            issues
                .iter()
                .any(|issue| issue.severity == Severity::Error && issue.message.contains(expected)),
            "{body}: {issues:#?}"
        );
    }
    let issues = diagnostics(
        &wrapper
            .replace(
                "BODY",
                "for item in items key item { Text { content: item } }",
            )
            .replace(
                "VList { row_height: row_height offset <=> scroll rows }",
                "VList { row_height: row_height offset <=> scroll Text { content: \"wrong\" } }",
            ),
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("visual child must be a keyed `for`")),
        "{issues:#?}"
    );
}

#[test]
fn template_slot_kind_and_usage_are_source_located() {
    let issues = diagnostics(
        r#"import { VList, Text } from "@argui/native"
component Broken {
    in-out property scroll: float = 0.0
    slot rows: eager
    VList { row_height: 24.0 offset <=> scroll rows }
}"#,
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("unknown slot kind `eager`")
                && issue.primary.range.start() < issue.primary.range.end()),
        "{issues:#?}"
    );
    let issues = diagnostics(
        r#"import { VList } from "@argui/native"
component Broken {
    in-out property scroll: float = 0.0
    slot rows: template
    VList { row_height: 24.0 offset <=> scroll }
}"#,
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("exactly one native VList")),
        "{issues:#?}"
    );

    let issues = diagnostics(
        r#"import { VList } from "@argui/native"
component Broken {
    in-out property scroll: float = 0.0
    slot rows: template
    slot footer
    VList { row_height: 24.0 offset <=> scroll rows }
}"#,
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("component's only slot")),
        "{issues:#?}"
    );
}

#[test]
fn template_slot_does_not_skip_native_metadata_or_structural_state_checks() {
    let issues = diagnostics(
        r#"import { VList } from "@argui/native"
component VirtualList {
    in-out property scroll: float = 0.0
    slot rows: template
    VList {
        row_height: 24.0
        offset <=> scroll
        __item_count: 5
        states { compact when true { row_height: 20.0 } }
        rows
    }
}"#,
    );
    for expected in ["compiler-owned", "cannot be overridden by `states`"] {
        assert!(
            issues
                .iter()
                .any(|issue| issue.severity == Severity::Error && issue.message.contains(expected)),
            "{expected}: {issues:#?}"
        );
    }
}
