//! Public diagnostics for keyframe stop and value validation.

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

/// Checks one DSL module and returns its semantic diagnostics.
///
/// * `source` — complete module containing an animated component.
fn diagnostics(source: &str) -> Vec<argui_dsl_semantic::Diagnostic> {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("app.argui", source);
    database.check().diagnostics.clone()
}

#[test]
fn keyframes_require_two_stops_and_complete_endpoints() {
    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component App { Column { gap: 4.0 animate gap {
    duration: 100ms keyframes { 50%: 2.0 }
} } }"#,
    );
    for expected in ["at least two stops", "start at 0% and end at 100%"] {
        assert!(
            issues.iter().any(|issue| {
                issue.code == DiagnosticCode::InvalidAnimation && issue.message.contains(expected)
            }),
            "missing {expected:?}: {issues:#?}"
        );
    }
}

#[test]
fn keyframes_reject_duplicate_offsets_and_mixed_dimension_units() {
    let issues = diagnostics(
        r#"import { Container } from "@argui/native"
export component App { Container { width: 20px animate width {
    duration: 100ms keyframes { 0%: 20px 50%: 30px 50%: 40% 100%: 60px }
} } }"#,
    );
    for expected in ["strictly increasing", "pixel and percentage dimensions"] {
        assert!(
            issues.iter().any(|issue| {
                issue.code == DiagnosticCode::InvalidAnimation && issue.message.contains(expected)
            }),
            "missing {expected:?}: {issues:#?}"
        );
    }
}

/// Percentages outside the legal interval and incomplete stops are diagnosed.
#[test]
fn keyframes_reject_out_of_range_and_missing_values() {
    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component App { Column { gap: 4.0 animate gap {
    duration: 100ms keyframes { 0%: 0.0 120%: 1.0 100%: }
} } }"#,
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == DiagnosticCode::InvalidAnimation
                && issue.message.contains("between 0% and 100%")),
        "{issues:#?}"
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("expected expression")),
        "{issues:#?}"
    );
}

/// Every stop must be assignable to its animated target, even at valid offsets.
#[test]
fn keyframes_reject_values_with_the_wrong_type() {
    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component App { Column { gap: 4.0 animate gap {
    duration: 100ms keyframes { 0%: false 100%: true }
} } }"#,
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == DiagnosticCode::TypeMismatch
                && issue.message.contains("keyframe expects")),
        "{issues:#?}"
    );
}
