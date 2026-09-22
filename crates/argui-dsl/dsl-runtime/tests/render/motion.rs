use std::collections::HashMap;

use argui_animation::{Duration, Frame, Time};
use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_runtime::{AssetPayload, LivePackage, LiveRuntime};
use argui_runtime::{Context, Render};
use argui_ui::ElementKind;

#[test]
fn live_generic_rotation_keeps_one_motion_across_renders_and_reload() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "import { Spinner } from \"@argui/icons\"\nexport component Main { Spinner { rotation: 0.0 animate rotation { from: 0.0 to: 360.0 duration: 1000ms iterations: \"infinite\" } } }",
        )],
        "ui/main.argui",
        |_| Err("virtual icon must not use filesystem loader".into()),
    )
    .unwrap();
    let root = compiled.roots[0];
    let icon = compiled
        .ir
        .assets
        .iter()
        .find(|asset| asset.path == "@argui/icons/Loader2.svg")
        .unwrap()
        .id;
    let bytes = argui_dsl_stdlib::icon_svg("Loader2").unwrap().into_bytes();
    let assets = HashMap::from([(icon, AssetPayload::new(1, bytes.clone()))]);
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir.clone(), assets).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();

    let first = runtime.render().unwrap();
    assert!(matches!(first.kind, ElementKind::Vector { .. }));
    assert_eq!(first.transform.rotation, 0.0);
    assert!(runtime.wants_animation_frame());
    assert_eq!(runtime.inspect().animations, 1);

    let mut context = Context::<LiveRuntime>::default();
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(1),
            elapsed: Duration::ZERO,
        },
        &mut context,
    );
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(500_000_001),
            elapsed: Duration::from_millis(500),
        },
        &mut context,
    );

    let second = runtime.render().unwrap();
    assert!((second.transform.rotation - std::f32::consts::PI).abs() < 0.01);

    let assets = HashMap::from([(icon, AssetPayload::new(2, bytes))]);
    let next = LivePackage::prepare(2, compiled.public_api_hash, compiled.ir, assets).unwrap();
    let prepared = runtime.prepare_reload(next).unwrap();
    let _ = runtime.commit_reload(prepared);
    let reloaded = runtime.render().unwrap();
    assert!((reloaded.transform.rotation - second.transform.rotation).abs() < 0.01);
    assert!(runtime.wants_animation_frame());
}

#[test]
fn live_user_component_color_and_native_dimension_share_property_engine() {
    let source = "import { Container } from \"@argui/native\"\ncomponent Tile { in property tone: color = #ff0000 Container { background: tone } }\nexport component Main { Container { width: 200px animate width { from: 100px to: 200px duration: 100ms } Tile { tone: #0000ff animate tone { from: #ff0000 to: #0000ff duration: 100ms } } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(compiled.roots[0], []).unwrap();
    let first = runtime.render().unwrap();
    assert_eq!(
        first.style.size.width.expand(),
        argui_ui::ExpandedDimension::Length(100.0)
    );
    assert_eq!(runtime.inspect().animations, 2);
    assert!(runtime.wants_animation_frame());
    let mut context = Context::<LiveRuntime>::default();
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(1),
            elapsed: Duration::ZERO,
        },
        &mut context,
    );
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(50_000_001),
            elapsed: Duration::from_millis(50),
        },
        &mut context,
    );
    let halfway = runtime.render().unwrap();
    let argui_ui::ExpandedDimension::Length(width) = halfway.style.size.width.expand() else {
        panic!("expected length")
    };
    assert!((width - 150.0).abs() < 0.01);
    let child = &halfway.children[0];
    let Some(argui_paint::Fill::Solid(color)) = &child.paint.quad.background else {
        panic!("expected solid background")
    };
    assert_ne!(*color, argui_core::Color::from_srgba8(255, 0, 0, 255));
    assert_ne!(*color, argui_core::Color::from_srgba8(0, 0, 255, 255));
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(100_000_001),
            elapsed: Duration::from_millis(50),
        },
        &mut context,
    );
    runtime.render().unwrap();
    assert!(!runtime.wants_animation_frame());
}

