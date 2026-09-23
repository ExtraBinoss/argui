//! AOT expression forms exercised through the public compiler boundary.

use argui_dsl_compiler::{Compiler, SourceModule};

#[test]
fn aot_emits_all_supported_gradient_geometries() {
    let source = r#"import { Container, Text } from "@argui/native"
export component Main {
    Container {
        Text {
            content: "linear"
            selection_fill: linear_gradient([#ff0000, #00ff00, #0000ff], [0.0, 0.5, 1.0], 45.0, "oklab")
        }
        Text {
            content: "radial"
            selection_fill: radial_gradient([#ffffff, #000000], [0.0, 1.0], 0.5, 0.5, 0.7, 0.7, "srgb")
        }
        Text {
            content: "conic"
            selection_fill: conic_gradient([#ff0000, #0000ff], [0.0, 1.0], 0.5, 0.5, 90.0, "linear-srgb")
        }
    }
}"#;
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    for constructor in ["linear_gradient(", "radial_gradient(", "conic_gradient("] {
        assert!(compiled.rust.contains(constructor), "missing {constructor}");
    }
    assert_eq!(
        compiled
            .rust
            .matches("invalid authored gradient stops or color space")
            .count(),
        3
    );
}

#[test]
fn aot_emits_boolean_arithmetic_and_comparison_expressions() {
    let source = r#"import { Text } from "@argui/native"
export component Main {
    in property first: int = 9
    in property second: int = 4
    Text { content: (first % second == 1 && first >= second) || first < second ? "yes" : "no" }
}"#;
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    for operator in [".checked_rem(", " == ", " && ", " >= ", " || ", " < "] {
        assert!(compiled.rust.contains(operator), "missing {operator}");
    }
}
