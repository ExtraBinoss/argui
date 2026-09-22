use std::{collections::HashMap, sync::Arc};

use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_ir::{
    AssetId, AssetKind, EffectId, IrAsset, IrEffect, IrEffectParameter, IrProject, IrType,
    PropertyId, SourceInfo,
};
use argui_dsl_protocol::{
    AssetBytes, LiveMessage, LivePackageEnvelope, PackageHeader, RuntimeVersions,
};
use argui_dsl_runtime::{AssetPayload, LivePackage, LiveRuntime, RuntimeError};
use argui_dsl_syntax::{FileId, Span, TextRange, TextSize};

fn empty_project() -> IrProject {
    IrProject {
        modules: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        components: Vec::new(),
        themes: Vec::new(),
        styles: Vec::new(),
        effects: Vec::new(),
        assets: Vec::new(),
    }
}

fn source() -> SourceInfo {
    SourceInfo::new(
        Span::new(FileId::from_raw(1), TextRange::empty(TextSize::from(0))),
        None,
        None,
    )
}

#[test]
fn package_prepare_clamps_generation_and_exposes_precompiled_lookup() {
    let package = LivePackage::prepare(0, 44, empty_project(), HashMap::new()).unwrap();
    assert_eq!(package.generation, 1);
    assert_eq!(package.public_api_hash, 44);
    assert!(package.roots.is_empty());
    assert!(package.programs.is_empty());
    assert!(
        package
            .program(argui_dsl_ir::ExpressionId::from_raw(1))
            .is_none()
    );
    let payload = AssetPayload::new(0, vec![1_u8, 2, 3]);
    assert_eq!(payload.revision, 1);
    assert_eq!(&*payload.bytes, &[1, 2, 3]);
}

#[test]
fn package_rejects_missing_and_invalid_assets_before_runtime_commit() {
    let asset = IrAsset {
        id: AssetId::from_raw(9),
        path: "images/icon.png".into(),
        kind: AssetKind::Image,
        inline_bytes: None,
    };
    let mut project = empty_project();
    project.assets.push(asset.clone());
    assert!(matches!(
        LivePackage::prepare(2, 0, project.clone(), HashMap::new()),
        Err(RuntimeError::MissingAsset(9))
    ));
    let mut effect_project = empty_project();
    effect_project.effects.push(IrEffect {
        id: EffectId::from_raw(11),
        shader: AssetId::from_raw(11),
        bounded_damage: false,
        parameters: Vec::new(),
        source: source(),
    });
    assert!(matches!(
        LivePackage::prepare(2, 0, effect_project, HashMap::new()),
        Err(RuntimeError::MissingAsset(11))
    ));
    let mut assets = HashMap::new();
    assets.insert(
        AssetId::from_raw(9),
        AssetPayload::new(1, vec![0xff, 0xd8, 0xff]),
    );
    let package = LivePackage::prepare(2, 0, project, assets).unwrap();
    assert_eq!(package.assets.len(), 1);
    assert!(package.shader_hashes.is_empty());

    let mut project = empty_project();
    project.assets.push(IrAsset {
        id: AssetId::from_raw(10),
        path: "/bad path.png".into(),
        kind: AssetKind::Image,
        inline_bytes: None,
    });
    let mut assets = HashMap::new();
    assets.insert(AssetId::from_raw(10), AssetPayload::new(1, vec![1]));
    let package = LivePackage::prepare(1, 0, project, assets).unwrap();
    assert!(matches!(
        LiveRuntime::new(package),
        Err(RuntimeError::Asset(_))
    ));
}

