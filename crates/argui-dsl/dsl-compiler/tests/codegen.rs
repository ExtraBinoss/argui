use argui_dsl_compiler::{Compiler, CompilerError, Reachability, SourceModule};

#[path = "codegen/node/virtual_list.rs"]
mod virtual_list;

#[path = "codegen/node/motion.rs"]
mod motion;

#[path = "codegen/node.rs"]
mod retained_children;

#[path = "codegen/expression.rs"]
mod expressions;

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

/// String built-ins and concatenation produce valid typed AOT expressions.
#[test]
fn compiler_emits_string_conversion_contains_and_concatenation() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "import { Text } from \"@argui/native\" export component Main { private property count: int = 0 Text { content: contains(\"abc\", \"b\") ? \"Count: \" + str(count) : \"missing\" } }",
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains(".contains("));
    assert!(compiled.rust.contains(".to_string()"));
    assert!(compiled.rust.contains("text.push_str("));
}

/// Rust-reserved field names remain legal in generated structs.
#[test]
fn compiler_escapes_rust_keyword_fields() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
export struct raw_record { type: string }
export component Main {
    in property record: raw_record
    Text { content: record.type }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("type_"));
    assert!(compiled.rust.contains("struct RawRecord"));
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
        r#"import { TextInput } from "@argui/native"
export component Main {
    private property query: string = ""
    TextInput { value <=> query + "" }
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

/// Token reads retain the owning theme even when it lives in another module.
#[test]
fn external_theme_tokens_are_reachable_from_a_component() {
    let compiled = Compiler::compile(
        [
            SourceModule::new(
                "ui/main.argui",
                "import { Text } from \"@argui/ui\"\nexport component Main { Text { color: var(--external) content: \"hello\" } }",
            ),
            SourceModule::new(
                "ui/theme.argui",
                "export theme Palette { --external: color = #336699 }",
            ),
        ],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert_eq!(compiled.reachability.themes.len(), 1);
    assert_eq!(compiled.reachability.tokens.len(), 1);
    assert!(compiled.rust.contains("fn theme_token_"));
}

/// Animation stops retain theme dependencies even when no normal binding uses them.
#[test]
fn keyframe_only_token_reads_retain_their_theme() {
    let compiled = Compiler::compile(
        [
            SourceModule::new(
                "ui/main.argui",
                "import { Container } from \"@argui/native\"\nexport component Main { Container { rotation: 0.0 animate rotation { duration: 100ms keyframes { 0%: var(--start) 100%: var(--end) } } } }",
            ),
            SourceModule::new(
                "ui/theme.argui",
                "export theme Motion { --start: float = 0.0 --end: float = 90.0 }",
            ),
        ],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert_eq!(compiled.reachability.themes.len(), 1);
    assert_eq!(compiled.reachability.tokens.len(), 2);
    let mut pruned = compiled.ir;
    compiled.reachability.prune(&mut pruned);
    assert_eq!(pruned.themes.len(), 1);
}

/// State overrides retain external tokens even when their base binding is constant.
#[test]
fn state_only_token_reads_retain_their_theme() {
    let compiled = Compiler::compile(
        [
            SourceModule::new(
                "ui/main.argui",
                "import { Container } from \"@argui/native\"\nexport component Main { in property expanded: bool = false Container { rotation: 0.0 states { open when expanded { rotation: var(--target) } } } }",
            ),
            SourceModule::new(
                "ui/theme.argui",
                "export theme Motion { --target: float = 90.0 }",
            ),
        ],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert_eq!(compiled.reachability.themes.len(), 1);
    assert_eq!(compiled.reachability.tokens.len(), 1);
}

#[test]
fn reachability_can_resolve_a_theme_through_a_mode_override_edge() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/native"
theme Palette {
    --accent: color = #112233
    dark { --accent: #ffffff }
}
export component Main { Text { content: "hello" color: var(--accent) } }"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let mut ir = compiled.ir.clone();
    let palette_id = compiled
        .semantic
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == "Palette")
        .expect("Palette definition")
        .id
        .raw();
    let palette = ir
        .themes
        .iter_mut()
        .find(|theme| theme.id.raw() == palette_id)
        .expect("lowered Palette theme");
    assert_eq!(palette.tokens.len(), 1);
    assert_eq!(palette.modes.len(), 1);
    let accent_id = palette.tokens[0].id;
    // Model an incomplete token table while preserving the mode's token edge.
    palette.tokens.clear();
    let reachability = Reachability::analyze(&compiled.semantic, &ir, "ui/main.argui");
    assert!(reachability.themes.iter().any(|id| id.raw() == palette_id));
    assert!(reachability.tokens.contains(&accent_id));
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
style DeadStyle for Text { content: "dead" }
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

#[test]
fn asset_codegen_handles_a_project_without_reachable_media() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "import { Container } from \"@argui/native\"\nexport component Main { Container {} }",
        )],
        "ui/main.argui",
        |_| Err("no media expected".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("static MEDIA_ASSETS: ::argui::schema::AssetRegistry = ::argui::schema::AssetRegistry::new();"));
    assert!(
        compiled
            .rust
            .contains("fn asset_handle(id: u64) -> ::argui::schema::AssetHandle { panic!")
    );
    assert!(!compiled.rust.contains("let mut assets ="));
    assert!(!compiled.rust.contains("let path = match id"));
}

#[path = "codegen/child_reference.rs"]
mod child_reference;
