//! Dynamic theme mode behavior in the live runtime.

use std::collections::HashMap;

use argui_dsl_compiler::{Compiler, CompilerError, SourceModule};
use argui_dsl_ir::{IrNode, ThemeModeId};
use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime, RuntimeError};

/// A handler switches a dynamic mode and sees custom overrides under canonical token names.
#[test]
fn event_mode_switch_composes_custom_theme_and_rejects_unknown_dynamic_name() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { TouchArea } from "@argui/native"
export theme CustomTheme {
    --argui-background: color = #112233
    light { --argui-background: #abcdef }
    dark { --argui-background: #445566 }
}
export component Main {
    in-out property selected: string = "dark"
    in-out property observed: color = #000000
    TouchArea {
        on click { set_theme_mode(selected) observed = var(--argui-background) }
    }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no external assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("fn set_theme_mode(name: String)"));
    let component = compiled
        .ir
        .components
        .iter()
        .find(|component| {
            compiled.semantic.modules.iter().any(|module| {
                module.definitions.iter().any(|definition| {
                    definition.name == "Main" && definition.id.raw() == component.id.raw()
                })
            })
        })
        .unwrap();
    let selected = component.properties[0].id;
    let observed = component.properties[1].id;
    let component_id = component.id;
    let site = match &component.body[0] {
        IrNode::Element { site, .. } => *site,
        _ => panic!("expected TouchArea"),
    };
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(component_id, []).unwrap();
    runtime.render().unwrap();
    runtime
        .dispatch_native_event(root, site, argui_schema::builtin::CLICK)
        .unwrap();
    assert_eq!(
        runtime.instance(root).unwrap().properties[&observed].get(),
        &DslValue::Color(argui_core::Color::from_srgba8(0x44, 0x55, 0x66, 0xff))
    );
    assert_eq!(
        runtime.inspect().theme_mode,
        Some(ThemeModeId::named("dark"))
    );

    runtime
        .set_property(root, selected, DslValue::String("light".into()))
        .unwrap();
    runtime
        .dispatch_native_event(root, site, argui_schema::builtin::CLICK)
        .unwrap();
    assert_eq!(
        runtime.instance(root).unwrap().properties[&observed].get(),
        &DslValue::Color(argui_core::Color::from_srgba8(0xab, 0xcd, 0xef, 0xff))
    );

    runtime
        .set_property(root, selected, DslValue::String("missing".into()))
        .unwrap();
    assert!(matches!(
        runtime.dispatch_native_event(root, site, argui_schema::builtin::CLICK),
        Err(RuntimeError::InvalidBytecode(message)) if message.contains("unknown theme mode")
    ));
    assert_eq!(
        runtime.inspect().theme_mode,
        Some(ThemeModeId::named("light"))
    );
}

/// A misspelled literal mode is rejected before live execution or AOT generation.
#[test]
fn unknown_literal_mode_is_a_compile_diagnostic() {
    let error = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { TouchArea } from "@argui/native"
export component Main {
    TouchArea { on click { set_theme_mode("missing") } }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no external assets".into()),
    )
    .err()
    .expect("unknown literal mode must fail compilation");
    assert!(matches!(
        error,
        CompilerError::Semantic(diagnostics)
            if diagnostics.iter().any(|diagnostic| diagnostic.message.contains("unknown theme mode `missing`"))
    ));
}

/// A mode-only custom theme remains reachable in AOT without redeclaring a token.
#[test]
fn mode_only_custom_theme_overrides_builtin_token_in_aot() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { TouchArea, Rectangle } from "@argui/native"
export theme OverrideOnly { dark { --argui-primary: #aabbcc } }
export component Main {
    TouchArea {
        Rectangle { background: solid(var(--argui-primary)) }
        on click { set_theme_mode("dark") }
    }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no external assets".into()),
    )
    .unwrap();
    let custom = compiled
        .ir
        .themes
        .iter()
        .find(|theme| theme.tokens.is_empty() && !theme.modes.is_empty())
        .unwrap();
    assert!(compiled.reachability.themes.contains(&custom.id));
    assert!(
        compiled
            .rust
            .contains("::argui::core::Color::from_srgba8(170, 187, 204, 255)")
    );
}
