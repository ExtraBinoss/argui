use argui_dsl_compiler::{Compiler, SourceModule};

#[test]
fn aot_generates_one_generic_motion_store_for_native_dimensions_colors_and_rotation() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "import { Svg, Container, Text } from \"@argui/native\"\nexport component Main { Container { width: 200px animate width { from: 100px to: 200px duration: 100ms } rotation: 0.0 animate rotation { from: 0.0 to: 360.0 duration: 1000ms iterations: \"infinite\" } Svg { source: asset(\"loader.svg\") rotation: 0.0 animate rotation { from: 0.0 to: 360.0 duration: 1000ms iterations: \"infinite\" } } Text { text: \"Hello\" color: #000000 animate color { from: #000000 to: #ffffff duration: 100ms } } } }",
        )],
        "ui/main.argui",
        |path| if path == "ui/loader.svg" { Ok(b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\"></svg>".to_vec()) } else { Err("missing".into()) },
    ).unwrap();
    assert!(compiled.rust.contains("property_motions.sample_number("));
    assert!(compiled.rust.contains("property_motions.sample_dimension("));
    assert!(compiled.rust.contains("property_motions.sample_color("));
    assert!(compiled.rust.contains("property_motions.begin_render()"));
    assert!(compiled.rust.contains("property_motions.end_render()"));
    assert!(compiled.rust.contains("fn wants_animation_frame(&self)"));
    assert!(compiled.rust.contains("cx.environment().reduced_motion"));
    assert!(
        compiled
            .rust
            .contains("pub fn animation_error(&self) -> Option<String>")
    );
    assert!(
        compiled
            .rust
            .contains("property_motions.report_error(error)")
    );
    assert!(
        !compiled
            .rust
            .contains("expect(\"valid animated property\")")
    );
    assert!(!compiled.rust.contains("SvgMotionStore"));
}

#[test]
fn imported_icon_animates_only_when_the_call_site_requests_it() {
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", "import { Spinner } from \"@argui/icons\"\nexport component Main { Spinner { rotation: 0.0 animate rotation { from: 0.0 to: 360.0 duration: 1000ms iterations: \"infinite\" } } }")],
        "ui/main.argui",
        |_| Err("virtual icon".into()),
    ).unwrap();
    assert!(compiled.rust.contains("property_motions.sample_number("));
    assert!(!compiled.rust.contains("spinning"));
}

