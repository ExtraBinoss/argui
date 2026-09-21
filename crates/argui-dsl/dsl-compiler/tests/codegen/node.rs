//! Retained child property identities emitted by AOT node code generation.

use argui_dsl_compiler::{Compiler, SourceModule};

#[test]
fn aot_children_reuse_defaulted_properties_across_render_passes() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Pressable, Text } from "@argui/native"
component Child {
    private property activated: bool = false
    Pressable { label: "activate" on click { activated = !activated } }
    Text { content: activated ? "on" : "off" }
}
export component Main { Child {} }"#,
        )],
        "ui/main.argui",
        |_| Err("no assets in this test".into()),
    )
    .unwrap();
    let rust = &compiled.rust;
    assert!(rust.contains("child_properties.defaulted("));
    assert!(rust.contains("child_properties.begin_render()"));
    assert!(rust.contains("child_properties.end_render()"));
    assert!(rust.contains("child_owner(&child_identity_"));
}

#[test]
fn aot_children_keep_two_way_aliases_and_controlled_inputs_distinct() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
component Child {
    in-out property linked: bool = false
    in property title: string = "child"
    Text { content: title }
}
export component Main {
    private property shared: bool = false
    Child { linked <=> shared title: "parent" }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets in this test".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("child_properties.controlled("));
    assert!(!compiled.rust.contains("child_properties.defaulted("));
    assert!(compiled.rust.contains(".clone();"));
}
