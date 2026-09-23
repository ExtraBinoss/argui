//! Public diagnostics for handler mutation and child input contracts.

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode, Severity};

/// Checks `source` and returns its semantic diagnostics, retaining source ranges.
fn diagnostics(source: &str) -> Vec<argui_dsl_semantic::Diagnostic> {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("app.argui", source);
    database.check().diagnostics.clone()
}

/// Invalid writes must fail before either backend receives the program.
#[test]
fn rejects_invalid_handler_assignments() {
    for (statement, code) in [
        ("count = \"bad\"", DiagnosticCode::TypeMismatch),
        ("count += true", DiagnosticCode::TypeMismatch),
        ("count /= 1.5", DiagnosticCode::TypeMismatch),
        ("input = 1", DiagnosticCode::ReadOnlyProperty),
        ("(count + 1) = 2", DiagnosticCode::ReadOnlyProperty),
        ("return 1", DiagnosticCode::TypeMismatch),
    ] {
        let source = format!(
            "import {{ TouchArea }} from \"@argui/native\"\nexport component App {{
                in property input: int = 0
                private property count: int = 0
                TouchArea {{ on moved {{ {statement} }} }}
            }}"
        );
        let errors = diagnostics(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == code && error.severity == Severity::Error),
            "{statement} should fail with {code:?}: {errors:?}"
        );
    }
}

/// Only public input properties accept values supplied by a parent.
#[test]
fn rejects_private_and_output_child_inputs() {
    for direction in ["private", "out"] {
        let source = format!(
            "component Child {{ {direction} property value: int = 0 }}
             export component App {{ Child {{ value: 1 }} }}"
        );
        assert!(
            diagnostics(&source)
                .iter()
                .any(|error| error.code == DiagnosticCode::ReadOnlyProperty
                    && error.severity == Severity::Error)
        );
    }
}

/// An omitted key or a key of an unsupported type is a source error.
#[test]
fn requires_integer_or_string_repeater_keys() {
    for (key, code) in [
        ("", DiagnosticCode::MissingRepeaterKey),
        ("key true", DiagnosticCode::TypeMismatch),
        ("key missing", DiagnosticCode::UnknownName),
    ] {
        let source = format!(
            "import {{ Text }} from \"@argui/native\"\nexport component App {{
                for item in [1, 2] {key} {{ Text {{ content: str(item) }} }}
            }}"
        );
        let errors = diagnostics(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == code && error.severity == Severity::Error),
            "{key}: {errors:?}"
        );
    }
}

/// A binding expression containing a writable property is not a writable target.
#[test]
fn rejects_computed_two_way_targets() {
    let errors = diagnostics(
        "import { TextInput } from \"@argui/native\"\nexport component App {
            private property value: string = \"\"
            TextInput { value <=> value + \"suffix\" }
        }",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::InvalidTwoWayBinding)
    );
}

/// Legal property writes retain their existing behavior.
#[test]
fn accepts_writable_properties_and_compound_operations() {
    let errors = diagnostics(
        "import { TouchArea } from \"@argui/native\"\nexport component App {
            private property count: int = 0
            out property result: float = 0.0
            in-out property text: string = \"\"
            TouchArea { on moved { count += 1; result = count; text += \"a\" } }
        }",
    );
    assert!(errors.is_empty(), "{errors:?}");
}

/// Input-only children, computed aliases, and widening aliases cannot publish safely.
#[test]
fn rejects_two_way_bindings_without_matching_bidirectional_contracts() {
    for source in [
        "component Child { in property value: int = 0 } export component App { private property value: int = 0 Child { value <=> value } }",
        "component Child { in-out property value: float = 0.0 } export component App { private property value: int = 0 Child { value <=> value } }",
        "import { Rectangle } from \"@argui/native\" export component App { private property value: float = 0.0 Rectangle { opacity <=> value } }",
    ] {
        let issues = diagnostics(source);
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == DiagnosticCode::InvalidTwoWayBinding),
            "{issues:?}"
        );
    }
}

/// Value-returning callbacks require a return, while native void handlers may exit early.
#[test]
fn handler_returns_obey_callback_result_types() {
    for body in ["", "return true", "return;"] {
        let source = format!(
            "component Child {{ callback compute() -> int }} export component App {{ Child {{ on compute {{ {body} }} }} }}"
        );
        assert!(
            diagnostics(&source)
                .iter()
                .any(|issue| issue.code == DiagnosticCode::TypeMismatch)
        );
    }
    let source = "import { TouchArea } from \"@argui/native\" component Child { callback compute() -> float } export component App { Child { on compute { return 1 } } TouchArea { on click { return; } } }";
    assert!(diagnostics(source).is_empty(), "{:?}", diagnostics(source));
}

/// Bindings cannot invoke host callbacks whose side effects would depend on rendering.
#[test]
fn effectful_callbacks_are_restricted_to_handlers() {
    let source = "export component App { callback compute() -> int private property value: int = compute() }";
    assert!(
        diagnostics(source)
            .iter()
            .any(|issue| issue.message.contains("only be invoked in a handler"))
    );
}