#[test]
fn live_component_own_property_animates_without_mutating_its_canonical_input() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { in property angle: float = 0.0 animate angle { from: 0.0 to: 360.0 duration: 1000ms iterations: \"infinite\" } Container { rotation: angle } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let angle = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == compiled.roots[0])
        .unwrap()
        .properties[0]
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(compiled.roots[0], []).unwrap();
    assert_eq!(runtime.render().unwrap().transform.rotation, 0.0);
    let mut context = Context::<LiveRuntime>::default();
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(1),
            elapsed: Duration::ZERO,
        },
        &mut context,
    );
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(500_000_001),
            elapsed: Duration::from_millis(500),
        },
        &mut context,
    );
    let halfway = runtime.render().unwrap();
    assert!((halfway.transform.rotation - std::f32::consts::PI).abs() < 0.01);
    assert_eq!(
        runtime.instance(root).unwrap().properties[&angle].get(),
        &argui_dsl_runtime::DslValue::Float(0.0)
    );
}

#[test]
fn live_spring_retargets_a_component_property_without_resetting_state() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { in property angle: float = 0.0 animate angle { spring { stiffness: 220 damping: 24 } } Container { rotation: angle } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let angle = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == compiled.roots[0])
        .unwrap()
        .properties[0]
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(compiled.roots[0], []).unwrap();
    runtime.render().unwrap();
    assert!(!runtime.wants_animation_frame());
    runtime
        .set_property(root, angle, argui_dsl_runtime::DslValue::Float(180.0))
        .unwrap();
    assert_eq!(runtime.render().unwrap().transform.rotation, 0.0);
    assert!(runtime.wants_animation_frame());
    let mut context = Context::<LiveRuntime>::default();
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(1),
            elapsed: Duration::ZERO,
        },
        &mut context,
    );
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(50_000_001),
            elapsed: Duration::from_millis(50),
        },
        &mut context,
    );
    let advancing = runtime.render().unwrap().transform.rotation;
    assert!(advancing > 0.0 && advancing < std::f32::consts::PI);
    runtime
        .set_property(root, angle, argui_dsl_runtime::DslValue::Float(270.0))
        .unwrap();
    let retargeted = runtime.render().unwrap().transform.rotation;
    assert!((retargeted - advancing).abs() < 0.01);
    assert_eq!(runtime.inspect().animations, 1);
}

#[test]
fn live_animates_unbound_native_property_from_explicit_endpoint() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { Container { animate rotation { from: 0.0 to: 90.0 duration: 100ms } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(compiled.roots[0], []).unwrap();
    assert_eq!(runtime.render().unwrap().transform.rotation, 0.0);
    assert_eq!(runtime.inspect().animations, 1);
    let mut context = Context::<LiveRuntime>::default();
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(1),
            elapsed: Duration::ZERO,
        },
        &mut context,
    );
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(50_000_001),
            elapsed: Duration::from_millis(50),
        },
        &mut context,
    );
    let halfway = runtime.render().unwrap().transform.rotation;
    assert!((halfway - std::f32::consts::FRAC_PI_4).abs() < 0.01);
}

#[test]
fn live_multistop_keyframes_apply_easing_per_segment() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { Container { rotation: 0.0 animate rotation { duration: 100ms easing: \"ease-in\" keyframes { 0%: 0.0 50%: 90.0 100%: 0.0 } } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(compiled.roots[0], []).unwrap();
    assert_eq!(runtime.render().unwrap().transform.rotation, 0.0);
    let mut context = Context::<LiveRuntime>::default();
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(1),
            elapsed: Duration::ZERO,
        },
        &mut context,
    );
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(25_000_001),
            elapsed: Duration::from_millis(25),
        },
        &mut context,
    );
    let quarter = runtime.render().unwrap().transform.rotation;
    assert!(quarter > 0.0 && quarter < std::f32::consts::FRAC_PI_4);
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(50_000_001),
            elapsed: Duration::from_millis(25),
        },
        &mut context,
    );
    let middle = runtime.render().unwrap().transform.rotation;
    assert!((middle - std::f32::consts::FRAC_PI_2).abs() < 0.01);
}

