use std::collections::HashMap;

use argui_core::{Point, Rect, Size};
use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime, RuntimeError};
use argui_dsl_semantic::DefinitionKind;
use argui_runtime::{Context, LayoutBounds, LayoutSnapshot, Render, ViewUpdate};
use argui_testing::TestApp;
use argui_ui::Role;
use argui_ui::{Element, ElementKind, UiTree};

/// Collects visible text from one rendered subtree.
fn visible_text(element: &Element, output: &mut Vec<String>) {
    if let ElementKind::Text { content, .. } = &element.kind {
        output.push(content.as_str().to_owned());
    }
    for child in &element.children {
        visible_text(child, output);
    }
}

/// VirtualWindow lazily mounts the same bounded keyed rows as the DSL ListView.
#[test]
fn live_virtual_window_mounts_only_visible_rows() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { VirtualWindow, Text } from "@argui/native"
export component Main {
    in property items: model<string>
    private property offset: float = 0.0
    VirtualWindow #viewport { row_height: 20.0 viewport_height: 60.0 overscan: 2 offset <=> offset
        for item in items key item { Text { content: item } }
    }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no external assets".into()),
    )
    .unwrap();
    let root = compiled.roots[0];
    let definition = compiled
        .ir
        .components
        .iter()
        .find(|part| part.id == root)
        .unwrap();
    let items = definition.properties[0].id;
    let offset = definition.properties[1].id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let values = (0..1000)
        .map(|index| DslValue::String(format!("row-{index}")))
        .collect();
    let instance = runtime
        .mount(root, [(items, DslValue::Array(values))])
        .unwrap();
    let mut visible = Vec::new();
    visible_text(&runtime.render().unwrap(), &mut visible);
    assert!(visible.len() <= 12, "mounted {} rows", visible.len());
    assert!(visible.iter().any(|value| value == "row-0"));
    runtime
        .set_property(instance, offset, DslValue::Float(2000.0))
        .unwrap();
    visible.clear();
    visible_text(&runtime.render().unwrap(), &mut visible);
    assert!(visible.len() <= 12, "mounted {} rows", visible.len());
    assert!(visible.iter().any(|value| value == "row-100"));
}

#[test]
fn live_virtual_list_mounts_only_visible_model_rows_and_moves_its_window() {
    let source = r#"import { VirtualWindow, Text } from "@argui/native"
export component Main {
    in property items: model<string>
    private property row_height: float = 20.0
    private property offset: float = 0.0
    VirtualWindow #rows {
        row_height: row_height
        viewport_height: 60.0
        overscan: 2
        offset <=> offset
        for item in items key item { Text { content: item } }
    }
}"#;
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no external assets".into()),
    )
    .unwrap();
    let root_component = compiled.roots[0];
    let definition = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == root_component)
        .unwrap();
    let items_property = definition.properties[0].id;
    let height_property = definition.properties[1].id;
    let offset_property = definition.properties[2].id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let rows = (0..1000)
        .map(|index| DslValue::String(format!("row-{index}")))
        .collect::<Vec<_>>();
    let root = runtime
        .mount(root_component, [(items_property, DslValue::Array(rows))])
        .unwrap();
    let mut first = Vec::new();
    let first_element = runtime.render().unwrap();
    assert_eq!(first_element.key.as_deref(), Some("rows"));
    visible_text(&first_element, &mut first);
    assert!(first.len() <= 12, "mounted {} rows", first.len());
    assert!(first.iter().any(|value| value == "row-0"));
    runtime
        .set_property(root, offset_property, DslValue::Float(2000.0))
        .unwrap();
    let mut scrolled = Vec::new();
    visible_text(&runtime.render().unwrap(), &mut scrolled);
    assert!(
        scrolled.len() <= 12,
        "mounted {} rows after scroll",
        scrolled.len()
    );
    assert!(scrolled.iter().any(|value| value == "row-100"));
    assert!(!scrolled.iter().any(|value| value == "row-0"));
    runtime
        .set_property(root, offset_property, DslValue::Float(100_000.0))
        .unwrap();
    let mut clamped = Vec::new();
    visible_text(&runtime.render().unwrap(), &mut clamped);
    assert!(clamped.iter().any(|value| value == "row-999"));
    runtime
        .set_property(root, items_property, DslValue::Array(Vec::new()))
        .unwrap();
    let mut empty = Vec::new();
    visible_text(&runtime.render().unwrap(), &mut empty);
    assert!(empty.is_empty());
    runtime
        .set_property(root, height_property, DslValue::Float(0.0))
        .unwrap();
    assert!(matches!(runtime.render(), Err(RuntimeError::Schema(_))));
}

