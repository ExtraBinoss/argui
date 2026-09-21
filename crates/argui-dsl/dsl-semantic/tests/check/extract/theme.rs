//! Public diagnostics for typed theme token values and mode overrides.

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

#[test]
fn theme_values_and_mode_overrides_are_checked_against_token_types() {
    let source = r##"export theme AppTheme {
    --accent: color = "not a color"
    --spacing: length = 12px
    dark {
        --accent: #ffffff
        --spacing: "not a length"
        --missing: 4px
    }
}"##;
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("app.argui", source);
    let project = database.check();
    let issues = &project.diagnostics;
    for name in ["--accent", "--spacing"] {
        assert!(
            issues.iter().any(|issue| {
                issue.code == DiagnosticCode::TypeMismatch && issue.message.contains(name)
            }),
            "missing type error for {name}: {issues:#?}"
        );
    }
    assert!(
        issues.iter().any(|issue| {
            issue.code == DiagnosticCode::UnknownName && issue.message.contains("--missing")
        }),
        "missing unknown mode token error: {issues:#?}"
    );
}

#[test]
fn theme_mode_accepts_matching_literal_and_token_reference() {
    let source = r##"export theme AppTheme {
    --base: color = #336699
    --accent: color = var(--base)
    dark { --accent: #ffffff }
}"##;
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("app.argui", source);
    let project = database.check();
    let issues = &project.diagnostics;
    assert!(
        issues.is_empty(),
        "valid theme reported errors: {issues:#?}"
    );
}
