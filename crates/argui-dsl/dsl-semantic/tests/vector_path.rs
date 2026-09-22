//! Authored path expression types and static geometry diagnostics.

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

/// Checks one project source and returns its semantic diagnostics.
fn diagnostics(source: &str) -> Vec<argui_dsl_semantic::Diagnostic> {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("ui/main.argui", source);
    database.check().diagnostics.clone()
}

/// A literal command array forms a typed asset for the generic Path primitive.
#[test]
fn literal_path_geometry_is_a_valid_asset_expression() {
    let issues = diagnostics(
        r#"import { Path } from "@argui/native"
export component Main {
    Path {
        source: path(20.0, 20.0, [move_to(1.0, 1.0), line_to(19.0, 1.0), quadratic_to(20.0, 5.0, 19.0, 19.0), cubic_to(15.0, 20.0, 5.0, 20.0, 1.0, 19.0), close_path()], true, 2.0, false)
        width: 20px
        height: 20px
        color: #ff0000
    }
}"#,
    );
    assert!(issues.is_empty(), "{issues:#?}");
}

/// Runtime-dependent command coordinates and malformed arity get source diagnostics.
#[test]
fn dynamic_or_malformed_path_commands_are_rejected() {
    let dynamic = diagnostics(
        r#"import { Path } from "@argui/native"
export component Main {
    private property offset: float = 2.0
    Path { source: path(20.0, 20.0, [move_to(offset, 0.0), line_to(10.0, 10.0)], true, 0.0, false) }
}"#,
    );
    assert!(
        dynamic.iter().any(|issue| {
            issue.code == DiagnosticCode::InvalidAsset
                && issue
                    .message
                    .contains("dynamic path geometry is unsupported")
        }),
        "{dynamic:#?}"
    );
    let malformed = diagnostics(
        r#"import { Path } from "@argui/native"
export component Main {
    Path { source: path(20.0, 20.0, [move_to(0.0), line_to(10.0, 10.0)], true, 0.0, false) }
}"#,
    );
    assert!(
        malformed.iter().any(|issue| {
            issue.code == DiagnosticCode::InvalidAsset
                && issue.message.contains("move_to() expects 2")
        }),
        "{malformed:#?}"
    );
}

/// Static paths reject malformed geometry and style at the offending argument.
#[test]
fn path_literals_reject_invalid_commands_coordinates_and_styles() {
    let cases = [
        ("path(20.0)", "expects width, height, commands"),
        (
            "path(offset, 20.0, [], true, 0.0, false)",
            "path width must be a numeric literal",
        ),
        (
            "path(20px, 20.0, [], true, 0.0, false)",
            "path width must be a finite, unitless",
        ),
        (
            "path(1e999, 20.0, [], true, 0.0, false)",
            "path width must be finite",
        ),
        (
            "path(-offset, 20.0, [], true, 0.0, false)",
            "dynamic path geometry",
        ),
        (
            "path(20.0, 20.0, offset, true, 0.0, false)",
            "path commands must be a literal",
        ),
        (
            "path(20.0, 20.0, [offset], true, 0.0, false)",
            "path commands must be literal constructor",
        ),
        (
            "path(20.0, 20.0, [arc_to(0.0)], true, 0.0, false)",
            "unknown path command",
        ),
        (
            "path(20.0, 20.0, [line_to(0.0, offset)], true, 0.0, false)",
            "dynamic path geometry",
        ),
        (
            "path(20.0, 20.0, [], offset, 0.0, false)",
            "path fill must be a bool literal",
        ),
        (
            "path(20.0, 20.0, [], true, 0.0, offset)",
            "path even_odd must be a bool literal",
        ),
    ];
    for (expression, expected) in cases {
        let source = format!(
            "import {{ Path }} from \"@argui/native\"\nexport component Main {{ private property offset: float = 1.0 Path {{ source: {expression} }} }}"
        );
        let issues = diagnostics(&source);
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == DiagnosticCode::InvalidAsset
                    && issue.message.contains(expected)),
            "{expression}: expected {expected:?}, got {issues:#?}"
        );
    }
}