#[test]
fn live_virtual_list_recomputes_its_window_from_layout_height() {
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", r#"import { VirtualWindow, Text } from "@argui/native"
export component Main {
    private property items: array<string> = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"]
    private property offset: float = 0.0
    VirtualWindow #measured { row_height: 20.0 height: 100% overscan: 1
        offset <=> offset
        for item in items key item { Text { content: item } }
    }
}"#)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    ).unwrap();
    let root_component = compiled.roots[0];
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root_component, []).unwrap();
    let mut before = Vec::new();
    let element = runtime.render().unwrap();
    visible_text(&element, &mut before);
    let tree = UiTree::new(element);
    let snapshot = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(400.0, 300.0)),
        nodes: vec![LayoutBounds {
            node: tree.node_id_at(0).unwrap(),
            key: Some("measured".into()),
            retained_identity: tree
                .element_at(0)
                .and_then(|element| element.source_identity().cloned()),
            bounds: Rect::new(Point::default(), Size::new(200.0, 200.0)),
        }],
    };
    let mut context = Context::<LiveRuntime>::default();
    Render::layout_changed(&mut runtime, &snapshot, &mut context);
    assert_eq!(context.view_update(), ViewUpdate::Rebuild);
    let mut stable_context = Context::<LiveRuntime>::default();
    Render::layout_changed(&mut runtime, &snapshot, &mut stable_context);
    assert_eq!(stable_context.view_update(), ViewUpdate::None);
    let mut after = Vec::new();
    visible_text(&runtime.render().unwrap(), &mut after);
    assert!(
        after.len() > before.len(),
        "before {} rows; after {}",
        before.len(),
        after.len()
    );
    assert!(after.len() < 26);
}

#[test]
fn template_lists_with_equal_row_keys_keep_handlers_and_instances_independent() {
    let source = r#"import { ListView, Button } from "@argui/ui"
import { Column } from "@argui/native"
export component Main {
    private property items: array<string> = ["same"]
    private property left_offset: float = 0.0
    private property right_offset: float = 0.0
    private property left_selected: string = ""
    private property right_selected: string = ""
    Column {
        width: 100%
        height: 100%
        ListView #left {
            row_height: 40.0
            offset <=> left_offset
            for item in items key item {
                Button { text: "Left " + item on click { left_selected = item } }
            }
        }
        ListView #right {
            row_height: 40.0
            offset <=> right_offset
            for item in items key item {
                Button { text: "Right " + item on click { right_selected = item } }
            }
        }
    }
}"#;
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let main = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == compiled.roots[0])
        .unwrap();
    let main_id = main.id;
    let left_selected = main.properties[3].id;
    let right_selected = main.properties[4].id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(main_id, []).unwrap();
    let mut app = TestApp::new(runtime);
    app.get_by_role(Role::Button, "Left same").click().unwrap();
    let selected = app.entity().read(|runtime| {
        (
            runtime.instance(root).unwrap().properties[&left_selected]
                .get()
                .clone(),
            runtime.instance(root).unwrap().properties[&right_selected]
                .get()
                .clone(),
        )
    });
    assert_eq!(
        selected,
        (
            DslValue::String("same".into()),
            DslValue::String(String::new())
        )
    );
    app.get_by_role(Role::Button, "Right same").click().unwrap();
    let selected = app.entity().read(|runtime| {
        runtime.instance(root).unwrap().properties[&right_selected]
            .get()
            .clone()
    });
    assert_eq!(selected, DslValue::String("same".into()));
}

