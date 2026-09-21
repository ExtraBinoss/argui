use argui_dsl_compiler::{Compiler, CompilerError, Reachability, SourceModule};

/// Compiles a source module and returns the AOT code-generation failure.
fn codegen_error(source: &str) -> CompilerError {
    let result = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    );
    match result {
        Err(error @ CompilerError::Codegen(_)) => error,
        Err(error) => panic!("expected code-generation error, received {error}"),
        Ok(_) => panic!("source unexpectedly compiled successfully"),
    }
}

/// Rejects a native element whose children cannot be represented by a slot.
#[test]
fn compiler_rejects_children_for_slotless_native_elements() {
    let error = codegen_error(
        r#"import { Text } from "@argui/ui"
export component Main {
    Text { content: "outer" Text { content: "inner" } }
}"#,
    );
    assert!(
        error
            .to_string()
            .contains("does not accept visual children")
    );
}

/// Rejects a native two-way binding whose expression is not a property read.
#[test]
fn compiler_rejects_non_property_native_two_way_sources() {
    let error = codegen_error(
        r#"import { TextEditor } from "@argui/native"
export component Main {
    private property query: string = ""
    TextEditor { value <=> query + "" }
}"#,
    );
    let message = error.to_string();
    assert!(
        message.contains("native two-way source is not a property"),
        "unexpected codegen error: {message}"
    );
}

/// Rejects a component two-way binding whose expression is not a property read.
#[test]
fn compiler_rejects_non_property_component_two_way_sources() {
    let error = codegen_error(
        r#"import { Text } from "@argui/ui"
export component Child {
    in-out property value: string
    Text { content: value }
}
export component Main {
    private property query: string = ""
    Child { value <=> query + "" }
}"#,
    );
    assert!(
        error
            .to_string()
            .contains("two-way binding did not lower to a property ID")
    );
}

/// Rejects event assignments to a repeater local instead of generating an invalid closure.
#[test]
fn compiler_rejects_repeater_local_assignments_in_events() {
    let error = codegen_error(
        r#"import { Button, Column } from "@argui/ui"
export struct Item { id: int }
export component Main {
    in property items: model<Item>
    Column {
        for item in items key item.id {
            Button { text: "edit" on click { item = item } }
        }
    }
}"#,
    );
    assert!(
        error
            .to_string()
            .contains("mutable event locals are not part of the restricted handler ABI")
    );
}

/// Keeps reachability analysis total when its entry module or IR component is absent.
#[test]
fn reachability_handles_missing_entry_and_component_ids() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "export component Main {}",
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();

    let missing_entry = Reachability::analyze(&compiled.semantic, &compiled.ir, "ui/missing.argui");
    assert!(missing_entry.components.is_empty());
    assert!(missing_entry.structs.is_empty());

    let mut missing_component_ir = compiled.ir.clone();
    missing_component_ir.components.clear();
    let missing_component =
        Reachability::analyze(&compiled.semantic, &missing_component_ir, "ui/main.argui");
    assert_eq!(missing_component.components.len(), 1);
}

/// Prunes every unreachable declaration while retaining the public component ABI.
#[test]
fn reachability_prune_removes_private_declarations_and_assets() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/ui"
export struct Used { value: string }
struct Dead { value: string }
export enum LiveMode { only }
enum DeadMode { only }
export theme LiveTheme { --accent: color = #123456 }
theme DeadTheme { --unused: color = #000000 }
export style LiveStyle for Text { content: "live" }
effect DeadEffect { shader: "effects/dead.wgsl" }
export effect LiveEffect { shader: "effects/live.wgsl" }
export component Main {
    in property value: Used
    Text { content: "main" }
}"#,
        )],
        "ui/main.argui",
        |path| {
            assert_eq!(path, "ui/effects/live.wgsl");
            Ok(b"fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source; }".to_vec())
        },
    )
    .unwrap();
    let mut pruned = compiled.ir.clone();
    compiled.reachability.prune(&mut pruned);

    assert_eq!(pruned.structs.len(), 1);
    assert_eq!(pruned.enums.len(), 1);
    assert_eq!(pruned.themes.len(), 1);
    assert_eq!(pruned.styles.len(), 1);
    assert_eq!(pruned.effects.len(), 1);
    assert_eq!(pruned.assets.len(), 1);
    assert!(
        pruned
            .components
            .iter()
            .all(|component| { compiled.reachability.components.contains(&component.id) })
    );
}