#[test]
fn package_validates_shader_parameter_word_widths_and_source() {
    let shader = AssetId::from_raw(20);
    let mut project = empty_project();
    project.assets.push(IrAsset {
        id: shader,
        path: "effects/glow.wgsl".into(),
        kind: AssetKind::Shader,
        inline_bytes: None,
    });
    project.effects.push(IrEffect {
        id: EffectId::from_raw(21),
        shader,
        bounded_damage: false,
        parameters: vec![
            IrEffectParameter {
                id: PropertyId::from_raw(1),
                name: "tint".into(),
                value_type: IrType::Color,
                default: None,
                source: source(),
            },
            IrEffectParameter {
                id: PropertyId::from_raw(2),
                name: "matrix".into(),
                value_type: IrType::Transform,
                default: None,
                source: source(),
            },
            IrEffectParameter {
                id: PropertyId::from_raw(3),
                name: "amount".into(),
                value_type: IrType::Float,
                default: None,
                source: source(),
            },
        ],
        source: source(),
    });
    let shader_source = br#"
fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> {
    return source * argui_param_f32(0u);
}
"#;
    let mut assets = HashMap::new();
    assets.insert(
        shader,
        AssetPayload::new(1, Arc::<[u8]>::from(shader_source.as_slice())),
    );
    let package = LivePackage::prepare(1, 99, project.clone(), assets).unwrap();
    assert!(package.shader_hashes.contains_key(&EffectId::from_raw(21)));

    let mut invalid_source_assets = HashMap::new();
    invalid_source_assets.insert(shader, AssetPayload::new(1, b"not wgsl".to_vec()));
    assert!(matches!(
        LivePackage::prepare(2, 99, project.clone(), invalid_source_assets),
        Err(RuntimeError::InvalidShader(_))
    ));
    let mut bad_assets = HashMap::new();
    bad_assets.insert(shader, AssetPayload::new(1, vec![0xff, 0xfe]));
    assert!(matches!(
        LivePackage::prepare(2, 99, project, bad_assets),
        Err(RuntimeError::InvalidShader(_))
    ));
}

#[test]
fn envelope_checks_versions_duplicates_and_assigns_roots() {
    let header = PackageHeader::current(77, 0);
    let root = argui_dsl_ir::ComponentId::from_raw(8);
    let envelope = LivePackageEnvelope {
        header: header.clone(),
        roots: vec![root],
        ir: empty_project(),
        assets: Vec::new(),
    };
    let package = LivePackage::from_envelope(envelope).unwrap();
    assert_eq!(package.generation, 1);
    assert_eq!(package.roots, vec![root]);

    let mut wrong = header.clone();
    wrong.protocol_version += 1;
    assert!(matches!(
        LivePackage::from_envelope(LivePackageEnvelope {
            header: wrong,
            roots: Vec::new(),
            ir: empty_project(),
            assets: Vec::new(),
        }),
        Err(RuntimeError::IncompatiblePackage(_))
    ));

    let duplicate = LivePackageEnvelope {
        header,
        roots: Vec::new(),
        ir: empty_project(),
        assets: vec![
            AssetBytes {
                id: AssetId::from_raw(1),
                revision: 1,
                bytes: vec![1],
            },
            AssetBytes {
                id: AssetId::from_raw(1),
                revision: 2,
                bytes: vec![2],
            },
        ],
    };
    assert!(matches!(
        LivePackage::from_envelope(duplicate),
        Err(RuntimeError::IncompatiblePackage(message)) if message.contains("more than once")
    ));

    let hello = LiveMessage::Hello {
        protocol_version: RuntimeVersions::current().protocol,
        ir_format_version: RuntimeVersions::current().ir,
        engine_version: RuntimeVersions::current().engine,
    };
    assert!(matches!(hello, LiveMessage::Hello { .. }));
}

/// Compiles a project containing every package expression container.
fn rich_project() -> argui_dsl_compiler::CompiledProject {
    let source = r#"import { Button, Column, Input, Row, Text } from "@argui/ui"
export struct Item { id: int label: string }
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
    parameter tint: color = #ffffff
}
export component Main {
    in-out property title: string = "title"
    in property items: model<Item>
    private property flag: bool = true
    private property count: int = 0
    callback activate()
    slot content
    Column #root {
        gap: var(--space)
        Text { content: title + "!" }
        Input { value <=> title on submit { count = 1 count += 1 return activate() } }
        Button { text: title on click { activate() } }
        for entry in items key entry.id {
            Text { content: entry.label }
        }
        if flag { Text { content: "yes" } } else { Text { content: "no" } }
        content
    }
    states { compact when flag { count: 2 } }
    animate count { duration: 10ms }
}
"#;
    let shader = b"fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source; }";
    Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Ok(shader.to_vec()),
    )
    .expect("rich package fixture should compile")
}