#[test]
fn aot_animates_an_exposed_user_component_property_without_native_special_casing() {
    let source = "import { Container } from \"@argui/native\"\ncomponent Tile { in property tone: color = #ffffff Container { background: tone } }\nexport component Main { Tile { tone: #000000 animate tone { from: #000000 to: #ffffff duration: 200ms } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("property_motions.sample_color("));
    assert!(compiled.rust.contains("child_properties.controlled("));
    assert!(compiled.rust.contains("PropertyMotionKey::new("));
}

#[test]
fn aot_animates_a_component_own_property_before_descendant_reads() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { in property angle: float = 0.0 animate angle { from: 0.0 to: 360.0 duration: 1000ms iterations: \"infinite\" } Container { rotation: angle } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("let rendered_p_"));
    assert!(compiled.rust.contains("property_motions.sample_number("));
    assert!(compiled.rust.contains("SchemaValue::Float(rendered_p_"));
}

#[test]
fn aot_spring_uses_the_generic_property_motion_driver() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { Container { rotation: 45.0 animate rotation { spring { stiffness: 220 damping: 24 } } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("PropertyAnimation::spring("));
    assert!(compiled.rust.contains("property_motions.sample_number("));
}

#[test]
fn aot_animates_unbound_native_property_from_explicit_endpoint() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { Container { animate rotation { from: 0.0 to: 90.0 duration: 100ms } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("property_motions.sample_number("));
    assert!(compiled.rust.contains("SchemaValue::Float("));
}

#[test]
fn aot_emits_typed_multistop_keyframes_and_easing() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { Container { rotation: 0.0 animate rotation { duration: 100ms easing: \"ease-in\" keyframes { 0%: 0.0 50%: 90.0 100%: 0.0 } } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("with_easing_name("));
    assert!(compiled.rust.contains("PropertyAnimation::keyframes(vec!["));
    assert!(compiled.rust.contains("0.5_f32"));
}

#[test]
fn aot_native_state_transition_uses_schema_default_without_explicit_binding() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { in property expanded: bool = false Container { states { open when expanded { opacity: 0.5 } } animate opacity { transition: in-out duration: 100ms } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("StateTransitionPolicy::InOut"));
    assert!(compiled.rust.contains("with_state_transition("));
    assert!(compiled.rust.contains("property_motions.sample_number("));
    assert!(compiled.rust.contains("1_f32"));
}

#[test]
fn aot_component_own_and_child_state_transitions_preserve_canonical_inputs() {
    let source = "import { Container } from \"@argui/native\"\ncomponent Tile { in property angle: float = 0.0 Container { rotation: angle } }\nexport component Main { in property expanded: bool = false in property own_angle: float = 0.0 states { open when expanded { own_angle: 45.0 } } animate own_angle { transition: enter duration: 100ms } Tile { angle: own_angle states { open when expanded { angle: 90.0 } } animate angle { transition: leave duration: 100ms } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("StateTransitionPolicy::Enter"));
    assert!(compiled.rust.contains("StateTransitionPolicy::Leave"));
    assert!(compiled.rust.contains("let rendered_p_"));
    assert!(compiled.rust.contains("child_properties.controlled("));
    assert!(compiled.rust.matches("with_state_transition(").count() >= 2);
}

#[test]
fn aot_child_animation_passes_a_presentation_overlay_beside_canonical_input() {
    let source = "import { Container } from \"@argui/native\"\ncomponent Tile { in property angle: float = 0.0 Container { rotation: angle } }\nexport component Main { Tile { angle: 90.0 animate angle { from: 0.0 to: 90.0 duration: 100ms } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let rust = &compiled.rust;
    assert!(rust.contains("child_properties.controlled("));
    assert!(rust.contains("presented_p_"));
    assert!(rust.contains("_input_rendered_p_"));
    assert!(rust.contains("property_motions.sample_number("));
    assert!(rust.contains("let presented_child_"));
    assert!(rust.contains("= Some({ let target ="));
}

#[test]
fn aot_two_way_child_animation_keeps_alias_and_separate_presentation() {
    let source = "import { Container } from \"@argui/native\"\ncomponent Tile { in-out property angle: float = 0.0 Container { rotation: angle } }\nexport component Main { in-out property angle: float = 0.0 in property expanded: bool = false Tile { angle <=> angle states { open when expanded { angle: 90.0 } } animate angle { transition: in-out duration: 100ms } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let rust = &compiled.rust;
    assert!(rust.contains("child_p_"));
    assert!(rust.contains("= p_"));
    assert!(!rust.contains("child_properties.controlled("));
    assert!(rust.contains("StateTransitionPolicy::InOut"));
    assert!(rust.contains("_input_rendered_p_"));
}

#[test]
fn aot_child_inherits_parent_presentation_without_replacing_its_input() {
    let source = "import { Container } from \"@argui/native\"\ncomponent Tile { in property angle: float = 0.0 Container { rotation: angle } }\nexport component Main { in property angle: float = 0.0 animate angle { from: 0.0 to: 90.0 duration: 1000ms iterations: \"infinite\" } Tile { angle: angle } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let rust = &compiled.rust;
    assert!(rust.contains("child_properties.controlled("));
    assert!(rust.contains("_input_rendered_p_"));
    assert!(rust.contains("rendered_p_"));
    assert!(rust.contains("Some(rendered_p_"));
}

#[test]
fn aot_state_override_converts_native_dimension_values() {
    let source = r#"import { Container } from "@argui/native"
export component Main {
    in property expanded: bool = false
    Container {
        width: 100px
        states { open when expanded { width: 200px } }
    }
}"#;
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("SchemaValue::Dimension("));
    assert!(compiled.rust.contains("::argui::ui::length(100.0_f32)"));
    assert!(compiled.rust.contains("::argui::ui::length(200.0_f32)"));
}
