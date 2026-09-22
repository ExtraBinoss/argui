use argui_dsl_compiler::{Compiler, CompilerError, CompilerSession, SourceModule};

#[test]
fn multi_file_aot_performs_transitive_component_and_asset_dce() {
    let modules = [
        SourceModule::new(
            "ui/card.argui",
            r#"import { Text } from "@argui/ui"
export component Card {
    in property label: string
    Text { content: label }
}
export component Dead {
    private property image: asset = asset("../assets/dead.png")
    Text { content: "Dead" }
}
"#,
        ),
        SourceModule::new(
            "ui/main.argui",
            r#"import { Column } from "@argui/ui"
import { Card } from "./card.argui"
export component Main {
    in property title: string
    Column { Card { label: title } }
}
"#,
        ),
    ];
    let compiled = Compiler::compile(modules, "ui/main.argui", |_| {
        Err("unreachable asset should not be loaded".into())
    })
    .unwrap();
    assert_eq!(compiled.reachability.components.len(), 2);
    assert!(compiled.reachability.assets.is_empty());
    assert!(!compiled.rust.contains("dead.png"));
    assert!(compiled.rust.contains("pub struct Main"));
}

#[test]
fn public_api_hash_ignores_visual_edits_and_changes_with_abi() {
    let compile = |content: &str, extra_api: &str| {
        Compiler::compile(
            [SourceModule::new(
                "ui/main.argui",
                format!(
                    "import {{ Text }} from \"@argui/ui\"\nexport component Main {{ in property title: string {extra_api} Text {{ content: \"{content}\" }} }}"
                ),
            )],
            "ui/main.argui",
            |_| Err("no assets".into()),
        )
        .unwrap()
    };
    let first = compile("A", "");
    let visual_edit = compile("B", "");
    let abi_edit = compile("A", "callback close()");
    assert_eq!(first.public_api_hash, visual_edit.public_api_hash);
    assert_ne!(first.public_api_hash, abi_edit.public_api_hash);
}

#[test]
fn public_api_hash_excludes_private_component_state() {
    let compile = |private: &str| {
        Compiler::compile(
            [SourceModule::new(
                "ui/main.argui",
                format!(
                    "import {{ Text }} from \"@argui/ui\"\nexport component Main {{ in property title: string private property internal: {private} Text {{ content: title }} }}"
                ),
            )],
            "ui/main.argui",
            |_| Err("no assets".into()),
        )
        .unwrap()
    };
    assert_eq!(
        compile("string = \"first\"").public_api_hash,
        compile("int = 42").public_api_hash
    );
}

#[test]
fn reachable_wgsl_is_validated_before_rust_emission() {
    let source = SourceModule::new(
        "ui/main.argui",
        r#"export effect Broken { shader: "../shaders/broken.wgsl" }
export component Main {}
"#,
    );
    let result = Compiler::compile([source], "ui/main.argui", |path| {
        assert_eq!(path, "shaders/broken.wgsl");
        Ok(b"fn argui_effect( -> vec4<f32> {".to_vec())
    });
    let Err(error) = result else {
        panic!("broken WGSL should reject the generation");
    };
    assert!(matches!(error, CompilerError::Asset { .. }));
}

#[test]
fn incremental_session_reparses_only_the_changed_module() {
    let mut session = CompilerSession::new("ui/main.argui").unwrap();
    session.update_module(SourceModule::new(
        "ui/card.argui",
        r#"import { Text } from "@argui/ui"
export component Card { Text { content: "A" } }"#,
    ));
    session.update_module(SourceModule::new(
        "ui/main.argui",
        r#"import { Card } from "./card.argui"
export component Main { Card {} }"#,
    ));
    session.compile(|_| Err("no assets".into())).unwrap();
    let initial = session.stats();
    assert!(initial.parse_executions >= 3);

    session.update_module(SourceModule::new(
        "ui/card.argui",
        r#"import { Text } from "@argui/ui"
export component Card { Text { content: "B" } }"#,
    ));
    session.compile(|_| Err("no assets".into())).unwrap();
    let updated = session.stats();
    assert_eq!(updated.parse_executions, initial.parse_executions + 1);
    assert_eq!(updated.semantic_executions, initial.semantic_executions + 1);
}

#[test]
fn zero_argument_components_implement_default_but_required_inputs_do_not() {
    let compile = |property: &str| {
        Compiler::compile(
            [SourceModule::new(
                "ui/main.argui",
                format!("export component Main {{ {property} }}"),
            )],
            "ui/main.argui",
            |_| Err("no assets".into()),
        )
        .unwrap()
        .rust
    };

    assert!(
        compile("private property count: int = 0")
            .contains("impl Default for Main { fn default() -> Self { Self::new() } }")
    );
    assert!(!compile("in property title: string").contains("impl Default for Main"));
}

