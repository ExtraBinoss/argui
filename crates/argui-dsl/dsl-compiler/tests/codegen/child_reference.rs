//! Diagnostics for public output reads on identified child components.

use argui_dsl_compiler::{Compiler, CompilerError, SourceModule};

/// A later identified child output is initialized before an earlier sibling reads it.
#[test]
fn sibling_child_output_read_uses_a_retained_handle() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
component Child {
    out property title: string = "ready"
    Text { content: title }
}
export component Main {
    Text { content: child.title }
    Child #child {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("child_ref_identity_"));
    assert!(compiled.rust.contains("child_properties.defaulted("));
    assert!(compiled.rust.contains("_child_ref_p_"));
}

/// Nested native containers preserve child outputs for earlier sibling bindings.
#[test]
fn nested_child_output_read_uses_the_same_handle() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Container, Text } from "@argui/native"
component Child {
    in-out property title: string = "ready"
    Text { content: title }
}
export component Main {
    private property caption: string = "updated"
    Container {
        Text { content: child.title }
        Child #child { title <=> caption }
    }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("child_ref_identity_"));
    assert!(compiled.rust.contains("_child_ref_p_"));
    assert!(compiled.rust.contains(".clone()"));
}

/// Controlled child input and a child output coexist without replacing each other.
#[test]
fn child_output_read_preserves_controlled_inputs() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
component Child {
    in property prefix: string = "default"
    out property title: string = prefix + " title"
    Text { content: title }
}
export component Main {
    Text { content: child.title }
    Child #child { prefix: "parent" }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("child_properties.controlled("));
    assert!(compiled.rust.contains("child_properties.defaulted("));
}

/// A property default that reads a child output is evaluated after handles exist.
#[test]
fn child_output_in_parent_default_is_render_contextual() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
component Child {
    out property title: string = "ready"
    Text { content: title }
}
export component Main {
    private property caption: string = true ? child.title : "fallback"
    Text { content: caption }
    Child #child {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("child_ref_identity_"));
    assert!(compiled.rust.contains("_child_ref_p_"));
    assert!(compiled.rust.contains("fallback"));
}

/// A two-way child output cannot alias a computed expression in the parent.
#[test]
fn child_output_rejects_computed_two_way_source() {
    let result = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
component Child {
    in-out property title: string = "ready"
    Text { content: title }
}
export component Main {
    private property caption: string = "parent"
    Text { content: child.title }
    Child #child { title <=> caption + "" }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    );
    let Err(CompilerError::Codegen(message)) = result else {
        panic!("computed two-way source should fail code generation");
    };
    assert!(message.contains("two-way child reference source is not a property"));
}

/// An output without an authored default still gets a stable fallback handle.
#[test]
fn child_output_without_default_is_available_to_binary_parent_defaults() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
component Child {
    in-out property title: string
    Text { content: title }
}
export component Main {
    private property caption: string = child.title + "!"
    private property prefixed: string = "prefix" + child.title
    private property fallback: string = false ? "unused" : child.title
    Text { content: caption + prefixed + fallback }
    Child #child {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("child_properties.defaulted("));
    assert!(compiled.rust.contains("String::new()"));
    assert!(compiled.rust.contains("text.push_str("));
    assert!(compiled.rust.contains("unused"));
}

/// Private child state cannot escape through an identified visual reference.
#[test]
fn private_child_property_read_is_rejected() {
    let result = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
component Child {
    private property secret: string = "hidden"
    Text { content: secret }
}
export component Main {
    Text { content: child.secret }
    Child #child {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    );
    let Err(CompilerError::Semantic(diagnostics)) = result else {
        panic!("private child output read must fail semantic validation");
    };
    assert!(
        diagnostics
            .iter()
            .any(|item| { item.message.contains("secret") && item.message.contains("readable") })
    );
}

/// Child inputs remain writable at the call site but are not public outputs.
#[test]
fn input_only_child_property_read_is_rejected() {
    let result = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
component Child {
    in property title: string = "input"
    Text { content: title }
}
export component Main {
    Text { content: child.title }
    Child #child {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    );
    let Err(CompilerError::Semantic(diagnostics)) = result else {
        panic!("input-only child property read must fail semantic validation");
    };
    assert!(
        diagnostics
            .iter()
            .any(|item| { item.message.contains("title") && item.message.contains("readable") })
    );
}
