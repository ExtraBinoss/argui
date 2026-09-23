//! Named styles reject unresolved, incompatible, and inaccessible assignments.
use argui_dsl_semantic::CompilerDatabase;

/// Shared styles can satisfy required component inputs and inline overrides.
#[test]
fn styles_apply_to_their_resolved_target() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "main.argui",
        r#"
import { Text } from "@argui/native"
component Label { in property text: string Text { content: text } }
style Caption for Label { text: "shared" }
export component Main { Label { style: Caption text: "inline" } Label { style: Caption } }
"#,
    );
    let project = database.check();
    assert!(project.is_valid(), "{:?}", project.diagnostics);
}

/// Each malformed style is diagnosed before Rust generation.
#[test]
fn invalid_styles_are_source_diagnostics() {
    for source in [
        r#"style S for Missing { width: 1px }"#,
        r#"style S for Text { missing: 1 }"#,
        r#"style S for Text { content: 1 }"#,
        r#"style S for Text { content: "a" content: "b" }"#,
        r#"style S for Text { unknown { content: "a" } }"#,
        r#"component C { private property text: string } style S for C { text: "a" }"#,
        r#"style S for Text { content: "a" } export component Main { Row { style: S } }"#,
        r#"style S for Text { content: "a" } export component Main { Text { style: S style: S } }"#,
        r#"export component Main { Text { style: Missing } }"#,
    ] {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file(
            "main.argui",
            format!("import {{ Text, Row }} from \"@argui/native\"\n{source}"),
        );
        assert!(!database.check().is_valid(), "accepted {source}");
    }
}
