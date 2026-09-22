//! Observed native bindings and source identities across live generations.

use std::collections::HashMap;

use argui_core::{Affine2D, Point, PointerEvent, PointerPhase, Rect, Size};
use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
use argui_dsl_ir::{IrNode, SiteId};
#[cfg(not(target_arch = "wasm32"))]
use argui_dsl_protocol::{LivePackageEnvelope, PackageHeader};
#[cfg(not(target_arch = "wasm32"))]
use argui_dsl_runtime::{ClientEvent, LiveClient};
use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
use argui_paint::ClipChain;
#[cfg(not(target_arch = "wasm32"))]
use argui_testing::TestApp;
use argui_ui::{
    CursorIcon, Element, ElementKind, FocusPolicy, FocusRequest, GestureSet, HitRegion, HitShape,
    HitTestStyle, NodeId, RetainedIdentity, UiTree, VisualState,
};

/// Reads the first text descendant of a rendered component.
///
/// `element` is the current rendered subtree; returns its first text value.
fn first_text(element: &Element) -> Option<&str> {
    if let ElementKind::Text { content, .. } = &element.kind {
        return Some(content.as_str());
    }
    element.children.iter().find_map(first_text)
}

/// Child defaults depending on bound inputs refresh on the first and later live renders.
#[test]
fn bound_child_inputs_refresh_derived_defaults() {
    let compiled = compile(
        r#"import { Text } from "@argui/native"
component Preview {
    in property variant: string = "primary"
    in property value: float = 0.0
    private property description: string = variant + ":" + str(value)
    Text { content: description }
}
export component Main {
    in-out property variant: string = "outline"
    in-out property value: float = 25.0
    Preview { variant: variant value: value }
}"#,
    );
    let root = compiled.roots[0];
    let definition = compiled
        .ir
        .components
        .iter()
        .find(|part| part.id == root)
        .unwrap();
    let variant = definition.properties[0].id;
    let value = definition.properties[1].id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let instance = runtime.mount(root, []).unwrap();
    assert_eq!(first_text(&runtime.render().unwrap()), Some("outline:25"));
    runtime
        .set_property(instance, variant, DslValue::String("ghost".into()))
        .unwrap();
    runtime
        .set_property(instance, value, DslValue::Float(50.0))
        .unwrap();
    assert_eq!(first_text(&runtime.render().unwrap()), Some("ghost:50"));
}

/// Compiles `source` as one asset-free generation and returns both AOT and live outputs.
///
/// # Panics
///
/// Panics if the source fails semantic checking or code generation.
fn compile(source: &str) -> CompiledProject {
    Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("observation tests do not load external assets".into()),
    )
    .unwrap()
}

/// Searches `nodes` for the explicit `name` and returns its stable site, if found.
fn find_site(nodes: &[IrNode], name: &str) -> Option<SiteId> {
    for node in nodes {
        if let IrNode::Element {
            site,
            source_id,
            children,
            ..
        } = node
        {
            if source_id.as_deref() == Some(name) {
                return Some(*site);
            }
            if let Some(found) = find_site(children, name) {
                return Some(found);
            }
        }
    }
    None
}

/// Finds the retained node corresponding to `identity` in `tree`.
///
/// # Panics
///
/// Panics when the identity is absent from the retained tree.
fn node(tree: &UiTree, identity: &RetainedIdentity) -> NodeId {
    tree.node_ids()
        .iter()
        .copied()
        .find(|node| {
            tree.element_for(*node)
                .and_then(argui_ui::Element::source_identity)
                == Some(identity)
        })
        .expect("source identity must survive reconciliation")
}

/// Builds hit geometry for `node` with the requested `focus_policy`.
///
/// Returns an enabled rectangular region for pointer and focus tests.
fn region(node: NodeId, focus_policy: FocusPolicy) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(160.0, 60.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: HitTestStyle::default().slop,
        enabled: true,
        focus_policy,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    }
}

