//! Typed, effect-free function contracts.

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

/// Returns source diagnostics after resolving built-in symbols.
///
/// `source` is the complete DSL module.
fn diagnostics(source: &str) -> Vec<argui_dsl_semantic::Diagnostic> {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("app.argui", source);
    database.check().diagnostics.clone()
}

/// Pure signatures permit forward calls and reject arity and result mismatch.
#[test]
fn checks_function_signature_and_forward_calls() {
    let valid = "fn first(value: int) -> int = second(value) + 1\nfn second(value: int) -> int = value * 2\nexport component App { private property count: int = first(3) }";
    assert!(diagnostics(valid).is_empty(), "{:?}", diagnostics(valid));
    let invalid = "fn wrong(value: int) -> string = value + 1\nexport component App { private property count: int = wrong() }";
    let issues = diagnostics(invalid);
    assert!(
        issues
            .iter()
            .filter(|issue| issue.code == DiagnosticCode::TypeMismatch)
            .count()
            >= 2,
        "{issues:?}"
    );
}

/// Recursive calls and runtime effects cannot enter the pure expression graph.
#[test]
fn rejects_cycles_and_effectful_calls() {
    let source = "fn first(value: int) -> int = second(value)\nfn second(value: int) -> int = first(value)\nfn impure(value: int) -> int = set_theme_mode(\"dark\")";
    let issues = diagnostics(source);
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("function call cycle")),
        "{issues:?}"
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("effectful")),
        "{issues:?}"
    );
}
