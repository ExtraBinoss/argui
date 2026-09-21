//! Semantic contracts for property animations and directional transitions.

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

/// Returns semantic diagnostics for one self-contained DSL source.
///
/// * `source` — DSL module being checked against built-in schemas.
fn diagnostics(source: &str) -> Vec<argui_dsl_semantic::Diagnostic> {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("ui/animation.argui", source);
    database.check().diagnostics.clone()
}

#[test]
fn animation_targets_any_element_or_user_component_property_by_type() {
    let issues = diagnostics(
        r#"import { Column, Container, Svg, Image } from "@argui/native"
export component Meter {
    in property progress: float = 0.0
    animate progress { duration: 180ms }
    Container { width: 20px animate width { from: 20px to: 80px duration: 400ms } }
}
export component Main {
    Column {
        gap: 8.0
        animate gap { duration: 200ms }
        Meter { progress: 1.0 animate progress { from: 0.0 to: 1.0 duration: 500ms iterations: "infinite" } }
        Svg { source: asset("spinner.svg") rotation: 0.0 animate rotation { from: 0.0 to: 360.0 duration: 1000ms iterations: "infinite" } }
        Image { source: asset("photo.png") rotation: 0.0 animate rotation { duration: 300ms } }
    }
}"#,
    );
    assert!(
        issues.is_empty(),
        "unexpected animation errors: {issues:#?}"
    );
}

#[test]
fn animation_reports_unknown_and_non_interpolable_targets() {
    let issues = diagnostics(
        r#"import { Text, Container } from "@argui/native"
export component Main {
    Text {
        content: "hello"
        animate missing { duration: 100ms }
        animate content { duration: 100ms }
    }
    Container { scroll_y: true animate scroll_y { duration: 100ms } }
}"#,
    );
    assert!(issues.iter().any(|issue| {
        issue.code == DiagnosticCode::UnknownProperty && issue.message.contains("missing")
    }));
    assert!(issues.iter().any(|issue| {
        issue.code == DiagnosticCode::InvalidAnimation && issue.message.contains("content")
    }));
    assert!(issues.iter().any(|issue| {
        issue.code == DiagnosticCode::InvalidAnimation && issue.message.contains("scroll_y")
    }));
}

#[test]
fn animation_checks_driver_parameters_and_timeline_requirements() {
    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component Main {
    Column {
        gap: 8.0
        animate gap {
            from: "bad"
            to: 20.0
            duration: 10
            duration: 20ms
            iterations: "forever"
            unknown: 1
        }
        animate gap { iterations: "infinite" duration: 10ms }
        animate gap { from: 0.0 to: 20.0 }
    }
}"#,
    );
    assert!(issues.iter().any(
        |issue| issue.code == DiagnosticCode::TypeMismatch && issue.message.contains("`from`")
    ));
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == DiagnosticCode::TypeMismatch
                && issue.message.contains("`duration`"))
    );
    for expected in [
        "assigned more than once",
        "unknown animation parameter",
        "must be",
        "requires both",
        "requires `duration`",
    ] {
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == DiagnosticCode::InvalidAnimation
                    && issue.message.contains(expected)),
            "missing {expected:?}: {issues:#?}"
        );
    }
}

#[test]
fn spring_driver_remains_valid_for_implicit_transitions() {
    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component Main { Column { gap: 4.0 animate gap { spring { stiffness: 220 damping: 24 } } } }"#,
    );
    assert!(issues.is_empty(), "unexpected spring errors: {issues:#?}");
}

#[test]
fn spring_parameters_select_the_driver_and_reject_unused_duration() {
    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component Main { Column { gap: 4.0 animate gap { from: 0.0 to: 20.0 stiffness: 220.0 damping: 24.0 } } }"#,
    );
    assert!(
        issues.is_empty(),
        "direct spring parameters should be valid: {issues:#?}"
    );

    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component Main { Column { gap: 4.0 animate gap { spring { duration: 100ms } } } }"#,
    );
    assert!(issues.iter().any(|issue| {
        issue.code == DiagnosticCode::InvalidAnimation
            && issue.message.contains("cannot use `duration`")
    }));
}

