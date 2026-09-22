//! Retained child property identities emitted by AOT node code generation.

use argui_dsl_compiler::{Compiler, SourceModule};

#[test]
fn aot_applies_typed_effect_and_registers_its_definition() {
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
        |_| {
            Ok(b"fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source * argui_param_f32(0u); }".to_vec())
        },
    )
    .unwrap();
    assert_eq!(compiled.reachability.effects.len(), 1);
    assert!(compiled.rust.contains("EffectArgument::new(\"amount\""));
    assert!(compiled.rust.contains("EffectValue::F32"));
    assert!(compiled.rust.contains("apply_visual_effect_scoped(element"));
    assert!(compiled.rust.contains("fn effect_definitions(&self)"));
}

/// An effect on a user component wraps the rendered child output.
#[test]
fn aot_applies_effect_to_user_component() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Rectangle } from "@argui/native"
export effect Glow { shader: "effects/glow.wgsl" }
component Tile { Rectangle {} }
export component Main { Tile { effect: Glow {} } }"#,
        )],
        "ui/main.argui",
        |_| {
            Ok(b"fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source; }".to_vec())
        },
    )
    .unwrap();
    assert!(compiled.rust.contains("apply_visual_effect_scoped(element"));
    assert!(compiled.rust.contains("VisualEffectTarget::WholeElement"));
}

#[test]
fn aot_event_handlers_bind_native_and_component_payloads() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { TextInput } from "@argui/native"
component Child { callback changed(value: string) }
export component Main {
    private property text: string = ""
    TextInput { on input(value) { text = value } }
    Child { on changed(value) { text = value } }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets in this test".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("UiEventKind::TextChanged(value)"));
    assert!(compiled.rust.contains("_event_payload_"));
    assert!(compiled.rust.contains("_parameter_0"));
}

/// Controlled native text values use the edit delta in generated code.
#[test]
fn aot_text_input_edits_without_copying_the_full_event_value() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { TextInput } from "@argui/native"
export component Main {
    in-out property value: string = "é🙂"
    TextInput { value <=> value on edit { } }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("UiEventKind::TextEdited(edit)"));
    assert!(compiled.rust.contains(".mutate(|value|"));
    assert!(compiled.rust.contains("edit.apply_to(value)"));
    assert!(!compiled.rust.contains("UiEventKind::TextChanged(value)"));
}

/// Standard-library inputs inherit the native edit route in generated code.
#[test]
fn aot_stdlib_input_and_text_area_use_text_edits() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Input, TextArea } from "@argui/ui"
export component Main {
    in-out property value: string = "é🙂"
    Input { value <=> value }
    TextArea { value <=> value on changed { } }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(
        compiled
            .rust
            .matches("UiEventKind::TextEdited(edit)")
            .count()
            >= 2
    );
    assert!(!compiled.rust.contains("UiEventKind::TextChanged(value)"));
    assert!(compiled.ir.assets.is_empty());
}

/// Component callbacks retain host effects requested by their parent handler.
#[test]
fn aot_component_callback_can_request_focus_traversal() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"component Child { callback changed() }
export component Main { Child { on changed { focus_next() } } }"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(
        compiled
            .rust
            .contains("let host_effects = host_effects.clone()")
    );
    assert!(compiled.rust.contains("FocusRequest::Next"));
}

#[test]
fn aot_reads_observed_native_outputs_by_retained_identity() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Rectangle, TouchArea } from "@argui/native"
export component Main {
    Rectangle { opacity: touch.has_hover ? 0.5 : 1.0 }
    TouchArea #touch {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets in this test".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("cx.observation_reader()"));
    assert!(
        compiled
            .rust
            .contains("observer.get(&::argui::ui::RetainedIdentity::new(owner,")
    );
    assert!(compiled.rust.contains("::argui::ui::VisualState::Hovered"));
}

#[test]
fn aot_watches_event_only_observations_before_dispatch() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { TouchArea } from "@argui/native"
export component Main {
    private property last_x: length = 0px
    TouchArea #touch { on moved { last_x = touch.mouse_x } }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets in this test".into()),
    )
    .unwrap();
    assert!(
        compiled
            .rust
            .contains("observer.watch(&::argui::ui::RetainedIdentity::new(owner,")
    );
    assert!(compiled.rust.contains(".pointer_position.map_or(0.0"));
}

/// Both drag axes read coalesced pan displacement directly from the event payload.
#[test]
fn aot_uses_pan_displacement_for_touch_area_drag_events() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { TouchArea } from "@argui/native"
export component Main {
    private property x: float = 0.0
    private property y: float = 0.0
    TouchArea { on drag_x(delta) { x = delta } on drag_y(delta) { y = delta } }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(
        compiled
            .rust
            .contains("GestureKind::Pan { total, .. } => total.x")
    );
    assert!(
        compiled
            .rust
            .contains("GestureKind::Pan { total, .. } => total.y")
    );
}

#[test]
fn aot_reads_focus_scope_output_through_the_same_observation_path() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { FocusScope, Rectangle } from "@argui/native"
export component Main {
    Rectangle { opacity: focus_area.has_focus ? 1.0 : 0.4 }
    FocusScope #focus_area {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets in this test".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("::argui::ui::VisualState::Focused"));
    assert!(
        compiled
            .rust
            .contains("observer.get(&::argui::ui::RetainedIdentity::new(owner,")
    );
}

#[test]
fn aot_focus_traversal_uses_host_effects_for_both_directions() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { KeyBinding } from "@argui/native"
export component Main {
    KeyBinding { shortcut: "ArrowDown" on activated { focus_next() } }
    KeyBinding { shortcut: "ArrowUp" on activated { focus_previous() } }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no assets in this test".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("FocusRequest::Next"));
    assert!(compiled.rust.contains("FocusRequest::Previous"));
    assert!(compiled.rust.contains("cx.focus_next()"));
    assert!(compiled.rust.contains("cx.focus_previous()"));
}

#[test]
fn authored_path_embeds_the_same_validated_svg_used_by_live_ir() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Path } from "@argui/native"
export component Main {
    private property selected: bool = false
    Path #shape {
        source: path(20.0, 20.0, [move_to(1.0, 1.0), line_to(19.0, 1.0), line_to(19.0, 19.0), close_path()], true, 1.5, false)
        width: 20px
        height: 20px
        color: selected ? #ff0000 : #0000ff
    }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no external assets in this test".into()),
    )
    .unwrap();
    let generated = compiled
        .ir
        .assets
        .iter()
        .find(|asset| asset.inline_bytes.is_some())
        .unwrap();
    let bytes = generated.inline_bytes.as_ref().unwrap();
    let byte_list = bytes
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",");
    assert!(compiled.rust.contains(&format!(
        "ASSET_{}: &[u8] = &[{byte_list}]",
        generated.id.raw()
    )));
    assert!(compiled.rust.contains("upsert_vector"));
    assert!(!compiled.dependencies.contains(&generated.path));
}

#[test]
fn aot_children_reuse_defaulted_properties_across_render_passes() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { TouchArea, Text } from "@argui/native"
component Child {
    private property activated: bool = false
    TouchArea { on click { activated = !activated } }
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
