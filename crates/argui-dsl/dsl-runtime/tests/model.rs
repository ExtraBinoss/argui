use std::collections::HashMap;

use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
use argui_dsl_ir::{ComponentId, PropertyId};
use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime, RuntimeError};
use argui_dsl_semantic::DefinitionKind;

/// Compiles one model fixture without external assets.
///
/// # Panics
///
/// Panics if the test fixture is rejected by the compiler.
fn compile(source: &str) -> CompiledProject {
    Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("model tests do not load external assets".into()),
    )
    .unwrap()
}

struct MainMembers {
    component: ComponentId,
    properties: HashMap<String, PropertyId>,
}

/// Finds component and property IDs for the exported `Main` fixture.
///
/// # Panics
///
/// Panics if `Main` or one of its lowered declarations is absent.
fn members(compiled: &CompiledProject) -> MainMembers {
    let definition = compiled
        .semantic
        .modules
        .iter()
        .filter(|module| !module.path.starts_with("@argui/ui/"))
        .flat_map(|module| &module.definitions)
        .find(|definition| {
            definition.name == "Main" && matches!(definition.kind, DefinitionKind::Component(_))
        })
        .unwrap();
    let DefinitionKind::Component(semantic) = &definition.kind else {
        unreachable!();
    };
    let component = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == definition.id.raw())
        .unwrap();
    MainMembers {
        component: component.id,
        properties: semantic
            .properties
            .iter()
            .zip(&component.properties)
            .map(|(semantic, ir)| (semantic.name.clone(), ir.id))
            .collect(),
    }
}

/// Prepares a generation and its fixture IDs.
///
/// # Panics
///
/// Panics if compilation or live package preparation fails.
fn package(generation: u64, source: &str) -> (LivePackage, MainMembers, u64) {
    let compiled = compile(source);
    let members = members(&compiled);
    let public_api_hash = compiled.public_api_hash;
    let package =
        LivePackage::prepare(generation, public_api_hash, compiled.ir, HashMap::new()).unwrap();
    (package, members, public_api_hash)
}

/// Uses an internal default change to create a compatible reload generation.
fn main_source(internal_default: &str) -> String {
    format!(
        "export component Main {{ in property rows: model<int> private property note: string = \"{internal_default}\" }}"
    )
}

