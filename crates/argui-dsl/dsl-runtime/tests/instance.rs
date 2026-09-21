use std::collections::BTreeMap;

use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_ir::{
    AnimationId, CallbackId, ComponentId, IrType, PropertyId, PropertyTargetId, SiteId,
};
use argui_dsl_runtime::{
    AnimationKey, AnimationStore, ComponentInstance, DslValue, DynamicProperty, InstanceId,
    LivePackage, LiveRuntime, RuntimeError,
};

#[test]
fn animation_store_retargets_updates_and_prunes_stable_slots() {
    let first = AnimationKey {
        instance: InstanceId::from_raw(1),
        component: ComponentId::from_raw(2),
        site: SiteId::from_raw(3),
        property: PropertyTargetId::Component(PropertyId::from_raw(4)),
        animation: AnimationId::from_raw(5),
    };
    let second = AnimationKey {
        site: SiteId::from_raw(6),
        ..first
    };
    let mut store = AnimationStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
    assert!(!store.update(first, 9.0, 1.0));
    {
        let state = store.retarget(first, 2.0, 10);
        assert_eq!(state.value, 2.0);
        assert_eq!(state.velocity, 0.0);
        assert_eq!(state.specification, 10);
        state.velocity = 3.5;
    }
    assert_eq!(store.get(first).unwrap().velocity, 3.5);
    assert_eq!(store.retarget(first, 99.0, 11).value, 2.0);
    assert_eq!(store.get(first).unwrap().specification, 11);
    assert!(store.update(first, 7.0, -2.0));
    assert_eq!(store.get(first).unwrap().value, 7.0);
    assert_eq!(store.get(first).unwrap().velocity, -2.0);
    store.retarget(second, 4.0, 12);
    assert_eq!(store.len(), 2);
    store.retain(|key| *key == second);
    assert_eq!(store.get(first), None);
    assert_eq!(store.get(second).unwrap().value, 4.0);
    store.retain(|_| false);
    assert!(store.is_empty());
}

#[test]
fn dynamic_property_accepts_compatible_values_and_preserves_state_on_errors() {
    let id = PropertyId::from_raw(9);
    let mut property = DynamicProperty::new(id, IrType::Int, DslValue::Int(2));
    assert_eq!(property.id, id);
    assert_eq!(property.get(), &DslValue::Int(2));
    assert_eq!(property.revision(), 0);
    assert!(!property.set(DslValue::Int(2)).unwrap());
    assert_eq!(property.revision(), 0);
    assert!(property.set(DslValue::Int(5)).unwrap());
    assert_eq!(property.get(), &DslValue::Int(5));
    assert_eq!(property.revision(), 1);
    assert_eq!(
        property.set(DslValue::String("wrong".into())).unwrap_err(),
        RuntimeError::TypeMismatch {
            expected: "Int".into(),
            actual: "string".into(),
        }
    );
    assert_eq!(property.get(), &DslValue::Int(5));
    assert_eq!(property.revision(), 1);
    let mut float = DynamicProperty::new(PropertyId::from_raw(10), IrType::Float, DslValue::Int(1));
    assert!(float.set(DslValue::Int(2)).unwrap());
    assert_eq!(float.get(), &DslValue::Int(2));
    let mut optional = DynamicProperty::new(
        PropertyId::from_raw(11),
        IrType::Optional(Box::new(IrType::String)),
        DslValue::Null,
    );
    assert!(optional.set(DslValue::String("value".into())).unwrap());
}

#[test]
fn runtime_error_display_is_stable_for_each_wire_failure() {
    let errors = [
        (
            RuntimeError::MissingComponent(1),
            "component 1 is unavailable",
        ),
        (
            RuntimeError::MissingProperty(2),
            "property 2 is unavailable",
        ),
        (
            RuntimeError::MissingExpression(3),
            "expression 3 is unavailable",
        ),
        (RuntimeError::MissingAsset(4), "asset 4 is unavailable"),
        (
            RuntimeError::TypeMismatch {
                expected: "int".into(),
                actual: "string".into(),
            },
            "expected int, found string",
        ),
        (
            RuntimeError::InvalidBytecode("broken".into()),
            "invalid bytecode: broken",
        ),
        (
            RuntimeError::InvalidShader("bad wgsl".into()),
            "invalid shader: bad wgsl",
        ),
        (
            RuntimeError::IncompatiblePackage("wrong version".into()),
            "incompatible live package: wrong version",
        ),
        (RuntimeError::Asset("missing".into()), "asset: missing"),
        (
            RuntimeError::Schema("unknown".into()),
            "native schema: unknown",
        ),
        (
            RuntimeError::RestartRequired {
                previous: 0x12,
                next: 0x34,
            },
            "public UI ABI changed from 0000000000000012 to 0000000000000034; restart required",
        ),
    ];
    for (error, expected) in errors {
        assert_eq!(error.to_string(), expected);
        assert_eq!(format!("{error}"), expected);
    }
}

