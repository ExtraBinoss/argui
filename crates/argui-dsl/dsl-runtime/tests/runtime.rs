use std::{
    cell::Cell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
use argui_dsl_ir::{CallbackId, ComponentId, PropertyId};
use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime, RuntimeError};
use argui_dsl_semantic::DefinitionKind;
use argui_testing::TestApp;
use argui_ui::Role;

#[path = "runtime/theme.rs"]
mod theme_mode;

#[path = "runtime/reload.rs"]
mod reload;

fn compile(source: &str) -> CompiledProject {
    Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("test source has no assets".into()),
    )
    .unwrap()
}

fn package(generation: u64, source: &str) -> (LivePackage, Members) {
    let compiled = compile(source);
    let members = members(&compiled, "Main");
    let package = LivePackage::prepare(
        generation,
        compiled.public_api_hash,
        compiled.ir,
        HashMap::new(),
    )
    .unwrap();
    (package, members)
}

fn members(compiled: &CompiledProject, name: &str) -> Members {
    let definition = compiled
        .semantic
        .modules
        .iter()
        .filter(|module| !module.path.starts_with("@argui/ui/"))
        .flat_map(|module| &module.definitions)
        .find(|definition| {
            definition.name == name && matches!(definition.kind, DefinitionKind::Component(_))
        })
        .unwrap();
    let semantic = match &definition.kind {
        DefinitionKind::Component(component) => component,
        _ => unreachable!(),
    };
    let component = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == definition.id.raw())
        .unwrap();
    Members {
        component: component.id,
        properties: semantic
            .properties
            .iter()
            .zip(&component.properties)
            .map(|(semantic, ir)| (semantic.name.clone(), ir.id))
            .collect(),
        callbacks: semantic
            .callbacks
            .iter()
            .zip(&component.callbacks)
            .map(|(semantic, ir)| (semantic.name.clone(), ir.id))
            .collect(),
    }
}

struct Members {
    component: ComponentId,
    properties: HashMap<String, PropertyId>,
    callbacks: HashMap<String, CallbackId>,
}

fn migration_source(private_properties: &str) -> String {
    format!(
        r#"import {{ Text }} from "@argui/ui"
export component Main {{
    in property title: string
    {private_properties}
    callback save()
    Text {{ content: title }}
}}"#
    )
}

/// A changed authored default refreshes untouched live state and reaches the rendered tree.
#[test]
fn reload_uses_new_default_for_an_unmodified_input_property() {
    let source = |label: &str| {
        format!(
            "import {{ Text }} from \"@argui/ui\" export component Main {{ in property title: string = \"{label}\" Text {{ content: title }} }}"
        )
    };
    let (first, old) = package(1, &source("before"));
    let (second, new) = package(2, &source("after"));
    let mut runtime = LiveRuntime::new(first).unwrap();
    let root = runtime.mount(old.component, []).unwrap();
    assert_eq!(
        runtime.instance(root).unwrap().properties[&old.properties["title"]].revision(),
        0
    );
    let _ = runtime.commit_reload(runtime.prepare_reload(second).unwrap());
    assert_eq!(
        runtime.instance(root).unwrap().properties[&new.properties["title"]].get(),
        &DslValue::String("after".into())
    );
    TestApp::new(runtime).assert_text("after");
}

/// A host-supplied input remains authoritative when an authored default changes.
#[test]
fn reload_preserves_explicitly_supplied_input_property() {
    let source = |label: &str| {
        format!(
            "import {{ Text }} from \"@argui/ui\" export component Main {{ in property title: string = \"{label}\" Text {{ content: title }} }}"
        )
    };
    let (first, old) = package(1, &source("before"));
    let (second, new) = package(2, &source("after"));
    let mut runtime = LiveRuntime::new(first).unwrap();
    let root = runtime
        .mount(
            old.component,
            [(
                old.properties["title"],
                DslValue::String("from host".into()),
            )],
        )
        .unwrap();
    assert_eq!(
        runtime.instance(root).unwrap().properties[&old.properties["title"]].revision(),
        1
    );
    let _ = runtime.commit_reload(runtime.prepare_reload(second).unwrap());
    assert_eq!(
        runtime.instance(root).unwrap().properties[&new.properties["title"]].get(),
        &DslValue::String("from host".into())
    );
    TestApp::new(runtime).assert_text("from host");
}

/// An explicit input remains authoritative even when equal to the old default.
#[test]
fn reload_preserves_explicit_input_equal_to_old_default() {
    let source = |label: &str| {
        format!(
            "import {{ Text }} from \"@argui/ui\" export component Main {{ in property title: string = \"{label}\" Text {{ content: title }} }}"
        )
    };
    let (first, old) = package(1, &source("before"));
    let (second, new) = package(2, &source("after"));
    let mut runtime = LiveRuntime::new(first).unwrap();
    let root = runtime
        .mount(
            old.component,
            [(old.properties["title"], DslValue::String("before".into()))],
        )
        .unwrap();
    assert_eq!(
        runtime.instance(root).unwrap().properties[&old.properties["title"]].revision(),
        0
    );
    let _ = runtime.commit_reload(runtime.prepare_reload(second).unwrap());
    assert_eq!(
        runtime.instance(root).unwrap().properties[&new.properties["title"]].get(),
        &DslValue::String("before".into())
    );
    TestApp::new(runtime).assert_text("before");
}

