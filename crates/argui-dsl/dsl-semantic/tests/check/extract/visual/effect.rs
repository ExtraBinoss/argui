//! Generic visual effect application diagnostics.

use argui_dsl_semantic::{CompilerDatabase, Diagnostic, DiagnosticCode};

/// Checks one source file and returns all semantic diagnostics.
fn diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("ui/main.argui", source);
    database.check().diagnostics.clone()
}

/// All visual primitives accept the same named, typed effect application.
#[test]
fn effect_arguments_are_valid_on_rectangle_path_svg_and_image() {
    let issues = diagnostics(
        r##"import { Rectangle, Path, Svg, Image } from "@argui/native"
export effect Glow {
    shader: "effects/glow.wgsl"
    parameter amount: float = 0.5
    parameter tint: color = #ffffff
}
export component Main {
    private property strength: float = 0.7
    Rectangle { effect: Glow { amount: strength tint: #ff0000 } }
    Path {
        source: path(20.0, 20.0, [move_to(1.0, 1.0), line_to(19.0, 19.0)], true, 0.0, false)
        effect: Glow { amount: strength }
    }
    Svg { source: asset("icons/star.svg") effect: Glow { tint: #00ff00 } }
    Image { source: asset("images/photo.png") effect: Glow {} }
}"##,
    );
    assert!(issues.is_empty(), "{issues:#?}");
}

/// Unknown declarations and arguments, mismatched types, and missing required
/// values produce source diagnostics before IR lowering.
#[test]
fn invalid_effect_bindings_are_reported() {
    let issues = diagnostics(
        r#"import { Rectangle } from "@argui/native"
export effect Glow {
    shader: "effects/glow.wgsl"
    parameter amount: float
}
export component Main {
    Rectangle { effect: Missing {} }
    Rectangle { effect: Glow { amount: "wrong" extra: 1 } }
    Rectangle { effect: Glow {} }
    Rectangle { effect: Glow { amount: 0.5 amount: 0.7 } }
    Rectangle { effect: Glow { amount: 0.5 } effect: Glow { amount: 0.7 } }
}"#,
    );
    for (code, fragment) in [
        (DiagnosticCode::InvalidEffect, "unknown effect `Missing`"),
        (DiagnosticCode::InvalidEffect, "no parameter `extra`"),
        (DiagnosticCode::InvalidEffect, "requires parameter `amount`"),
        (DiagnosticCode::InvalidEffect, "only one effect"),
        (DiagnosticCode::TypeMismatch, "parameter `amount`"),
        (DiagnosticCode::DuplicateMember, "assigned more than once"),
    ] {
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == code && issue.message.contains(fragment)),
            "missing {code:?} / {fragment}: {issues:#?}"
        );
    }
}

/// Transform parameters receive a diagnostic until transform values have a
/// runtime representation for both compilation modes.
#[test]
fn transform_effect_parameter_is_diagnosed_at_visual_use() {
    let issues = diagnostics(
        r#"import { Rectangle } from "@argui/native"
export effect Warp {
    shader: "effects/warp.wgsl"
    parameter matrix: transform
}
export component Main { Rectangle { effect: Warp {} } }"#,
    );
    assert!(
        issues.iter().any(|issue| {
            issue.code == DiagnosticCode::InvalidEffect
                && issue
                    .message
                    .contains("parameter `matrix` has type `transform`")
        }),
        "{issues:#?}"
    );
}

/// Paint-region scopes are accepted while dynamic or unknown scopes are rejected.
#[test]
fn effect_scope_requires_a_known_literal_region() {
    let issues = diagnostics(
        r#"import { Rectangle } from "@argui/native"
export effect Glow { shader: "effects/glow.wgsl" }
export component Main {
    private property region: string = "border"
    Rectangle { effect: Glow { scope: "border" } }
    Rectangle { effect: Glow { scope: "backdrop" } }
    Rectangle { effect: Glow { scope: "unknown" } }
    Rectangle { effect: Glow { scope: region } }
}"#,
    );
    assert_eq!(
        issues
            .iter()
            .filter(|issue| issue.code == DiagnosticCode::InvalidEffect)
            .count(),
        2,
        "{issues:#?}"
    );
}
