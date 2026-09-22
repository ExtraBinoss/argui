//! AOT and live effect instances share typed arguments and reactive updates.

use std::collections::HashMap;

use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_runtime::{AssetPayload, DslValue, LivePackage, LiveRuntime};
use argui_paint::{EffectValue, Filter};
use argui_ui::{EffectScope, Element};

const SHADER: &[u8] = b"fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source * argui_param_f32(0u); }";

/// Reads the first validated effect's scalar parameter from a rendered visual.
///
/// `element` is the root visual. Returns its typed scalar argument.
fn amount(element: &Element) -> f32 {
    let Filter::Effect(instance) = &element.effects[0].layer.filters[0] else {
        panic!("the visual must carry a custom effect");
    };
    assert_eq!(instance.parameters[0].name.as_str(), "amount");
    let EffectValue::F32(value) = &instance.parameters[0].value else {
        panic!("the effect amount must be a float");
    };
    *value
}

/// A changed component property rebuilds the same generic effect argument.
#[test]
fn live_effect_instance_reacts_to_property_changes() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Rectangle } from "@argui/native"
export effect Glow {
    shader: "effects/glow.wgsl"
    parameter amount: float = 0.5
}

export component Main {
    in property intensity: float = 0.8
    Rectangle { effect: Glow { amount: intensity } }
}"#,
        )],
        "ui/main.argui",
        |_| Ok(SHADER.to_vec()),
    )
    .unwrap();
    assert!(compiled.rust.contains("EffectValue::F32"));
    let root = compiled.roots[0];
    let property = compiled
        .ir
        .components
        .iter()
        .find(|part| part.id == root)
        .unwrap()
        .properties[0]
        .id;
    let shader = compiled.ir.effects[0].shader;
    let package = LivePackage::prepare(
        1,
        compiled.public_api_hash,
        compiled.ir,
        HashMap::from([(shader, AssetPayload::new(1, SHADER.to_vec()))]),
    )
    .unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let instance = runtime.mount(root, []).unwrap();
    assert!((amount(&runtime.render().unwrap()) - 0.8).abs() < 0.0001);
    runtime
        .set_property(instance, property, DslValue::Float(0.2))
        .unwrap();
    assert!((amount(&runtime.render().unwrap()) - 0.2).abs() < 0.0001);
}

/// The same authored WGSL effect selects precise paint regions in AOT and live.
#[test]
fn effect_scope_reaches_both_render_paths() {
    for (name, expected) in [
        ("whole", Some(EffectScope::WholeElement)),
        ("background", Some(EffectScope::Background)),
        ("border", Some(EffectScope::Border)),
        ("content", Some(EffectScope::Content)),
        ("text", Some(EffectScope::Text)),
        ("backdrop", None),
    ] {
        let source = format!(
            r#"import {{ Rectangle }} from "@argui/native"
export effect Glow {{ shader: "effects/glow.wgsl" }}
export component Main {{ Rectangle {{
    width: 100px height: 50px background: solid(#ffffff)
    border_color: #ffffff border_width: 2.0
    effect: Glow {{ scope: "{name}" }}
}} }}"#
        );
        let compiled = Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Ok(SHADER.to_vec()),
        )
        .unwrap();
        assert!(compiled.rust.contains("apply_visual_effect_scoped"));
        assert!(compiled.rust.contains(&format!(
            "VisualEffectTarget::{}",
            match expected {
                Some(EffectScope::WholeElement) => "WholeElement",
                Some(EffectScope::Background) => "Background",
                Some(EffectScope::Border) => "Border",
                Some(EffectScope::Content) => "Content",
                Some(EffectScope::Text) => "Text",
                None => "Backdrop",
            }
        )));
        let root = compiled.roots[0];
        let shader = compiled.ir.effects[0].shader;
        let package = LivePackage::prepare(
            1,
            compiled.public_api_hash,
            compiled.ir,
            HashMap::from([(shader, AssetPayload::new(1, SHADER.to_vec()))]),
        )
        .unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.mount(root, []).unwrap();
        let element = runtime.render().unwrap();
        if let Some(scope) = expected {
            assert_eq!(element.effects[0].scope, scope, "{name}");
            assert!(matches!(
                element.effects[0].layer.filters[0],
                Filter::Effect(_)
            ));
        } else {
            assert!(element.effects.is_empty());
            assert!(matches!(
                element.layer.as_ref().unwrap().backdrop_filters[0],
                Filter::Effect(_)
            ));
        }
    }
}

mod damage {
    //! Explicit effect damage metadata shared by AOT and live definitions.

    use std::collections::HashMap;

    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{AssetPayload, LivePackage, LiveRuntime};
    use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};
    use argui_render::EffectDamage;

    const SHADER: &[u8] = b"fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source; }";

    /// The author must opt in to bounded damage and both backends honor that opt in.
    #[test]
    fn bounded_damage_reaches_aot_and_live_definitions() {
        let compiled = Compiler::compile(
            [SourceModule::new(
                "ui/main.argui",
                r#"import { Rectangle } from "@argui/native"
export effect Glow { shader: "effects/glow.wgsl" damage: "bounded" }
export component Main { Rectangle { effect: Glow {} } }"#,
            )],
            "ui/main.argui",
            |_| Ok(SHADER.to_vec()),
        )
        .unwrap();
        assert!(compiled.ir.effects[0].bounded_damage);
        assert!(compiled.rust.contains("EffectDamage::Bounded"));
        let shader = compiled.ir.effects[0].shader;
        let package = LivePackage::prepare(
            1,
            compiled.public_api_hash,
            compiled.ir,
            HashMap::from([(shader, AssetPayload::new(1, SHADER.to_vec()))]),
        )
        .unwrap();
        let runtime = LiveRuntime::new(package).unwrap();
        assert_eq!(
            runtime.effect_definitions()[0].damage,
            EffectDamage::Bounded
        );
    }

    /// Unknown values and duplicate declarations get source diagnostics.
    #[test]
    fn damage_declaration_rejects_unknown_and_duplicate_values() {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file(
            "ui/main.argui",
            r#"export effect Glow {
    shader: "effects/glow.wgsl"
    damage: "local"
    damage: "bounded"
}"#,
        );
        let issues = &database.check().diagnostics;
        assert!(issues.iter().any(|issue| {
            issue.code == DiagnosticCode::InvalidEffect
                && issue.message.contains("effect damage must be")
        }));
        assert!(issues.iter().any(|issue| {
            issue.code == DiagnosticCode::DuplicateMember
                && issue.message.contains("damage is declared more than once")
        }));
    }
}