#[test]
fn live_state_transition_uses_schema_default_and_retargets_on_both_edges() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { in property expanded: bool = false Container { states { open when expanded { opacity: 0.5 } } animate opacity { transition: in-out duration: 100ms } } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let expanded = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == compiled.roots[0])
        .unwrap()
        .properties[0]
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(compiled.roots[0], []).unwrap();
    assert_eq!(
        runtime.render().unwrap().layer.as_ref().unwrap().opacity,
        1.0
    );
    assert!(!runtime.wants_animation_frame());
    runtime
        .set_property(root, expanded, argui_dsl_runtime::DslValue::Bool(true))
        .unwrap();
    assert_eq!(
        runtime.render().unwrap().layer.as_ref().unwrap().opacity,
        1.0
    );
    assert!(runtime.wants_animation_frame());
    let mut context = Context::<LiveRuntime>::default();
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(1),
            elapsed: Duration::ZERO,
        },
        &mut context,
    );
    runtime.animation_frame(
        Frame {
            now: Time::from_nanos(100_000_001),
            elapsed: Duration::from_millis(100),
        },
        &mut context,
    );
    assert_eq!(
        runtime.render().unwrap().layer.as_ref().unwrap().opacity,
        0.5
    );
    assert!(!runtime.wants_animation_frame());
    runtime
        .set_property(root, expanded, argui_dsl_runtime::DslValue::Bool(false))
        .unwrap();
    assert_eq!(
        runtime.render().unwrap().layer.as_ref().unwrap().opacity,
        0.5
    );
    assert!(runtime.wants_animation_frame());
}

#[test]
fn live_component_state_changes_presentation_without_mutating_input() {
    let source = "import { Container } from \"@argui/native\"\nexport component Main { in property expanded: bool = false in property angle: float = 0.0 states { open when expanded { angle: 90.0 } } animate angle { transition: enter duration: 100ms } Container { rotation: angle } }";
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let component = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == compiled.roots[0])
        .unwrap();
    assert_eq!(component.properties.len(), 2);
    let expanded = component.properties[0].id;
    let angle = component.properties[1].id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(compiled.roots[0], []).unwrap();
    assert_eq!(runtime.render().unwrap().transform.rotation, 0.0);
    runtime
        .set_property(root, expanded, argui_dsl_runtime::DslValue::Bool(true))
        .unwrap();
    assert_eq!(runtime.render().unwrap().transform.rotation, 0.0);
    assert_eq!(
        runtime.instance(root).unwrap().properties[&angle].get(),
        &argui_dsl_runtime::DslValue::Float(0.0)
    );
    assert!(runtime.wants_animation_frame());
}

#[test]
fn nested_child_keeps_canonical_input_while_inheriting_parent_animation() {
    for (direction, binding) in [("in", "angle: angle"), ("in-out", "angle <=> angle")] {
        let source = format!(
            "import {{ Container }} from \"@argui/native\"\ncomponent Tile {{ {direction} property angle: float = 0.0 Container {{ rotation: angle }} }}\nexport component Main {{ in-out property angle: float = 0.0 animate angle {{ from: 0.0 to: 90.0 duration: 1000ms iterations: \"infinite\" }} Tile {{ {binding} }} }}"
        );
        let compiled = Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("no assets".into()),
        )
        .unwrap();
        let root_component = compiled.roots[0];
        let package = LivePackage::prepare(
            1,
            compiled.public_api_hash,
            compiled.ir.clone(),
            HashMap::new(),
        )
        .unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(root_component, []).unwrap();
        assert_eq!(runtime.render().unwrap().transform.rotation, 0.0);
        let mut context = Context::<LiveRuntime>::default();
        runtime.animation_frame(
            Frame {
                now: Time::from_nanos(1),
                elapsed: Duration::ZERO,
            },
            &mut context,
        );
        runtime.animation_frame(
            Frame {
                now: Time::from_nanos(500_000_001),
                elapsed: Duration::from_millis(500),
            },
            &mut context,
        );
        let halfway = runtime.render().unwrap();
        assert!((halfway.transform.rotation - std::f32::consts::FRAC_PI_4).abs() < 0.01);
        let inspection = runtime.inspect();
        let child = inspection
            .instances
            .iter()
            .find(|instance| instance.id != root)
            .unwrap();
        assert_eq!(child.properties.len(), 1);
        assert_eq!(
            child.properties[0].1,
            argui_dsl_runtime::DslValue::Float(0.0)
        );
        let next =
            LivePackage::prepare(2, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let prepared = runtime.prepare_reload(next).unwrap();
        let _ = runtime.commit_reload(prepared);
        let reloaded = runtime.render().unwrap();
        assert!((reloaded.transform.rotation - halfway.transform.rotation).abs() < 0.01);
        let inspection = runtime.inspect();
        let child = inspection
            .instances
            .iter()
            .find(|instance| instance.id != root)
            .unwrap();
        assert_eq!(
            child.properties[0].1,
            argui_dsl_runtime::DslValue::Float(0.0)
        );
    }
}