/// A committed package invalidates and rebuilds a visible observed expression.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn live_reload_rebuilds_an_observed_binding_without_input() {
    let first = compile(
        r#"import { TouchArea } from "@argui/native"
import { Text } from "@argui/ui"
export component Main {
    Text { content: touch.has_hover ? "hovered before" : "idle" }
    TouchArea #touch { width: 160px height: 60px }
}"#,
    );
    let second = compile(
        r#"import { TouchArea } from "@argui/native"
import { Text } from "@argui/ui"
export component Main {
    Text { content: touch.has_hover ? "hovered after" : "idle" }
    TouchArea #touch { width: 160px height: 60px }
}"#,
    );
    assert!(
        first
            .rust
            .contains("observer.get(&::argui::ui::RetainedIdentity::new(owner,")
    );
    assert!(
        second
            .rust
            .contains("observer.get(&::argui::ui::RetainedIdentity::new(owner,")
    );
    let root = first.roots[0];
    let package = LivePackage::prepare(1, first.public_api_hash, first.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    let mut app = TestApp::new(runtime);
    app.assert_text("idle");
    app.click("touch").unwrap();
    app.assert_text("hovered before");
    let envelope = LivePackageEnvelope {
        header: PackageHeader::current(second.public_api_hash, 2),
        roots: second.roots,
        ir: second.ir,
        assets: Vec::new(),
    };
    app.entity().update(|runtime, context| {
        assert!(matches!(
            LiveClient::apply(runtime, ClientEvent::Package(Box::new(envelope))).unwrap(),
            ClientEvent::Committed(_)
        ));
        context.notify();
    });
    app.settle().unwrap();
    app.assert_text("hovered after");
    app.assert_no_text("hovered before");
    assert_eq!(app.entity().read(LiveRuntime::generation), 2);
}

/// Explicit native sites keep retained focus and press state across source edits.
#[test]
fn unrelated_reload_edit_preserves_native_focus_and_press_identities() {
    let first = compile(
        r#"import { FocusScope, TouchArea } from "@argui/native"
export component Main {
    FocusScope #focus_area {
        TouchArea #touch { width: 160px height: 60px }
    }
}"#,
    );
    let second = compile(
        r#"import { FocusScope, TouchArea } from "@argui/native"
export component Main {
    private property unrelated: string = "edited"
    FocusScope #focus_area {
        TouchArea #touch { width: 160px height: 60px }
    }
}"#,
    );
    let root = first.roots[0];
    let first_component = first
        .ir
        .components
        .iter()
        .find(|part| part.id == root)
        .unwrap();
    let next_component = second
        .ir
        .components
        .iter()
        .find(|part| part.id == root)
        .unwrap();
    let focus_site = find_site(&first_component.body, "focus_area").unwrap();
    let touch_site = find_site(&first_component.body, "touch").unwrap();
    assert_eq!(
        focus_site,
        find_site(&next_component.body, "focus_area").unwrap()
    );
    assert_eq!(
        touch_site,
        find_site(&next_component.body, "touch").unwrap()
    );
    let package = LivePackage::prepare(1, first.public_api_hash, first.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let instance = runtime.mount(root, []).unwrap();
    let mut tree = UiTree::new(runtime.render().unwrap());
    let focus_identity = RetainedIdentity::new(instance.raw(), focus_site.raw());
    let touch_identity = RetainedIdentity::new(instance.raw(), touch_site.raw());
    let focus_node = node(&tree, &focus_identity);
    let touch_node = node(&tree, &touch_identity);
    tree.sync_focus(
        &[region(focus_node, FocusPolicy::TabStop)],
        Some(FocusRequest::Focus(focus_node.into())),
    );
    tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(10.0, 10.0)),
        &[region(touch_node, FocusPolicy::None)],
    );
    assert_eq!(tree.focused_node(), Some(focus_node));
    assert!(
        tree.visual_states(touch_node)
            .contains(VisualState::Pressed)
    );
    let next = LivePackage::prepare(2, second.public_api_hash, second.ir, HashMap::new()).unwrap();
    let prepared = runtime.prepare_reload(next).unwrap();
    let _ = runtime.commit_reload(prepared);
    tree.update(runtime.render().unwrap());
    assert_eq!(node(&tree, &focus_identity), focus_node);
    assert_eq!(node(&tree, &touch_identity), touch_node);
    assert_eq!(tree.focused_node(), Some(focus_node));
    assert!(
        tree.visual_states(touch_node)
            .contains(VisualState::Pressed)
    );
}
