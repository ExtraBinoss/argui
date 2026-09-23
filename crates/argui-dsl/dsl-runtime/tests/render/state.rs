//! Focus traversal requests from live DSL event handlers.

use std::collections::HashMap;

use argui_core::{Key, Modifiers};
use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_runtime::{LivePackage, LiveRuntime};
use argui_testing::TestApp;

#[test]
fn live_key_bindings_move_focus_in_both_directions() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { FocusScope, KeyBinding } from "@argui/native"
export component Main {
    FocusScope #first { width: 50px height: 30px }
    FocusScope #second { width: 50px height: 30px }
    KeyBinding { shortcut: "ArrowDown" on activated { focus_next() } }
    KeyBinding { shortcut: "ArrowUp" on activated { focus_previous() } }
}"#,
        )],
        "ui/main.argui",
        |_| Err("focus test does not load assets".into()),
    )
    .unwrap();
    let root = compiled.roots[0];
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    let mut app = TestApp::new(runtime);
    app.focus("first").unwrap();
    app.assert_focused("first");
    app.key(Key::ArrowDown, Modifiers::default()).unwrap();
    app.assert_focused("second");
    app.key(Key::ArrowUp, Modifiers::default()).unwrap();
    app.assert_focused("first");
}

mod scroll {
    //! Identity-targeted scroll request parity across AOT and live handlers.

    use std::collections::HashMap;

    use argui_core::Point;
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_ir::{IrNode, SiteId};
    use argui_dsl_runtime::{LivePackage, LiveRuntime};
    use argui_ui::{RetainedIdentity, ScrollRequest};

    /// Finds an explicitly identified native site in a lowered component body.
    ///
    /// `nodes` is the visual tree and `name` is the source identity. Returns the
    /// stable site ID if that name occurs in the tree.
    fn site(nodes: &[IrNode], name: &str) -> Option<SiteId> {
        for node in nodes {
            if let IrNode::Element {
                site: id,
                source_id,
                children,
                ..
            } = node
            {
                if source_id.as_deref() == Some(name) {
                    return Some(*id);
                }
                if let Some(id) = site(children, name) {
                    return Some(id);
                }
            }
        }
        None
    }

    /// AOT and live handlers target the same retained viewport instance.
    #[test]
    fn scroll_to_targets_a_retained_child_identity() {
        let compiled = Compiler::compile(
            [SourceModule::new(
                "ui/main.argui",
                r#"import { Flickable, TouchArea } from "@argui/native"
export component Main {
    Flickable #viewport { width: 100px height: 100px
        TouchArea #drag { on moved { scroll_to(#viewport, 0px, 12px) } }
    }
}"#,
            )],
            "ui/main.argui",
            |_| Err("scroll test does not load assets".into()),
        )
        .unwrap();
        let root = compiled.roots[0];
        let body = &compiled
            .ir
            .components
            .iter()
            .find(|component| component.id == root)
            .unwrap()
            .body;
        let viewport = site(body, "viewport").unwrap();
        let drag = site(body, "drag").unwrap();
        assert!(compiled.rust.contains("HostEffect::Scroll"));
        assert!(
            compiled
                .rust
                .contains(&format!("RetainedIdentity::new(owner, {})", viewport.raw()))
        );
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let instance = runtime.mount(root, []).unwrap();
        runtime
            .dispatch_native_event(instance, drag, argui_schema::builtin::MOVED)
            .unwrap();
        assert_eq!(
            runtime.take_scroll_request(),
            Some(ScrollRequest::offset(
                RetainedIdentity::new(instance.raw(), viewport.raw()),
                Point::new(0.0, 12.0)
            ))
        );
    }
}

mod motion_edges {
    //! Typed motion output and state-edge behavior at the live runtime boundary.

    use std::collections::HashMap;

    use argui_animation::{Duration, Frame, Time};
    use argui_core::Color;
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
    use argui_runtime::{Context, Render};
    use argui_ui::ExpandedDimension;

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

    /// Color and percentage state transitions jump on entry and animate only on exit.
    #[test]
    fn leave_transition_restores_color_and_percentage_base_values() {
        let mut runtime = mounted(
            r##"import { Container } from "@argui/native"
export component Main {
    in property active: bool = false
    in property angle: float = 0.0
    states { open when active { angle: 90.0 } }
    Container {
        rotation: angle
        width: 25%
        background: #ff0000
        states { open when active { width: 75% background: #0000ff } }
        animate width { transition: leave duration: 100ms }
        animate background { transition: leave duration: 100ms }
    }
}"##,
        );
        let root = runtime.root().unwrap();
        let definition = runtime.instance(root).unwrap().component;
        let properties = &runtime
            .ir()
            .components
            .iter()
            .find(|item| item.id == definition)
            .unwrap()
            .properties;
        let active = properties[0].id;
        let angle = properties[1].id;
        let first = runtime.render().unwrap();
        assert_eq!(
            first.style.size.width.expand(),
            ExpandedDimension::Percent(0.25)
        );
        runtime
            .set_property(root, active, DslValue::Bool(true))
            .unwrap();
        let entered = runtime.render().unwrap();
        assert_eq!(
            entered.style.size.width.expand(),
            ExpandedDimension::Percent(0.75)
        );
        assert!((entered.transform.rotation - std::f32::consts::FRAC_PI_2).abs() < 0.001);
        assert_eq!(
            runtime.instance(root).unwrap().properties[&angle].get(),
            &DslValue::Float(0.0)
        );
        assert!(!runtime.wants_animation_frame());
        runtime
            .set_property(root, active, DslValue::Bool(false))
            .unwrap();
        assert_eq!(
            runtime.render().unwrap().style.size.width.expand(),
            ExpandedDimension::Percent(0.75)
        );
        advance(&mut runtime, 0, 0);
        advance(&mut runtime, 0, 50);
        let middle = runtime.render().unwrap();
        assert_eq!(
            middle.style.size.width.expand(),
            ExpandedDimension::Percent(0.5)
        );
        let Some(argui_paint::Fill::Solid(color)) = &middle.paint.quad.background else {
            panic!("expected animated background");
        };
        assert_ne!(*color, Color::from_srgba8(255, 0, 0, 255));
        assert_ne!(*color, Color::from_srgba8(0, 0, 255, 255));
        advance(&mut runtime, 50, 100);
        let restored = runtime.render().unwrap();
        assert_eq!(
            restored.style.size.width.expand(),
            ExpandedDimension::Percent(0.25)
        );
        assert_eq!(
            restored.paint.quad.background,
            Some(argui_paint::Fill::Solid(Color::from_srgba8(255, 0, 0, 255)))
        );
        assert!(!runtime.wants_animation_frame());
    }
}