mod integer_values {
    //! Typed motion output and state-edge behavior at the live runtime boundary.

    use std::collections::HashMap;

    use argui_animation::{Duration, Frame, Time};
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
    use argui_runtime::{Context, Render};
    use argui_ui::{Element, ElementKind};

    /// Compiles and mounts `source`, returning its live runtime without advancing time.
    ///
    /// Panics when the fixture fails compilation, package preparation, or mounting.
    fn mounted(source: &str) -> LiveRuntime {
        let compiled = Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("motion fixtures have no assets".into()),
        )
        .unwrap();
        let root = compiled.roots[0];
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.mount(root, []).unwrap();
        runtime
    }

    /// Advances `runtime` from `previous_ms` to `now_ms` using a deterministic clock.
    fn advance(runtime: &mut LiveRuntime, previous_ms: u64, now_ms: u64) {
        runtime.animation_frame(
            Frame {
                now: Time::from_nanos(now_ms * 1_000_000 + 1),
                elapsed: Duration::from_millis(now_ms - previous_ms),
            },
            &mut Context::default(),
        );
    }

    /// Returns the number of rendered caret primitives on the editor `element`.
    ///
    /// Panics if `element` is not an editor.
    fn caret_count(element: &Element) -> usize {
        let ElementKind::TextEditor { caret, .. } = &element.kind else {
            panic!("expected an editor");
        };
        caret.visual.primitives.len()
    }

    /// Integer component inputs and native properties round at the same frame boundary.
    #[test]
    fn integer_motion_rounds_editor_caret_counts_without_mutating_inputs() {
        let mut runtime = mounted(
            r#"import { Container, TextInput } from "@argui/native"
component Editor {
    in property count: int = 1
    TextInput { caret_count: count caret_blink: false }
}
export component Main {
    Container {
        TextInput {
            caret_count: 3
            caret_blink: false
            animate caret_count { from: 1 to: 3 duration: 100ms }
        }
        Editor { count: 3 animate count { from: 1 to: 3 duration: 100ms } }
    }
}"#,
        );
        let first = runtime.render().unwrap();
        assert_eq!(caret_count(&first.children[0]), 1);
        assert_eq!(caret_count(&first.children[1]), 1);
        advance(&mut runtime, 0, 0);
        advance(&mut runtime, 0, 26);
        let middle = runtime.render().unwrap();
        assert_eq!(caret_count(&middle.children[0]), 2);
        assert_eq!(caret_count(&middle.children[1]), 2);
        let root = runtime.root().unwrap();
        let inspection = runtime.inspect();
        let child = inspection
            .instances
            .iter()
            .find(|item| item.id != root)
            .unwrap();
        assert_eq!(child.properties[0].1, DslValue::Int(3));
        advance(&mut runtime, 26, 100);
        let final_frame = runtime.render().unwrap();
        assert_eq!(caret_count(&final_frame.children[0]), 3);
        assert_eq!(caret_count(&final_frame.children[1]), 3);
        assert!(!runtime.wants_animation_frame());
    }
}
