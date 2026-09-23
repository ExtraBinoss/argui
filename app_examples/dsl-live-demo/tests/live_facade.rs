#![cfg(feature = "argui-live")]

include!(concat!(env!("OUT_DIR"), "/conformance.rs"));

use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime, RuntimeError};
use std::{cell::Cell, collections::HashMap, rc::Rc};

/// Compiles the shared language fixture used by the generated live facades.
///
/// Returns the compiler artifact; panics if the checked-in fixture stops compiling.
fn compile_fixture() -> CompiledProject {
    Compiler::compile(
        [SourceModule::new(
            "tests/fixtures/language.argui",
            include_str!("fixtures/language.argui"),
        )],
        "tests/fixtures/language.argui",
        |_| Err("fixture has no assets".into()),
    )
    .expect("the live facade fixture must compile")
}

/// Finds a fixture component's stable IR ID by its source name.
///
/// * `project` is the compiled fixture containing the component.
/// * `name` is its DSL declaration name.
///
/// Returns the matching component ID; panics if the fixture lacks that component.
fn component_id(project: &CompiledProject, name: &str) -> argui_dsl_runtime::ir::ComponentId {
    let definition = project
        .semantic
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == name)
        .unwrap_or_else(|| panic!("fixture has no component named `{name}`"));
    argui_dsl_runtime::ir::ComponentId::from_raw(definition.id.raw())
}

/// Creates a runtime for the compiled fixture with the requested ABI hash.
///
/// * `project` supplies the fixture IR and defaults.
/// * `public_api_hash` is the host ABI hash to expose to generated facades.
///
/// Returns an unmounted runtime; panics if package preparation fails.
fn live_runtime(project: &CompiledProject, public_api_hash: u64) -> LiveRuntime {
    let package = LivePackage::prepare(1, public_api_hash, project.ir.clone(), HashMap::new())
        .expect("the fixture should prepare as a live package");
    LiveRuntime::new(package).expect("the fixture runtime should initialize")
}

/// Attaches only to the matching mounted root and the generated public ABI.
#[test]
fn live_facade_attach_checks_root_and_abi() {
    let compiled = compile_fixture();
    assert_eq!(PUBLIC_API_HASH, compiled.public_api_hash);
    let conformance = component_id(&compiled, "Conformance");

    let empty = live_runtime(&compiled, compiled.public_api_hash);
    assert!(matches!(
        ConformanceLive::attach(&empty),
        Err(RuntimeError::IncompatiblePackage(_))
    ));

    let mut matching = live_runtime(&compiled, compiled.public_api_hash);
    matching.mount(conformance, []).unwrap();
    let facade = ConformanceLive::attach(&matching).unwrap();
    assert_eq!(facade.count(&matching).unwrap(), 2);

    let mut wrong_root = live_runtime(&compiled, compiled.public_api_hash);
    wrong_root
        .mount(component_id(&compiled, "NamedSlots"), [])
        .unwrap();
    assert!(matches!(
        ConformanceLive::attach(&wrong_root),
        Err(RuntimeError::IncompatiblePackage(_))
    ));

    let mut wrong_abi = live_runtime(&compiled, compiled.public_api_hash ^ 1);
    wrong_abi.mount(conformance, []).unwrap();
    assert!(matches!(
        ConformanceLive::attach(&wrong_abi),
        Err(RuntimeError::IncompatiblePackage(_))
    ));
}

/// Binds and invokes the fixture's named callback through its typed facade.
#[test]
fn live_facade_binds_named_typed_callback() {
    let compiled = compile_fixture();
    let root = component_id(&compiled, "Conformance");
    let callback = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == root)
        .and_then(|component| component.callbacks.first())
        .expect("Conformance declares probe")
        .id;
    let mut runtime = live_runtime(&compiled, compiled.public_api_hash);
    let instance = runtime.mount(root, []).unwrap();
    let facade = ConformanceLive::attach(&runtime).unwrap();
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);

    facade
        .on_probe(&mut runtime, move || {
            observed.set(observed.get() + 1);
            true
        })
        .unwrap();
    assert_eq!(
        runtime
            .invoke_callback(instance, callback, Vec::new())
            .unwrap(),
        DslValue::Bool(true)
    );
    assert_eq!(calls.get(), 1);
}

/// Commits model row edits through the generated facade and retains row IDs.
#[test]
fn live_facade_model_edits_preserve_ids_and_revisions() {
    let compiled = compile_fixture();
    let root = component_id(&compiled, "EditableModel");
    let rows_property = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == root)
        .and_then(|component| {
            component.properties.iter().find(|property| {
                matches!(
                    &property.value_type,
                    argui_dsl_runtime::ir::IrType::Model(_)
                )
            })
        })
        .expect("EditableModel declares a model property")
        .id;
    let mut runtime = live_runtime(&compiled, compiled.public_api_hash);
    runtime
        .mount(
            root,
            [(
                rows_property,
                DslValue::Array(vec![
                    DslValue::String("first".into()),
                    DslValue::String("second".into()),
                ]),
            )],
        )
        .unwrap();
    let facade = EditableModelLive::attach(&runtime).unwrap();
    let initial = facade.rows(&runtime).unwrap();
    let first_id = initial.row_id(0).unwrap();
    let second_id = initial.row_id(1).unwrap();
    let initial_revision = initial.revision();
    assert_ne!(first_id, second_id);

    let inserted_id = facade
        .rows_insert(&mut runtime, 1, "middle".into())
        .unwrap()
        .unwrap();
    let inserted = facade.rows(&runtime).unwrap();
    assert_eq!(inserted.row_ids(), &[first_id, inserted_id, second_id]);
    assert_eq!(inserted.revision(), initial_revision + 1);
    assert_eq!(&inserted[..], &["first", "middle", "second"]);

    assert!(
        facade
            .rows_update(&mut runtime, 1, |row| {
                *row = "edited".into();
                true
            })
            .unwrap()
    );
    let updated = facade.rows(&runtime).unwrap();
    assert_eq!(updated.row_ids(), &[first_id, inserted_id, second_id]);
    assert_eq!(updated.revision(), initial_revision + 2);
    assert_eq!(updated[1], "edited");

    assert!(facade.rows_move(&mut runtime, 2, 0).unwrap());
    let moved = facade.rows(&runtime).unwrap();
    assert_eq!(moved.row_ids(), &[second_id, first_id, inserted_id]);
    assert_eq!(moved.revision(), initial_revision + 3);

    assert_eq!(
        facade.rows_remove(&mut runtime, 1).unwrap(),
        Some((first_id, "first".into()))
    );
    let removed = facade.rows(&runtime).unwrap();
    assert_eq!(removed.row_ids(), &[second_id, inserted_id]);
    assert_eq!(removed.revision(), initial_revision + 4);
    assert_eq!(&removed[..], &["second", "edited"]);
}
