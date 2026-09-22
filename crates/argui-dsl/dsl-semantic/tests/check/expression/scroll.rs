//! Boundary diagnostics for identity-targeted scroll requests.

use argui_dsl_semantic::CompilerDatabase;

/// Returns semantic messages for one scroll request in a native event handler.
///
/// * `call` — complete `scroll_to` invocation inside the handler.
fn diagnostics(call: &str) -> Vec<String> {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "app.argui",
        format!(
            "import {{ Flickable, TouchArea }} from \"@argui/native\" export component App {{ Flickable #viewport {{ TouchArea {{ on moved {{ {call} }} }} }} }}"
        ),
    );
    database
        .check()
        .diagnostics
        .iter()
        .map(|issue| issue.message.clone())
        .collect()
}

/// Empty calls report their arity without inventing a target diagnostic.
#[test]
fn scroll_request_with_no_operands_reports_only_arity() {
    let issues = diagnostics("scroll_to()");
    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("expects an element reference")),
        "{issues:#?}"
    );
    assert!(
        !issues.iter().any(|issue| issue.contains("target must be")),
        "{issues:#?}"
    );
}

/// Extra operands and invalid coordinates are diagnosed independently.
#[test]
fn scroll_request_checks_extra_coordinate_after_arity_failure() {
    let issues = diagnostics("scroll_to(#viewport, 0px, 12px, true)");
    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("expects an element reference")),
        "{issues:#?}"
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("expected `Length`")),
        "{issues:#?}"
    );
    assert!(
        !issues.iter().any(|issue| issue.contains("target must be")),
        "{issues:#?}"
    );
}