#[test]
fn value_shapes_compare_and_debug_as_expected() {
    let values = vec![
        DslValue::Null,
        DslValue::Bool(false),
        DslValue::Int(-2),
        DslValue::Float(1.5),
        DslValue::String("text".into()),
        DslValue::Color(argui_core::Color::TRANSPARENT),
        DslValue::Struct(BTreeMap::new()),
        DslValue::Enum {
            symbol: 1,
            variant: 2,
        },
        DslValue::Array(vec![DslValue::Int(1)]),
        DslValue::Asset(argui_dsl_ir::AssetId::from_raw(8)),
    ];
    assert_eq!(values.len(), 10);
    assert_eq!(values[2], DslValue::Int(-2));
    assert_ne!(values[3], DslValue::Float(2.5));
    assert!(format!("{:?}", values).contains("Asset"));
}

/// Verifies that instance IDs preserve their raw value and total ordering.
#[test]
fn instance_ids_round_trip_and_order() {
    let first = InstanceId::from_raw(7);
    let second = InstanceId::from_raw(8);

    assert_eq!(first.raw(), 7);
    assert_eq!(second.raw(), 8);
    assert!(first < second);
    assert_ne!(first, second);
}

/// Verifies that a live instance indexes duplicate properties by their final definition.
#[test]
fn component_instance_keeps_latest_property_and_reports_callback_state() {
    let property = PropertyId::from_raw(12);
    let mut instance = ComponentInstance::new(
        InstanceId::from_raw(4),
        ComponentId::from_raw(9),
        [
            DynamicProperty::new(property, IrType::Int, DslValue::Int(1)),
            DynamicProperty::new(property, IrType::Int, DslValue::Int(2)),
        ],
    );

    assert_eq!(instance.id, InstanceId::from_raw(4));
    assert_eq!(instance.component, ComponentId::from_raw(9));
    assert_eq!(instance.properties.len(), 1);
    assert_eq!(instance.properties[&property].get(), &DslValue::Int(2));
    assert!(format!("{instance:?}").contains("callback_count: 0"));

    instance.bind_callback(CallbackId::from_raw(13), |_| DslValue::Null);
    assert!(format!("{instance:?}").contains("callback_count: 1"));
}

/// Compiles a component whose visual expression invokes a runtime callback.
fn callback_expression_package() -> (ComponentId, CallbackId, LivePackage) {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/ui"
export component Main {
    callback changed(value: string) -> string
    Text { content: changed("from expression") }
}
"#,
        )],
        "ui/main.argui",
        |_| Err("instance test source has no assets".into()),
    )
    .expect("callback expression source should compile");
    let (component, callback) = compiled
        .ir
        .components
        .iter()
        .find_map(|component| {
            component
                .callbacks
                .first()
                .map(|callback| (component.id, callback.id))
        })
        .expect("compiled component should expose its callback");
    let package = LivePackage::prepare(
        1,
        compiled.public_api_hash,
        compiled.ir,
        std::collections::HashMap::new(),
    )
    .expect("compiled IR should form a live package");
    (component, callback, package)
}

/// Verifies callback binding, callback expression dispatch, and cache invalidation after mounting.
#[test]
fn component_instance_callback_state_is_used_by_rendering() {
    let (component, callback, package) = callback_expression_package();
    let mut runtime = LiveRuntime::new(package).expect("package should initialize");
    let root = runtime
        .mount(component, [])
        .expect("component should mount");

    runtime.set_translator(|_| Some("translated".into()));
    runtime.clear_translator();
    runtime
        .bind_callback(root, callback, |arguments| {
            arguments.into_iter().next().unwrap_or(DslValue::Null)
        })
        .expect("mounted callback should bind");

    runtime
        .render()
        .expect("bound callback expression should render");
    assert_eq!(
        runtime.invoke_callback(root, callback, vec![DslValue::String("from host".into())],),
        Ok(DslValue::String("from host".into()))
    );
}