#[test]
fn package_precompiles_expressions_from_components_themes_styles_and_effects() {
    let compiled = rich_project();
    assert!(
        compiled
            .ir
            .components
            .iter()
            .any(|component| { !component.states.is_empty() && !component.animations.is_empty() })
    );
    assert!(
        compiled
            .ir
            .themes
            .iter()
            .any(|theme| !theme.modes.is_empty())
    );
    assert!(!compiled.ir.styles[0].states.is_empty());
    assert!(!compiled.ir.effects[0].parameters.is_empty());

    let mut assets = HashMap::new();
    for asset in &compiled.ir.assets {
        assets.insert(
            asset.id,
            AssetPayload::new(
                4,
                b"fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source; }".to_vec(),
            ),
        );
    }
    let component = compiled
        .ir
        .components
        .iter()
        .find(|component| !component.states.is_empty())
        .expect("fixture should contain a stateful component");
    let property_default = component
        .properties
        .iter()
        .find_map(|property| property.default.as_ref())
        .expect("fixture should contain a property default")
        .id;
    let state_condition = component.states[0].condition.id;
    let animation_value = component.animations[0].parameters[0].value.id;
    let theme = compiled
        .ir
        .themes
        .iter()
        .find(|theme| !theme.modes.is_empty())
        .expect("fixture should contain a theme mode");
    let theme_default = theme.tokens[0].default.id;
    let theme_override = theme.modes[0].overrides[0].1.id;
    let style_value = compiled.ir.styles[0].properties[0].value.id;
    let style_state_value = compiled.ir.styles[0].states[0].properties[0].value.id;
    let effect_default = compiled.ir.effects[0].parameters[0]
        .default
        .as_ref()
        .expect("fixture should contain an effect default")
        .id;

    let package = LivePackage::prepare(4, compiled.public_api_hash, compiled.ir, assets).unwrap();
    for id in [
        property_default,
        state_condition,
        animation_value,
        theme_default,
        theme_override,
        style_value,
        style_state_value,
        effect_default,
    ] {
        assert!(
            package.program(id).is_some(),
            "expression {id:?} was not compiled"
        );
    }
    assert!(package.programs.len() >= 20);
    assert_eq!(package.shader_hashes.len(), 1);
}

/// A live shader keeps its registry revision on an identical reload and
/// increments it when validated shader bytes change.
#[test]
fn effect_definitions_track_shader_revisions_across_reload() {
    let shader = AssetId::from_raw(71);
    let effect = EffectId::from_raw(72);
    let mut project = empty_project();
    project.assets.push(IrAsset {
        id: shader,
        path: "effects/glow.wgsl".into(),
        kind: AssetKind::Shader,
        inline_bytes: None,
    });
    project.effects.push(IrEffect {
        id: effect,
        shader,
        bounded_damage: false,
        parameters: vec![IrEffectParameter {
            id: PropertyId::from_raw(73),
            name: "amount".into(),
            value_type: IrType::Float,
            default: None,
            source: source(),
        }],
        source: source(),
    });
    let compile = |generation, extra: &str| {
        let shader_source = format!(
            "fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> {{ return source * argui_param_f32(0u); }}{extra}"
        );
        LivePackage::prepare(
            generation,
            99,
            project.clone(),
            HashMap::from([(
                shader,
                AssetPayload::new(generation, shader_source.into_bytes()),
            )]),
        )
        .unwrap()
    };
    let mut runtime = LiveRuntime::new(compile(1, "")).unwrap();
    let definitions = runtime.effect_definitions();
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].revision, 1);
    assert_eq!(definitions[0].parameters[0].name.as_str(), "amount");

    let same = runtime.prepare_reload(compile(2, "")).unwrap();
    let _ = runtime.commit_reload(same);
    assert_eq!(runtime.effect_definitions()[0].revision, 1);

    let changed = runtime.prepare_reload(compile(3, "\n")).unwrap();
    let _ = runtime.commit_reload(changed);
    assert_eq!(runtime.effect_definitions()[0].revision, 2);
}