/// Percentage dimensions use the same 0–1 engine scale in live and generated UI.
#[test]
fn live_percentage_width_uses_css_scale() {
    let source =
        "import { Column } from \"@argui/ui\" export component Main { Column { width: 65% } }";
    let (package, members) = package(1, source);
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(members.component, []).unwrap();
    let element = runtime.render().unwrap();
    assert_eq!(element.style.size.width, argui_ui::percent(0.65));
}

#[test]
fn reload_migrates_compatible_state_callbacks_and_new_defaults() {
    let first = migration_source(
        "private property draft: string = \"initial\"\nprivate property removed: int = 7",
    );
    let second = migration_source(
        "private property added: bool = true\nprivate property draft: string = \"replacement\"",
    );
    let (first, old) = package(1, &first);
    let (second, new) = package(2, &second);
    let mut runtime = LiveRuntime::new(first).unwrap();
    let root = runtime.mount(old.component, []).unwrap();
    runtime
        .set_property(
            root,
            old.properties["draft"],
            DslValue::String("edited".into()),
        )
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    runtime
        .bind_callback(root, old.callbacks["save"], move |_| {
            observed.set(observed.get() + 1);
            DslValue::Null
        })
        .unwrap();

    let outcome = runtime.commit_reload(runtime.prepare_reload(second).unwrap());
    assert_eq!(outcome.migrated_instances, 1);
    let instance = runtime.instance(root).unwrap();
    assert_eq!(
        instance.properties[&new.properties["draft"]].get(),
        &DslValue::String("edited".into())
    );
    assert_eq!(
        instance.properties[&new.properties["added"]].get(),
        &DslValue::Bool(true)
    );
    assert!(!instance.properties.contains_key(&old.properties["removed"]));
    runtime
        .invoke_callback(root, new.callbacks["save"], Vec::new())
        .unwrap();
    assert_eq!(calls.get(), 1);
}

#[test]
fn incompatible_private_state_resets_but_public_abi_change_requires_restart() {
    let (first, old) = package(
        1,
        &migration_source("private property draft: string = \"initial\""),
    );
    let (incompatible, new) = package(2, &migration_source("private property draft: int = 42"));
    let mut runtime = LiveRuntime::new(first).unwrap();
    let root = runtime.mount(old.component, []).unwrap();
    runtime
        .set_property(
            root,
            old.properties["draft"],
            DslValue::String("edited".into()),
        )
        .unwrap();
    let prepared = runtime.prepare_reload(incompatible).unwrap();
    let _ = runtime.commit_reload(prepared);
    assert_eq!(
        runtime.instance(root).unwrap().properties[&new.properties["draft"]].get(),
        &DslValue::Int(42)
    );

    let (changed_public_api, _) = package(
        3,
        r#"import { Text } from "@argui/ui"
export component Main {
    in property title: int
    callback save()
    Text { content: "restart" }
}"#,
    );
    assert!(matches!(
        runtime.prepare_reload(changed_public_api),
        Err(RuntimeError::RestartRequired { .. })
    ));
}

#[test]
fn failed_package_preparation_cannot_replace_the_active_generation() {
    let (first, members) = package(1, &migration_source(""));
    let mut runtime = LiveRuntime::new(first).unwrap();
    runtime.mount(members.component, []).unwrap();
    let mut invalid_ir = runtime.ir().clone();
    invalid_ir.assets.push(argui_dsl_ir::IrAsset {
        id: argui_dsl_ir::AssetId::from_raw(99),
        path: "missing.png".into(),
        kind: argui_dsl_ir::AssetKind::Image,
        inline_bytes: None,
    });
    let invalid = LivePackage::prepare(2, runtime.public_api_hash(), invalid_ir, HashMap::new());
    assert!(matches!(invalid, Err(RuntimeError::MissingAsset(99))));
    assert_eq!(runtime.generation(), 1);
}

#[test]
fn nested_callback_routes_mutations_and_external_business_callbacks() {
    let source = r#"import { Text } from "@argui/ui"
component Child {
    callback fire()
    Text { content: "child" }
}
export component Main {
    private property count: int = 0
    callback changed()
    Child { on fire { count += 1 changed() } }
}"#;
    let compiled = compile(source);
    let parent = members(&compiled, "Main");
    let child = members(&compiled, "Child");
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(parent.component, []).unwrap();
    let changes = Rc::new(Cell::new(0));
    let observed = Rc::clone(&changes);
    runtime
        .bind_callback(root, parent.callbacks["changed"], move |_| {
            observed.set(observed.get() + 1);
            DslValue::Null
        })
        .unwrap();
    runtime.render().unwrap();
    let child_instance = runtime
        .inspect()
        .instances
        .into_iter()
        .find(|instance| instance.component == child.component)
        .unwrap()
        .id;
    runtime
        .invoke_callback(child_instance, child.callbacks["fire"], Vec::new())
        .unwrap();
    assert_eq!(changes.get(), 1);
    assert_eq!(
        runtime.instance(root).unwrap().properties[&parent.properties["count"]].get(),
        &DslValue::Int(1)
    );
}

