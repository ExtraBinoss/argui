//! Gesture and scroll payload routing used by gallery resize and scrolling controls.

use std::collections::HashMap;

use argui_core::{Point, PointerId};
use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_ir::PropertyId;
use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
use argui_dsl_semantic::DefinitionKind;
use argui_runtime::{Entity, WindowEnvironment};
use argui_ui::{GestureDelivery, GestureEvent, GestureKind, GesturePhase, UiEventKind, UiTree};

/// Compiles and mounts `source`, returning its root and source-named properties.
fn fixture(
    source: &str,
) -> (
    LiveRuntime,
    argui_dsl_runtime::InstanceId,
    HashMap<String, PropertyId>,
) {
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no external assets".into()),
    )
    .unwrap();
    let component = compiled.roots[0];
    let semantic = compiled
        .semantic
        .modules
        .iter()
        .find(|module| module.path == "ui/main.argui")
        .unwrap()
        .definitions
        .iter()
        .find(|definition| definition.name == "Main")
        .unwrap();
    let DefinitionKind::Component(semantic) = &semantic.kind else {
        panic!("expected root component")
    };
    let ir = compiled
        .ir
        .components
        .iter()
        .find(|item| item.id == component)
        .unwrap();
    let properties = semantic
        .properties
        .iter()
        .zip(&ir.properties)
        .map(|(semantic, ir)| (semantic.name.clone(), ir.id))
        .collect();
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(component, []).unwrap();
    (runtime, root, properties)
}

/// Resize handles receive accumulated pan displacement on both axes, not frame deltas.
#[test]
fn resize_handlers_receive_distinct_total_displacements() {
    let (runtime, root, properties) = fixture(
        r#"import { TouchArea } from "@argui/native"
export component Main {
    private property horizontal: float = 0.0
    private property vertical: float = 0.0
    TouchArea {
        on drag_x(delta) { horizontal = delta }
        on drag_y(delta) { vertical = delta }
    }
}"#,
    );
    let entity = Entity::new(runtime);
    let mount = entity.mount().unwrap();
    let mut tree = UiTree::new(mount.render(WindowEnvironment::default()).unwrap());
    let node = tree.node_id_at(0).unwrap();
    for total in [Point::new(35.5, -12.25), Point::new(-4.0, 18.75)] {
        let event = UiEventKind::Gesture(GestureEvent {
            target: node,
            pointer: PointerId::MOUSE,
            phase: GesturePhase::Changed,
            kind: GestureKind::Pan {
                position: Point::default(),
                delta: Point::new(1.0, 2.0),
                total,
                velocity: Point::default(),
            },
            delivery: GestureDelivery::Immediate,
        });
        let deliveries = tree.event_deliveries(node, event);
        assert!(!deliveries.is_empty());
        for event in deliveries {
            mount.dispatch_event(&event).unwrap();
        }
        mount.read(|runtime| {
            let instance = runtime.instance(root).unwrap();
            assert_eq!(
                instance.properties[&properties["horizontal"]].get(),
                &DslValue::Float(f64::from(total.x))
            );
            assert_eq!(
                instance.properties[&properties["vertical"]].get(),
                &DslValue::Float(f64::from(total.y))
            );
            assert!(runtime.last_error().is_none());
        });
    }
}

/// Scrolling updates the controlled virtual offset before delivering the handler payload.
#[test]
fn virtual_scrolling_updates_two_way_state_before_handler() {
    let (runtime, root, properties) = fixture(
        r#"import { VirtualWindow, Text } from "@argui/native"
export component Main {
    private property offset: float = 0.0
    private property received: float = -1.0
    private property controlled: float = -1.0
    VirtualWindow {
        row_height: 20.0
        viewport_height: 60.0
        offset <=> offset
        on scroll(next) { received = next controlled = offset }
        for item in ["one", "two", "three", "four", "five"] key item { Text { content: item } }
    }
}"#,
    );
    let entity = Entity::new(runtime);
    let mount = entity.mount().unwrap();
    let mut tree = UiTree::new(mount.render(WindowEnvironment::default()).unwrap());
    let node = tree.node_id_at(0).unwrap();
    let events = tree.event_deliveries(
        node,
        UiEventKind::Scrolled {
            delta: Point::new(0.0, 5.0),
            offset: Point::new(0.0, 40.0),
        },
    );
    assert!(!events.is_empty());
    for event in events {
        mount.dispatch_event(&event).unwrap();
    }
    mount.read(|runtime| {
        for name in ["offset", "received", "controlled"] {
            assert_eq!(
                runtime.instance(root).unwrap().properties[&properties[name]].get(),
                &DslValue::Float(40.0),
                "{name}"
            );
        }
        assert!(runtime.last_error().is_none());
    });
}
