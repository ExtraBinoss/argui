//! Public child output reads share retained identities in AOT and live builds.

use std::collections::HashMap;

use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_ir::IrExpressionKind;
use argui_dsl_runtime::{LivePackage, LiveRuntime};
use argui_testing::TestApp;

/// A sibling reads an output derived from an identified child's observation.
#[test]
fn child_output_forwarding_reacts_to_native_hover() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { TouchArea } from "@argui/native"
import { Text } from "@argui/ui"
component Child {
    out property label: string = touch.has_hover ? "hovered" : "idle"
    TouchArea #touch { width: 160px height: 60px }
}
export component Main {
    Text { content: child.label }
    Child #child {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("child reference test has no assets".into()),
    )
    .unwrap();
    let root = compiled.roots[0];
    let main = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == root)
        .unwrap();
    let argui_dsl_ir::IrNode::Element { properties, .. } = &main.body[0] else {
        panic!("the sibling text must be a visual element");
    };
    assert!(matches!(
        properties[0].value.kind,
        IrExpressionKind::ChildPropertyRead { .. }
    ));
    assert!(compiled.rust.contains("child_ref_p_"));
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    let mut app = TestApp::new(runtime);
    app.assert_text("idle");
    app.click("touch").unwrap();
    app.assert_text("hovered");
}

/// A forwarded scroll observation stays a typed child output in both backends.
#[test]
fn child_output_forwards_scroll_offset_to_a_sibling() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Flickable } from "@argui/native"
import { Text } from "@argui/ui"
component ScrollView {
    out property offset_y: length = viewport.offset_y
    Flickable #viewport { width: 100px height: 100px }
}
export component Main {
    Text { content: str(scroll_view.offset_y / 1px) }
    ScrollView #scroll_view {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("scroll forwarding test has no assets".into()),
    )
    .unwrap();
    let root = compiled.roots[0];
    let main = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == root)
        .unwrap();
    assert_eq!(main.referenced_child_sites().len(), 1);
    assert!(compiled.rust.contains("child_properties.defaulted"));
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    let rendered = runtime.render().unwrap();
    assert!(!rendered.children.is_empty());
}

/// Event handlers can read the same retained child output as visual bindings.
#[test]
fn child_output_is_readable_in_a_live_event_handler() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text, TouchArea } from "@argui/native"
component Child {
    out property label: string = "ready"
    Text { content: label }
}
export component Main {
    private property result: string = "waiting"
    Text { content: result }
    TouchArea #trigger { width: 100px height: 40px on click { result = child.label } }
    Child #child {}
}"#,
        )],
        "ui/main.argui",
        |_| Err("event alias test has no assets".into()),
    )
    .unwrap();
    let root = compiled.roots[0];
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    let mut app = TestApp::new(runtime);
    app.assert_text("waiting");
    app.click("trigger").unwrap();
    app.assert_text("ready");
    app.assert_no_text("waiting");
}

/// An explicit value detaches its default, even if that default later becomes invalid.
#[test]
fn explicit_binding_stops_evaluating_a_replaced_default() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"
import { Text, Column } from "@argui/native"
component Child {
    in property divisor: int = 1
    in property value: int = 10 / divisor
    Column { Text { content: str(value) } }
}
export component Main { Child { divisor: 0 value: 42 } }
"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let root = compiled.roots[0];
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    let app = TestApp::new(runtime);
    app.assert_text("42");
    assert!(app.entity().read(|runtime| runtime.last_error().is_none()));
}