#[test]
fn two_way_component_binding_propagates_child_edits_to_parent() {
    let source = r#"import { Text } from "@argui/ui"
component Editor {
    in-out property value: string = ""
    Text { content: value }
}
export component Main {
    private property draft: string = "initial"
    Editor { value <=> draft }
}"#;
    let compiled = compile(source);
    let parent = members(&compiled, "Main");
    let child = members(&compiled, "Editor");
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(parent.component, []).unwrap();
    runtime.render().unwrap();
    let child_instance = runtime
        .inspect()
        .instances
        .iter()
        .find(|instance| instance.component == child.component)
        .unwrap()
        .id;
    runtime
        .set_property(
            child_instance,
            child.properties["value"],
            DslValue::String("edited".into()),
        )
        .unwrap();
    assert_eq!(
        runtime.instance(root).unwrap().properties[&parent.properties["draft"]].get(),
        &DslValue::String("edited".into())
    );
}

#[test]
fn keyed_repeater_instances_survive_reorder_and_unrelated_reload_edits() {
    fn source(extra: &str) -> String {
        format!(
            r#"import {{ Column, Text }} from "@argui/ui"
export struct Item {{ id: int label: string }}
component Card {{
    in property item: Item
    Text {{ content: item.label }}
}}
export component Main {{
    in property items: model<Item>
    Column {{
        {extra}
        for item in items key item.id {{ Card {{ item: item }} }}
    }}
}}"#
        )
    }

    let before = compile(&source(""));
    let after = compile(&source("Text { content: \"Unrelated\" }"));
    let parent = members(&before, "Main");
    let card = members(&before, "Card");
    let fields = before.ir.structs[0]
        .fields
        .iter()
        .map(|field| field.id)
        .collect::<Vec<_>>();
    let item = |id, label: &str| {
        DslValue::Struct(BTreeMap::from([
            (fields[0], DslValue::Int(id)),
            (fields[1], DslValue::String(label.into())),
        ]))
    };
    let package =
        LivePackage::prepare(1, before.public_api_hash, before.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime
        .mount(
            parent.component,
            [(
                parent.properties["items"],
                DslValue::Array(vec![item(1, "one"), item(2, "two")]),
            )],
        )
        .unwrap();
    runtime.render().unwrap();
    let card_ids = |runtime: &LiveRuntime| {
        runtime
            .inspect()
            .instances
            .into_iter()
            .filter(|instance| instance.component == card.component)
            .map(|instance| instance.id)
            .collect::<std::collections::BTreeSet<_>>()
    };
    let initial = card_ids(&runtime);
    runtime
        .set_property(
            root,
            parent.properties["items"],
            DslValue::Array(vec![item(2, "two"), item(1, "one")]),
        )
        .unwrap();
    runtime.render().unwrap();
    assert_eq!(card_ids(&runtime), initial);

    let next = LivePackage::prepare(2, after.public_api_hash, after.ir, HashMap::new()).unwrap();
    let prepared = runtime.prepare_reload(next).unwrap();
    let _ = runtime.commit_reload(prepared);
    runtime.render().unwrap();
    assert_eq!(card_ids(&runtime), initial);

    runtime
        .set_property(
            root,
            parent.properties["items"],
            DslValue::Array(vec![item(2, "two")]),
        )
        .unwrap();
    runtime.render().unwrap();
    assert_eq!(card_ids(&runtime).len(), 1);
}

#[test]
fn native_button_and_input_events_mutate_live_properties() {
    let source = r#"import { Button, Column, Input, Text } from "@argui/ui"
export component Main {
    private property count: int = 0
    private property name: string = ""
    Column {
        Button #increment { text: "Increment" on click { count += 1 } }
        Input #name { value <=> name placeholder: "Name" label: "Name" }
        Text { text: "Ready" }
    }
}"#;
    let compiled = compile(source);
    assert!(compiled.rust.contains("UiEventKind::TextEdited"));
    let members = members(&compiled, "Main");
    let count = members.properties["count"];
    let name = members.properties["name"];
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime.mount(members.component, []).unwrap();
    let mut app = TestApp::new(runtime);

    app.get_by_role(Role::Button, "Increment").click().unwrap();
    app.get_by_role(Role::TextInput, "Name")
        .type_text("Ada")
        .unwrap();

    app.entity().read(|runtime| {
        assert_eq!(runtime.last_error(), None);
        assert_eq!(
            runtime.instance(root).unwrap().properties[&count].get(),
            &DslValue::Int(1)
        );
        assert_eq!(
            runtime.instance(root).unwrap().properties[&name].get(),
            &DslValue::String("Ada".into())
        );
    });
}
