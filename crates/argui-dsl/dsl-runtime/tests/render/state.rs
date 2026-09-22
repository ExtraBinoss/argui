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