mod runtime_edges {
    use std::collections::HashMap;

    use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
    use argui_dsl_ir::{CallbackId, ComponentId, PropertyId};
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime, RuntimeError};
    use argui_dsl_semantic::DefinitionKind;
    use argui_testing::TestApp;

    /// Compiles one in-memory runtime fixture without loading external assets.
    fn compile(source: &str) -> CompiledProject {
        Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("runtime edge tests do not load external assets".into()),
        )
        .unwrap()
    }

    /// Resolves the exported component and its stable IDs from a compiled fixture.
    fn component(compiled: &CompiledProject) -> ComponentId {
        let definition = compiled
            .semantic
            .modules
            .iter()
            .filter(|module| module.path != "@argui/ui")
            .flat_map(|module| &module.definitions)
            .find(|definition| {
                definition.name == "Main" && matches!(definition.kind, DefinitionKind::Component(_))
            })
            .unwrap();
        compiled
            .ir
            .components
            .iter()
            .find(|candidate| candidate.id.raw() == definition.id.raw())
            .map(|candidate| candidate.id)
            .unwrap()
    }

    /// Resolves one component property by source name.
    fn property(compiled: &CompiledProject, name: &str) -> PropertyId {
        let definition = compiled
            .semantic
            .modules
            .iter()
            .filter(|module| module.path != "@argui/ui")
            .flat_map(|module| &module.definitions)
            .find(|definition| definition.name == "Main")
            .unwrap();
        let DefinitionKind::Component(semantic) = &definition.kind else {
            unreachable!();
        };
        let id = component(compiled);
        let ir = compiled
            .ir
            .components
            .iter()
            .find(|candidate| candidate.id == id)
            .unwrap();
        semantic
            .properties
            .iter()
            .zip(&ir.properties)
            .find(|(property, _)| property.name == name)
            .map(|(_, property)| property.id)
            .unwrap()
    }

    #[test]
    fn runtime_theme_translation_and_callback_lifecycle_is_publicly_observable() {
        let compiled = compile(
            r#"import { Text } from "@argui/ui"
export theme Palette {
    --accent: color = #123456
    dark { --accent: #abcdef }
}
export component Main {
    callback changed(value: string) -> string
    Text { content: tr("title") background: var(--accent) }
}
"#,
        );
        let main = component(&compiled);
        let callback = compiled
            .ir
            .components
            .iter()
            .find(|candidate| candidate.id == main)
            .and_then(|candidate| candidate.callbacks.first())
            .map(|callback| callback.id)
            .unwrap();
        let mode = compiled.ir.themes[0].modes[0].id;
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.set_translator(|id| (id == "title").then_some("localized".into()));
        let root = runtime.mount(main, []).unwrap();
        runtime.render().unwrap();

        runtime.set_theme_mode(mode).unwrap();
        runtime.render().unwrap();
        runtime.clear_translator();
        assert!(runtime.last_error().is_none());
        runtime
            .bind_callback(root, callback, |arguments| {
                arguments.into_iter().next().unwrap()
            })
            .unwrap();
        assert_eq!(
            runtime.invoke_callback(root, callback, vec![DslValue::String("callback".into())]),
            Ok(DslValue::String("callback".into()))
        );
        assert!(matches!(
            runtime.invoke_callback(root, CallbackId::from_raw(99), Vec::new()),
            Err(RuntimeError::InvalidBytecode(message)) if message.contains("not bound")
        ));
        assert_eq!(runtime.take_event_error(), None);
        assert_eq!(runtime.last_client_event(), None);

        TestApp::new(runtime).assert_text("title");
    }

    #[test]
    fn property_updates_are_transactional_for_same_and_different_values() {
        let compiled = compile(
            r#"export component Main {
    private property count: int = 2
}
"#,
        );
        let main = component(&compiled);
        let count = property(&compiled, "count");
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(main, []).unwrap();
        assert!(!runtime.set_property(root, count, DslValue::Int(2)).unwrap());
        assert!(runtime.set_property(root, count, DslValue::Int(4)).unwrap());
        assert_eq!(
            runtime.instance(root).unwrap().properties[&count].get(),
            &DslValue::Int(4)
        );
    }
}