#[test]
fn timeline_keyframes_and_css_easing_are_typed_for_any_numeric_property() {
    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component Main { Column { gap: 4.0 animate gap {
    duration: 400ms easing: ease-out iterations: "infinite"
    keyframes { 0%: 0.0 50%: 20.0 100%: 4.0 }
} } }"#,
    );
    assert!(
        issues.is_empty(),
        "valid keyframes should type-check: {issues:#?}"
    );

    let issues = diagnostics(
        r#"import { Container } from "@argui/native"
component Tile { in property tone: color = #ffffff Container { background: tone } }
export component Main { Tile { tone: #000000 animate tone {
    duration: 200ms easing: "ease-in-out"
    keyframes { 0%: #000000 50%: #ff0000 100%: #ffffff }
} } }"#,
    );
    assert!(
        issues.is_empty(),
        "user-component color keyframes should type-check: {issues:#?}"
    );
}

#[test]
fn invalid_keyframes_report_order_endpoints_type_and_driver_conflicts() {
    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component Main { Column { gap: 4.0 animate gap {
    easing: "bounce" from: 0.0 duration: 400ms
    keyframes { 10%: 0.0 50%: "bad" 40%: 2.0 110%: 4.0 }
} } }"#,
    );
    for expected in [
        "`easing` must",
        "strictly increasing",
        "between 0% and 100%",
        "start at 0% and end at 100%",
        "cannot be combined",
    ] {
        assert!(
            issues.iter().any(|issue| issue.message.contains(expected)),
            "missing {expected:?}: {issues:#?}"
        );
    }
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == DiagnosticCode::TypeMismatch
                && issue.message.contains("keyframe"))
    );

    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component Main { Column { gap: 4.0 animate gap {
    spring {} easing: "ease-in" keyframes { 0%: 0.0 100%: 4.0 }
} } }"#,
    );
    for expected in ["cannot be combined", "cannot use timeline `easing`"] {
        assert!(
            issues.iter().any(|issue| issue.message.contains(expected)),
            "missing {expected:?}: {issues:#?}"
        );
    }
}

#[test]
fn bare_easing_names_are_checked_without_masking_bound_string_properties() {
    let invalid = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { Container {
    rotation: 0.0 animate rotation { duration: 100ms easing: bounce }
} }"#,
    );
    assert!(invalid.iter().any(|issue| {
        issue.code == DiagnosticCode::InvalidAnimation && issue.message.contains("`easing` must be")
    }));
    assert!(
        !invalid
            .iter()
            .any(|issue| issue.code == DiagnosticCode::UnknownName)
    );

    let dynamic = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { in property curve: string = "ease-out" Container {
    rotation: 0.0 animate rotation { duration: 100ms easing: curve }
} }"#,
    );
    assert!(
        dynamic.is_empty(),
        "bound string easing should remain legal: {dynamic:#?}"
    );

    let keyword_collision = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { in property ease-out: int = 1 Container {
    rotation: 0.0 animate rotation { duration: 100ms easing: ease-out }
} }"#,
    );
    assert!(
        keyword_collision.is_empty(),
        "CSS easing names are contextual literals: {keyword_collision:#?}"
    );
}

#[test]
fn unbound_native_animation_requires_a_target_endpoint() {
    let issues = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { Container { animate rotation { duration: 100ms } } }"#,
    );
    assert!(issues.iter().any(|issue| {
        issue.code == DiagnosticCode::InvalidAnimation
            && issue
                .message
                .contains("requires a `to` endpoint or `keyframes`")
    }));

    let issues = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { Container { animate rotation {
    duration: 100ms keyframes { 0%: 0.0 100%: 90.0 }
} } }"#,
    );
    assert!(
        issues.is_empty(),
        "keyframes supply an unbound native target: {issues:#?}"
    );
}

