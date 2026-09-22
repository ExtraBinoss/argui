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

mod animation_validation {
    //! Malformed live animation packages report actionable render errors.

    use std::collections::HashMap;

    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_ir::{
        IrAnimation, IrAnimationDriver, IrAnimationKeyframe, IrExpressionKind, IrTransitionPolicy,
        IrType, IrValue,
    };
    use argui_dsl_runtime::{LivePackage, LiveRuntime, RuntimeError};

    const ROTATION: &str = r#"import { Container } from "@argui/native"
export component Main {
    Container { rotation: 90.0 animate rotation { from: 0.0 to: 90.0 duration: 100ms } }
}"#;

    /// Checks that editing the compiled `source` with `mutate` reports `diagnostic`.
    ///
    /// The mutation models malformed IR arriving through the public live-package API.
    /// Panics if the valid fixture fails setup or the runtime accepts the invalid animation.
    fn assert_invalid(source: &str, mutate: impl FnOnce(&mut IrAnimation), diagnostic: &str) {
        let mut compiled = Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("motion fixtures have no assets".into()),
        )
        .unwrap();
        let root = compiled.roots[0];
        let animation = &mut compiled
            .ir
            .components
            .iter_mut()
            .find(|component| component.id == root)
            .unwrap()
            .animations[0];
        mutate(animation);
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let instance = runtime.mount(root, []).unwrap();
        let error = runtime.render().expect_err("invalid animation must fail");
        assert!(
            matches!(&error, RuntimeError::Schema(message) if message.contains(diagnostic)),
            "expected {diagnostic:?}, got {error:?}"
        );
        assert_eq!(runtime.root(), Some(instance));
    }

    /// Adds two keyframes from `animation`'s existing endpoints, keeping those endpoints.
    ///
    /// Panics when the animation fixture does not contain both endpoint parameters.
    fn add_keyframes(animation: &mut IrAnimation) {
        animation.keyframes = [(0.0, "from"), (1.0, "to")]
            .into_iter()
            .map(|(offset, name)| {
                let parameter = animation
                    .parameters
                    .iter()
                    .find(|item| item.name == name)
                    .unwrap();
                IrAnimationKeyframe {
                    offset,
                    value: parameter.value.clone(),
                    source: parameter.source.clone(),
                }
            })
            .collect();
    }

    /// Invalid wire driver combinations fail before creating an active motion.
    #[test]
    fn malformed_timing_and_keyframe_combinations_report_the_cause() {
        assert_invalid(
            ROTATION,
            |animation| animation.parameters.retain(|item| item.name != "duration"),
            "timeline animation requires `duration`",
        );
        for retained_endpoint in ["from", "to"] {
            assert_invalid(
                ROTATION,
                |animation| {
                    add_keyframes(animation);
                    animation
                        .parameters
                        .retain(|item| item.name == "duration" || item.name == retained_endpoint);
                },
                "keyframes cannot be mixed with `from` or `to`",
            );
        }
        assert_invalid(
            ROTATION,
            |animation| {
                add_keyframes(animation);
                animation.parameters.clear();
            },
            "timeline animation requires `duration`",
        );
        assert_invalid(
            ROTATION,
            |animation| {
                add_keyframes(animation);
                animation.parameters.clear();
                animation.driver = IrAnimationDriver::Spring;
            },
            "spring animations do not accept keyframes",
        );
        assert_invalid(
            ROTATION,
            |animation| animation.transition = Some(IrTransitionPolicy::Leave),
            "state transition has no state assignment",
        );
    }

    /// Typed driver diagnostics name the invalid parameter instead of failing silently.
    #[test]
    fn malformed_parameter_types_and_names_are_rejected() {
        for (name, value, diagnostic) in [
            (
                "from",
                IrValue::Bool(true),
                "animation `from` must be numeric",
            ),
            (
                "iterations",
                IrValue::Bool(true),
                "animation `iterations` must be a string",
            ),
            (
                "easing",
                IrValue::Bool(true),
                "animation `easing` must be a string",
            ),
            (
                "misspelled",
                IrValue::Float(1.0),
                "unsupported animation parameter `misspelled`",
            ),
        ] {
            assert_invalid(
                ROTATION,
                |animation| {
                    let parameter = &mut animation.parameters[0];
                    parameter.name = name.into();
                    parameter.value.kind = IrExpressionKind::Constant(value);
                },
                diagnostic,
            );
        }
    }

    /// Endpoint conversion rejects payloads that cannot represent the authored target.
    #[test]
    fn malformed_color_and_dimension_endpoints_report_the_target_type() {
        assert_invalid(
            r##"import { Container } from "@argui/native"
export component Main {
    Container { background: #ffffff animate background { from: #000000 to: #ffffff duration: 100ms } }
}"##,
            |animation| {
                animation.parameters[0].value.kind = IrExpressionKind::Constant(IrValue::Bool(true))
            },
            "animation `from` must be a color",
        );
        assert_invalid(
            r#"import { Container } from "@argui/native"
export component Main {
    Container { width: 100px animate width { from: 50px to: 100px duration: 100ms } }
}"#,
            |animation| animation.parameters[0].value.value_type = IrType::Float,
            "cannot convert `Float` to a dimension",
        );
    }
}