#[test]
fn sibling_template_lists_keep_separate_measured_viewport_heights() {
    let source = r#"import { ListView } from "@argui/ui"
import { Column, Text } from "@argui/native"
export component Main {
    in property items: array<string>
    private property upper_offset: float = 0.0
    private property lower_offset: float = 0.0
    Column {
        width: 100%
        height: 100%
        ListView #upper {
            row_height: 20.0
            viewport_height: 0px
            offset <=> upper_offset
            for item in items key item { Text { content: item } }
        }
        ListView #lower {
            row_height: 20.0
            viewport_height: 0px
            offset <=> lower_offset
            for item in items key item { Text { content: item } }
        }
    }
}"#;
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let root_component = compiled.roots[0];
    let items_property = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == root_component)
        .unwrap()
        .properties[0]
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime
        .mount(
            root_component,
            [(
                items_property,
                DslValue::Array(
                    (0..100)
                        .map(|index| DslValue::String(format!("row-{index}")))
                        .collect(),
                ),
            )],
        )
        .unwrap();
    let tree = UiTree::new(runtime.render().unwrap());
    let viewports = tree
        .node_ids()
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let element = tree.element_at(index)?;
            (element.key.as_deref() == Some("virtual_viewport")).then(|| {
                (
                    *node,
                    element
                        .source_identity()
                        .cloned()
                        .expect("retained VirtualWindow identity"),
                )
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(viewports.len(), 2);
    assert_ne!(viewports[0].1, viewports[1].1);
    let snapshot = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(400.0, 400.0)),
        nodes: viewports
            .iter()
            .zip([80.0, 280.0])
            .map(|((node, identity), height)| LayoutBounds {
                node: *node,
                key: Some("virtual_viewport".into()),
                retained_identity: Some(identity.clone()),
                bounds: Rect::new(Point::default(), Size::new(400.0, height)),
            })
            .collect(),
    };
    let mut context = Context::<LiveRuntime>::default();
    Render::layout_changed(&mut runtime, &snapshot, &mut context);
    assert_eq!(context.view_update(), ViewUpdate::Rebuild);
    let root = runtime.render().unwrap();
    let mut upper = Vec::new();
    let mut lower = Vec::new();
    visible_text(&root.children[0], &mut upper);
    visible_text(&root.children[1], &mut lower);
    assert!(
        upper.len() < lower.len(),
        "upper={} lower={}",
        upper.len(),
        lower.len()
    );
    assert!(lower.len() < 100);
}

#[test]
fn virtual_list_rejects_invalid_window_settings_and_recovers() {
    let source = r#"import { VirtualWindow, Text } from "@argui/native"
export component Main {
    private property row_height: float = 20.0
    private property viewport_height: float = 60.0
    private property offset: float = 0.0
    private property overscan: int = 1
    VirtualWindow {
        row_height: row_height
        viewport_height: viewport_height
        offset <=> offset
        overscan: overscan
        for item in ["one", "two"] key item { Text { content: item } }
    }
}"#;
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let component = compiled.roots[0];
    let properties = &compiled
        .ir
        .components
        .iter()
        .find(|definition| definition.id == component)
        .unwrap()
        .properties;
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
        unreachable!();
    };
    let ids = semantic
        .properties
        .iter()
        .zip(properties)
        .map(|(semantic, ir)| (semantic.name.as_str(), ir.id))
        .collect::<HashMap<_, _>>();
    let row_height = ids["row_height"];
    let viewport_height = ids["viewport_height"];
    let offset = ids["offset"];
    let overscan = ids["overscan"];
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(component, []).unwrap();
    assert!(runtime.render().is_ok());

    for (name, property, invalid, valid) in [
        (
            "NaN row height",
            row_height,
            DslValue::Float(f64::NAN),
            DslValue::Float(20.0),
        ),
        (
            "zero row height",
            row_height,
            DslValue::Float(0.0),
            DslValue::Float(20.0),
        ),
        (
            "infinite viewport",
            viewport_height,
            DslValue::Float(f64::INFINITY),
            DslValue::Float(60.0),
        ),
    ] {
        runtime.set_property(root, property, invalid).unwrap();
        let result = runtime.render();
        assert!(
            matches!(&result, Err(RuntimeError::Schema(_))),
            "{name}: {:?}",
            result.err()
        );
        runtime.set_property(root, property, valid).unwrap();
        assert!(runtime.render().is_ok(), "failed to recover from {name}");
    }

    // A nonpositive explicit height selects the measured-height fallback.
    runtime
        .set_property(root, viewport_height, DslValue::Float(-1.0))
        .unwrap();
    assert!(runtime.render().is_ok());
    runtime
        .set_property(root, viewport_height, DslValue::Float(60.0))
        .unwrap();

    runtime
        .set_property(root, offset, DslValue::Float(f64::NAN))
        .unwrap();
    let result = runtime.render();
    assert!(
        matches!(&result, Err(RuntimeError::Schema(_))),
        "nonfinite offset: {:?}",
        result.err()
    );
    runtime
        .set_property(root, offset, DslValue::Float(0.0))
        .unwrap();
    assert!(
        runtime.render().is_ok(),
        "failed to recover from nonfinite offset"
    );

    runtime
        .set_property(root, overscan, DslValue::Int(-1))
        .unwrap();
    let result = runtime.render();
    assert!(
        matches!(&result, Err(RuntimeError::Schema(_))),
        "negative overscan: {:?}",
        result.err()
    );
    runtime
        .set_property(root, overscan, DslValue::Int(0))
        .unwrap();
    let mut text = Vec::new();
    visible_text(&runtime.render().unwrap(), &mut text);
    assert_eq!(text, ["one", "two"]);
}
