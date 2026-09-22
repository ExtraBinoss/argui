//! Native observation checking and diagnostics.

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

#[test]
fn read_only_native_observation_cannot_be_assigned() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "app.argui",
        r#"import { TouchArea } from "@argui/native"
export component Main { TouchArea { pressed: true } }"#,
    );
    let checked = database.check();
    assert!(
        checked.diagnostics.iter().any(|item| {
            item.code == DiagnosticCode::ReadOnlyProperty && item.message.contains("read-only")
        }),
        "{:#?}",
        checked.diagnostics
    );
    database.set_file(
        "app.argui",
        r#"import { FocusScope } from "@argui/native"
export component Main { FocusScope #focus_area { has_focus: true } }"#,
    );
    let checked = database.check();
    assert!(
        checked
            .diagnostics
            .iter()
            .any(|item| item.code == DiagnosticCode::ReadOnlyProperty),
        "{:#?}",
        checked.diagnostics
    );
}

#[test]
fn explicitly_identified_native_outputs_are_typed_in_sibling_bindings() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "app.argui",
        r#"import { Rectangle, TouchArea } from "@argui/native"
export component Main {
    private property track_width: length = 100px
    Rectangle { opacity: touch.pressed ? 0.5 : 1.0 }
    TouchArea #touch { width: track_width }
    Rectangle { width: touch.mouse_x / track_width * 100px }
}"#,
    );
    let checked = database.check();
    assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
    database.set_file(
        "app.argui",
        r#"import { Rectangle, TouchArea } from "@argui/native"
export component Main { Rectangle { opacity: touch.missing ? 0.5 : 1.0 } TouchArea #touch {} }"#,
    );
    let checked = database.check();
    assert!(
        checked
            .diagnostics
            .iter()
            .any(|item| item.code == DiagnosticCode::UnknownProperty),
        "{:#?}",
        checked.diagnostics
    );
    database.set_file(
        "app.argui",
        r#"import { FocusScope, Rectangle } from "@argui/native"
export component Main {
    Rectangle { opacity: focus_area.missing ? 1.0 : 0.4 }
    FocusScope #focus_area {}
}"#,
    );
    let checked = database.check();
    assert!(
        checked.diagnostics.iter().any(|item| {
            item.code == DiagnosticCode::UnknownProperty
                && item
                    .message
                    .contains("`focus_area` has no readable property `missing`")
        }),
        "{:#?}",
        checked.diagnostics
    );
}

mod event {
    //! Event parameter diagnostics.

    use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

    #[test]
    fn event_parameters_use_native_payload_and_component_callback_types() {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file(
            "app.argui",
            r#"import { TextInput } from "@argui/native"
component Child { callback changed(value: string) }
export component Main {
    private property text: string = ""
    TextInput { on input(value) { text = value } }
    Child { on changed(value) { text = value } }
}"#,
        );
        let checked = database.check();
        assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
        database.set_file(
            "app.argui",
            r#"import { TextInput } from "@argui/native"
export component Main { TextInput { on input(value, extra) { value + extra } } }"#,
        );
        let checked = database.check();
        assert!(
            checked
                .diagnostics
                .iter()
                .any(|item| item.code == DiagnosticCode::TypeMismatch),
            "{:#?}",
            checked.diagnostics
        );
    }

    #[test]
    fn focus_traversal_calls_require_zero_arguments() {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file(
            "app.argui",
            r#"import { KeyBinding } from "@argui/native"
export component Main {
    KeyBinding { shortcut: "ArrowDown" on activated { focus_next() focus_previous() } }
}"#,
        );
        let checked = database.check();
        assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
        database.set_file(
            "app.argui",
            r#"import { KeyBinding } from "@argui/native"
export component Main {
    KeyBinding { shortcut: "ArrowDown" on activated { focus_next(1) focus_previous("x") } }
}"#,
        );
        let checked = database.check();
        assert_eq!(
            checked
                .diagnostics
                .iter()
                .filter(|item| item.code == DiagnosticCode::TypeMismatch
                    && item.message.contains("expects no arguments"))
                .count(),
            2,
            "{:#?}",
            checked.diagnostics
        );
    }

    /// Scroll requests require a unique native reference and two length values.
    #[test]
    fn scroll_to_checks_identity_and_coordinate_units() {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file(
            "app.argui",
            r#"import { Flickable, TouchArea } from "@argui/native"
export component Main {
    Flickable #viewport {
        TouchArea { on moved { scroll_to(#viewport, 0px, 12px) } }
    }
}"#,
        );
        let checked = database.check();
        assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
        database.set_file(
            "app.argui",
            r#"import { Flickable, TouchArea } from "@argui/native"
export component Main {
    Flickable #viewport {
        TouchArea { on moved { scroll_to("viewport", 0.0, 12px) } }
    }
}"#,
        );
        let checked = database.check();
        assert!(
            checked.diagnostics.iter().any(|item| item
                .message
                .contains("target must be a unique native #element identity")),
            "{:#?}",
            checked.diagnostics
        );
        assert!(
            checked
                .diagnostics
                .iter()
                .any(|item| item.code == DiagnosticCode::TypeMismatch
                    && item.message.contains("Length")),
            "{:#?}",
            checked.diagnostics
        );
    }
}
