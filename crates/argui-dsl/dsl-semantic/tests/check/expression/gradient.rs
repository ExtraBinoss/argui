//! Invalid static gradient shape and offset boundaries.

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

/// Malformed lengths, ranges, arity, and geometry report typed diagnostics.
#[test]
fn gradient_builtins_reject_shape_and_offset_boundaries() {
    let cases = [
        (
            "linear_gradient([#ffffff], [0.0], 0.0, \"oklab\")",
            "at least two",
        ),
        (
            "linear_gradient([#ffffff, #000000], [0.0], 0.0, \"oklab\")",
            "matching lengths",
        ),
        (
            "linear_gradient([#ffffff, #000000], [-0.1, 1.0], 0.0, \"oklab\")",
            "between 0 and 1",
        ),
        (
            "linear_gradient([#ffffff, #000000], [0.0, 1.1], 0.0, \"oklab\")",
            "between 0 and 1",
        ),
        (
            "radial_gradient([#ffffff, #000000], [0.0, 1.0], 0.5, 0.5, 0.7, \"srgb\")",
            "expects color and offset arrays",
        ),
        (
            "conic_gradient([#ffffff, #000000], [0.0, 1.0], 0.5, 0.5, true, \"srgb\")",
            "type mismatch",
        ),
    ];
    for (expression, expected) in cases {
        let source =
            format!("export component App {{ private property fill: brush = {expression} }}");
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        let issues = &database.check().diagnostics;
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == DiagnosticCode::TypeMismatch
                    && (issue.message.contains(expected) || expected == "type mismatch")),
            "{expression}: expected {expected:?}: {issues:#?}"
        );
    }
}

/// Dynamic offsets and color spaces keep their types while value checks defer to runtime.
#[test]
fn gradient_accepts_typed_dynamic_offset_arrays() {
    for source in [
        "export component App { private property offsets: array<float> = [0.0, 1.0] private property fill: brush = linear_gradient([#ffffff, #000000], offsets, 0.0, \"oklab\") }",
        "export component App { private property start: float = 0.0 private property fill: brush = linear_gradient([#ffffff, #000000], [start, 1.0], 0.0, \"oklab\") }",
        "export component App { private property space: string = \"srgb\" private property fill: brush = linear_gradient([#ffffff, #000000], [0.0, 1.0], 0.0, space) }",
    ] {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        let issues = &database.check().diagnostics;
        assert!(issues.is_empty(), "{issues:#?}");
    }
}