/// Model row IDs survive edits, retain revisions, and migrate unchanged across reload.
#[test]
fn model_row_identity_and_revision_survive_edits_and_reload() {
    let (first, old, old_hash) = package(1, &main_source("before"));
    let (next, new, next_hash) = package(2, &main_source("after"));
    assert_eq!(
        old_hash, next_hash,
        "the private default preserves public ABI"
    );
    let rows = old.properties["rows"];
    let mut runtime = LiveRuntime::new(first).unwrap();
    let root = runtime
        .mount(
            old.component,
            [(
                rows,
                DslValue::Array(vec![
                    DslValue::Int(10),
                    DslValue::Int(20),
                    DslValue::Int(30),
                ]),
            )],
        )
        .unwrap();
    let initial_revision = runtime.instance(root).unwrap().properties[&rows].revision();
    let first_id = runtime.model_row(root, rows, 0).unwrap().unwrap().0;
    let second_id = runtime.model_row(root, rows, 1).unwrap().unwrap().0;
    let third_id = runtime.model_row(root, rows, 2).unwrap().unwrap().0;

    let inserted_id = runtime
        .model_insert(root, rows, 1, DslValue::Int(15))
        .unwrap()
        .unwrap();
    assert_eq!(
        runtime.instance(root).unwrap().properties[&rows].revision(),
        initial_revision + 1
    );
    assert_eq!(
        runtime.model_row(root, rows, 0).unwrap(),
        Some((first_id, DslValue::Int(10)))
    );
    assert_eq!(
        runtime.model_row(root, rows, 2).unwrap(),
        Some((second_id, DslValue::Int(20)))
    );
    assert_eq!(
        runtime.model_row(root, rows, 3).unwrap(),
        Some((third_id, DslValue::Int(30)))
    );

    assert!(
        runtime
            .model_update(root, rows, 2, DslValue::Int(25))
            .unwrap()
    );
    assert_eq!(
        runtime.model_row(root, rows, 2).unwrap(),
        Some((second_id, DslValue::Int(25)))
    );
    assert_eq!(
        runtime.instance(root).unwrap().properties[&rows].revision(),
        initial_revision + 2
    );

    assert!(runtime.model_move(root, rows, 3, 0).unwrap());
    assert_eq!(
        runtime.model_row(root, rows, 0).unwrap(),
        Some((third_id, DslValue::Int(30)))
    );
    assert_eq!(
        runtime.instance(root).unwrap().properties[&rows].revision(),
        initial_revision + 3
    );

    assert_eq!(
        runtime.model_remove(root, rows, 2).unwrap(),
        Some((inserted_id, DslValue::Int(15)))
    );
    assert_eq!(
        runtime.model_row(root, rows, 0).unwrap(),
        Some((third_id, DslValue::Int(30)))
    );
    assert_eq!(
        runtime.model_row(root, rows, 1).unwrap(),
        Some((first_id, DslValue::Int(10)))
    );
    assert_eq!(
        runtime.model_row(root, rows, 2).unwrap(),
        Some((second_id, DslValue::Int(25)))
    );
    let edited_revision = runtime.instance(root).unwrap().properties[&rows].revision();
    assert_eq!(edited_revision, initial_revision + 4);

    let outcome = runtime.commit_reload(runtime.prepare_reload(next).unwrap());
    assert_eq!(outcome.generation, 2);
    assert_eq!(outcome.migrated_instances, 1);
    let rows = new.properties["rows"];
    assert_eq!(
        runtime.instance(root).unwrap().properties[&rows].revision(),
        edited_revision
    );
    assert_eq!(
        runtime.model_row(root, rows, 0).unwrap(),
        Some((third_id, DslValue::Int(30)))
    );
    assert_eq!(
        runtime.model_row(root, rows, 1).unwrap(),
        Some((first_id, DslValue::Int(10)))
    );
    assert_eq!(
        runtime.model_row(root, rows, 2).unwrap(),
        Some((second_id, DslValue::Int(25)))
    );
}

/// Model mutations reject `array<T>` properties, even though both store array values.
#[test]
fn model_mutations_reject_array_properties() {
    let source =
        "export component Main { in property rows: model<int> in property values: array<int> }";
    let (package, members, _) = package(1, source);
    let rows = members.properties["rows"];
    let values = members.properties["values"];
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime
        .mount(
            members.component,
            [
                (
                    rows,
                    DslValue::Array(vec![DslValue::Int(1), DslValue::Int(2)]),
                ),
                (
                    values,
                    DslValue::Array(vec![DslValue::Int(1), DslValue::Int(2)]),
                ),
            ],
        )
        .unwrap();
    let revision = runtime.instance(root).unwrap().properties[&values].revision();

    assert!(matches!(
        runtime.model_insert(root, values, 1, DslValue::Int(3)),
        Err(RuntimeError::TypeMismatch { .. })
    ));
    assert!(matches!(
        runtime.model_remove(root, values, 0),
        Err(RuntimeError::TypeMismatch { .. })
    ));
    assert!(matches!(
        runtime.model_move(root, values, 0, 1),
        Err(RuntimeError::TypeMismatch { .. })
    ));
    assert!(matches!(
        runtime.model_update(root, values, 0, DslValue::Int(3)),
        Err(RuntimeError::TypeMismatch { .. })
    ));
    assert_eq!(
        runtime.instance(root).unwrap().properties[&values].revision(),
        revision
    );
    assert_eq!(
        runtime.model_row(root, rows, 0).unwrap(),
        Some((1, DslValue::Int(1)))
    );
}

/// Reading a row from an ordinary array reports that it is not a model.
#[test]
fn model_row_rejects_array_properties() {
    let source = "export component Main { in property values: array<int> }";
    let (package, members, _) = package(1, source);
    let values = members.properties["values"];
    let mut runtime = LiveRuntime::new(package).unwrap();
    let root = runtime
        .mount(
            members.component,
            [(values, DslValue::Array(vec![DslValue::Int(1)]))],
        )
        .unwrap();

    assert!(matches!(
        runtime.model_row(root, values, 0),
        Err(RuntimeError::TypeMismatch { .. })
    ));
}