#[test]
fn rich_aot_source_emits_types_tokens_effects_bindings_and_control_flow() {
    let source = r#"import { Button, Column, Input, Text } from "@argui/ui"
export struct Item { id: int label: string }
export enum Mode { idle active }
export enum Empty { }
struct PrivateData { ignored: string }
enum PrivateMode { only }
export theme Palette {
    --accent: color = #369
    --space: float = 8.0
    dark { --accent: #ffffff }
}
export style Spaced for Row { gap: 8.0 hover { gap: 12.0 } }
export effect Glow {
    shader: "effects/glow.wgsl"
    parameter amount: float = 0.5
    parameter count: int = 1
    parameter enabled: bool = true
    parameter tint: color = #ffffff
    parameter padding: length = 2px
}
theme UnusedTheme { --unused: color = #000000 }
effect UnusedEffect { shader: "effects/unused.wgsl" }
export component Child {
    in property item: Item
    callback pressed(value: string)
    slot content
    Text { content: item.label }
}
export component OptionalChild {
    private property value: int
    slot first
    slot second
    Text { content: "optional" }
}
export component Main {
    in property title: string
    in property item: Item
    in property items: model<Item>
    in-out property query: string = ""
    private property flag: bool
    private property count: int
    private property ratio: float
    private property positive: int = +1
    private property name: string
    private property family: font-family
    private property weight: font-weight
    private property size: font-size
    private property line: line-height
    private property percentage: percentage
    private property duration: duration
    private property angle: angle
    private property color: color
    private property brush: brush
    private property dimension: dimension
    private property radii: radii
    private property insets: insets
    private property border: border
    private property shadow: shadow
    private property transform: transform
    private property asset_value: asset
    private property values: array<int>
    private property models: model<Item>
    private property optional: optional<string>
    callback activate()
    callback notify(value: int) -> string
    slot content
    Column #root {
        gap: var(--space)
        padding: 2.0
        width: 100px
        height: 50px
        background: #123
        Text { content: tr("title") }
        Text { content: item.label }
        Input { value <=> query on submit { count = 1 count += 1 count -= 1 count *= 2 count /= 1 return activate() } }
        Button { text: title on click { activate() } }
        Child { item: item on pressed { notify(1) } }
        for entry in items key entry.id {
            Text { content: entry.label }
            Child { item: entry on pressed { notify(entry.id) } }
            Child { item: entry }
            OptionalChild { Text { content: "slot" } }
        }
        for named in items key named.label { Text { content: named.label } }
        if flag { Text { content: "yes" } } else { Text { content: "no" } }
        if flag { Text { content: "again" } }
        content
    }
}
"#;
    let valid_shader = "fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source; }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |path| {
            assert_eq!(path, "ui/effects/glow.wgsl");
            Ok(valid_shader.as_bytes().to_vec())
        },
    )
    .unwrap();
    assert_eq!(compiled.roots.len(), 3);
    assert!(compiled.reachability.components.len() >= 3);
    assert!(!compiled.reachability.structs.is_empty());
    assert!(!compiled.reachability.enums.is_empty());
    assert!(!compiled.reachability.themes.is_empty());
    assert!(!compiled.reachability.styles.is_empty());
    assert!(!compiled.reachability.effects.is_empty());
    assert!(
        compiled
            .dependencies
            .iter()
            .any(|path| path.ends_with("glow.wgsl"))
    );
    assert!(compiled.rust.contains("pub struct Item"));
    assert!(compiled.rust.contains("pub enum Mode"));
    assert!(compiled.rust.contains("fn theme_token_"));
    assert!(compiled.rust.contains("effect_definitions()"));
    assert!(compiled.rust.contains("with_signed_key"));
    assert!(compiled.rust.contains("TextEdited(edit)"));
    assert!(compiled.rust.contains("set_translator"));
}

#[test]
fn compiler_errors_display_each_public_failure_kind() {
    let Err(missing) = Compiler::compile([], "ui/missing.argui", |_| Ok(Vec::new())) else {
        panic!("missing entry should fail");
    };
    assert!(matches!(&missing, CompilerError::MissingEntry(path) if path == "ui/missing.argui"));
    assert!(missing.to_string().contains("entry module"));

    let Err(semantic) = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "export component Main { Missing {} }",
        )],
        "ui/main.argui",
        |_| Ok(Vec::new()),
    ) else {
        panic!("invalid source should fail");
    };
    assert!(matches!(&semantic, CompilerError::Semantic(diagnostics) if !diagnostics.is_empty()));
    assert!(semantic.to_string().contains("DSL diagnostic(s)"));

    let Err(asset) = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "export effect Glow { shader: \"shaders/glow.wgsl\" } export component Main {}",
        )],
        "ui/main.argui",
        |_| Err("asset unavailable".into()),
    ) else {
        panic!("asset loader failure should fail");
    };
    assert!(matches!(&asset, CompilerError::Asset { .. }));
    assert!(asset.to_string().contains("asset `ui/shaders/glow.wgsl`"));

    let Err(utf8) = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "export effect Glow { shader: \"shaders/glow.wgsl\" } export component Main {}",
        )],
        "ui/main.argui",
        |_| Ok(vec![0xff]),
    ) else {
        panic!("invalid UTF-8 should fail");
    };
    assert!(matches!(&utf8, CompilerError::Asset { .. }));
    assert!(utf8.to_string().contains("asset `ui/shaders/glow.wgsl`"));

    let lower = CompilerError::Lower(Vec::new());
    assert_eq!(lower.to_string(), "0 IR lowering error(s)");
    let codegen = CompilerError::Codegen("bad generated shape".into());
    assert!(codegen.to_string().contains("bad generated shape"));
    let schema = CompilerError::from(argui_schema::SchemaError::Adapter("bad schema".into()));
    assert!(schema.to_string().contains("native schema"));
}