#[test]
fn directional_transitions_require_a_matching_state_and_valid_policy() {
    let valid = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { in property expanded: bool = false Container {
    rotation: 0.0
    states { open when expanded { rotation: 90.0 } }
    animate rotation { transition: in-out duration: 200ms easing: ease-out }
} }"#,
    );
    assert!(
        valid.is_empty(),
        "state transition should type-check: {valid:#?}"
    );

    let quoted = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { in property expanded: bool = false Container {
    rotation: 0.0 states { open when expanded { rotation: 90.0 } }
    animate rotation { transition: "leave" spring { stiffness: 220 damping: 24 } }
} }"#,
    );
    assert!(
        quoted.is_empty(),
        "quoted spring transition should type-check: {quoted:#?}"
    );

    let unbound_with_default = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { in property expanded: bool = false Container {
    states { open when expanded { opacity: 0.5 } }
    animate opacity { transition: enter duration: 200ms }
} }"#,
    );
    assert!(
        unbound_with_default.is_empty(),
        "schema default supplies a base: {unbound_with_default:#?}"
    );

    let invalid = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { in property expanded: bool = false Container {
    rotation: 0.0
    animate rotation { transition: bounce from: 0.0 to: 90.0 duration: 200ms iterations: "infinite" }
} }"#,
    );
    for expected in [
        "static policy",
        "requires a state",
        "cannot use `from`, `to`",
    ] {
        assert!(
            invalid
                .iter()
                .any(|issue| issue.code == DiagnosticCode::InvalidAnimation
                    && issue.message.contains(expected)),
            "missing {expected:?}: {invalid:#?}"
        );
    }

    let wrong_property = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main { in property expanded: bool = false Container {
    rotation: 0.0
    states { open when expanded { opacity: 0.5 } }
    animate rotation { transition: enter duration: 200ms }
} }"#,
    );
    assert!(wrong_property.iter().any(|issue| {
        issue.code == DiagnosticCode::InvalidAnimation
            && issue
                .message
                .contains("requires a state on the same owner assigning that property")
    }));
}

#[test]
fn native_schema_can_mark_structural_properties_non_animatable() {
    let issues = diagnostics(
        r#"import { VList, Text } from "@argui/native"
export component Main { private property items: array<string> = ["one"] VList {
    row_height: 32.0 opacity: 1.0 rotation: 0.0
    animate row_height { duration: 100ms }
    animate opacity { duration: 100ms }
    animate rotation { duration: 100ms }
    for item in items key item { Text { content: item } }
} }"#,
    );
    let animation_errors = issues
        .iter()
        .filter(|issue| issue.code == DiagnosticCode::InvalidAnimation)
        .collect::<Vec<_>>();
    assert_eq!(
        animation_errors.len(),
        1,
        "only row_height should be rejected: {issues:#?}"
    );
    assert!(
        animation_errors[0]
            .message
            .contains("`row_height` is structural")
    );
}

#[test]
fn dimension_animation_rejects_mixed_literal_units() {
    let issues = diagnostics(
        r#"import { Container } from "@argui/native"
export component Main {
    Container { width: 20px animate width { from: 20px to: 50% duration: 200ms } }
}"#,
    );
    assert!(issues.iter().any(|issue| {
        issue.code == DiagnosticCode::InvalidAnimation
            && issue.message.contains("pixel and percentage")
    }));
}

#[test]
fn animation_rejects_duplicate_targets_on_one_owner() {
    let issues = diagnostics(
        r#"import { Column } from "@argui/native"
export component Main {
    private property progress: float = 0.0
    animate progress { duration: 100ms }
    animate progress { duration: 200ms }
    Column {
        gap: 4.0
        animate gap { duration: 100ms }
        animate gap { duration: 200ms }
    }
}"#,
    );
    assert_eq!(
        issues
            .iter()
            .filter(|issue| {
                issue.code == DiagnosticCode::InvalidAnimation
                    && issue.message.contains("more than one animation")
            })
            .count(),
        2
    );
}
