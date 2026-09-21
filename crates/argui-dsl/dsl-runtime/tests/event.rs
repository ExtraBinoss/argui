//! Runtime event dispatch and related render-state behavior.

mod event_behavior {
    use std::collections::HashMap;

    use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
    use argui_dsl_ir::{ComponentId, IrNode, PropertyId};
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
    use argui_dsl_semantic::DefinitionKind;

    fn compile(source: &str) -> CompiledProject {
        Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("tests do not load external assets".into()),
        )
        .unwrap()
    }

    struct Members {
        component: ComponentId,
        properties: HashMap<String, PropertyId>,
    }

    fn members(compiled: &CompiledProject) -> Members {
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
        let DefinitionKind::Component(semantic) = &definition.kind else {
            unreachable!();
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
        }
    }

    #[test]
    fn native_handler_executes_all_property_assignment_operators_and_returns_value() {
        let source = r#"import { Pressable } from "@argui/native"
export component Main {
    private property count: int = 10
    private property ratio: float = 8.0
    private property text: string = "start"
    Pressable {
        label: "Go"
        on click {
            count = 4
            count += 2
            count -= 1
            count *= 3
            count /= 5
            ratio += 1.5
            ratio -= 0.5
            ratio *= 2.0
            ratio /= 2.0
            text = "a"
            text += "b"
            return count
        }
    }
}
"#;
        let compiled = compile(source);
        let members = members(&compiled);
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(members.component, []).unwrap();
        runtime.render().unwrap();
        let definition = runtime
            .ir()
            .components
            .iter()
            .find(|component| component.id == members.component)
            .unwrap();
        let site = match &definition.body[0] {
            IrNode::Element { site, .. } => *site,
            _ => panic!("expected Pressable element"),
        };
        let result = runtime
            .dispatch_native_event(root, site, argui_schema::builtin::CLICK)
            .unwrap();
        assert_eq!(result, DslValue::Int(3));
        let instance = runtime.instance(root).unwrap();
        assert_eq!(
            instance.properties[&members.properties["count"]].get(),
            &DslValue::Int(3)
        );
        assert_eq!(
            instance.properties[&members.properties["ratio"]].get(),
            &DslValue::Float(9.0)
        );
        assert_eq!(
            instance.properties[&members.properties["text"]].get(),
            &DslValue::String("ab".into())
        );
    }

    #[test]
    fn native_handler_resolves_translations_theme_tokens_and_callbacks() {
        let source = r#"import { Pressable } from "@argui/native"
export theme Palette { --accent: color = #123456 }
export component Main {
    private property title: string = "initial"
    private property color: color = #000000
    callback changed(value: string) -> string
    Pressable {
        label: "Go"
        background: var(--accent)
        on click {
            title = tr("title")
            color = var(--accent)
            return changed(title)
        }
    }
}
"#;
        let compiled = compile(source);
        let members = members(&compiled);
        let callback = compiled
            .ir
            .components
            .iter()
            .find(|component| component.id == members.component)
            .and_then(|component| component.callbacks.first())
            .map(|callback| callback.id)
            .expect("the component declares one callback");
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.set_translator(|id| (id == "title").then_some("localized".into()));
        let root = runtime.mount(members.component, []).unwrap();
        runtime.render().unwrap();
        runtime
            .bind_callback(root, callback, |arguments| {
                arguments.into_iter().next().unwrap()
            })
            .unwrap();

        let site = match &runtime
            .ir()
            .components
            .iter()
            .find(|component| component.id == members.component)
            .unwrap()
            .body[0]
        {
            IrNode::Element { site, .. } => *site,
            _ => panic!("expected Pressable element"),
        };
        let result = runtime
            .dispatch_native_event(root, site, argui_schema::builtin::CLICK)
            .unwrap();
        assert_eq!(result, DslValue::String("localized".into()));
        assert_eq!(
            runtime.instance(root).unwrap().properties[&members.properties["title"]].get(),
            &DslValue::String("localized".into())
        );
        assert!(matches!(
            runtime.instance(root).unwrap().properties[&members.properties["color"]].get(),
            DslValue::Color(_)
        ));
        runtime.clear_translator();
    }
}

mod event_errors {
    use std::collections::HashMap;

    use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
    use argui_dsl_ir::{ComponentId, IrNode, SiteId};
    use argui_dsl_runtime::{LivePackage, LiveRuntime, RuntimeError};
    use argui_dsl_semantic::DefinitionKind;

    /// Compiles one event source without loading external assets.
    fn compile(source: &str) -> CompiledProject {
        Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("event tests do not load external assets".into()),
        )
        .unwrap()
    }

    /// Finds the exported component ID and its first native event site.
    fn event_site(compiled: &CompiledProject) -> (ComponentId, SiteId) {
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
        let component = compiled
            .ir
            .components
            .iter()
            .find(|component| component.id.raw() == definition.id.raw())
            .unwrap();
        let IrNode::Element { site, .. } = component.body.first().unwrap() else {
            panic!("expected native event element");
        };
        (component.id, *site)
    }

    /// Dispatches an assignment and returns its runtime error.
    fn dispatch_error(declaration: &str, assignment: &str) -> RuntimeError {
        let source = format!(
            r#"import {{ Pressable }} from "@argui/native"
export component Main {{
    private property value: {declaration}
    Pressable {{
        label: "Go"
        on click {{ value {assignment} }}
    }}
}}
"#
        );
        let compiled = compile(&source);
        let (component, site) = event_site(&compiled);
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(component, []).unwrap();
        runtime.render().unwrap();
        runtime
            .dispatch_native_event(root, site, argui_schema::builtin::CLICK)
            .unwrap_err()
    }

    /// Rejects integer division by zero instead of allowing a panic in event code.
    #[test]
    fn integer_division_by_zero_is_a_typed_runtime_error() {
        let error = dispatch_error("int = 10", "/= 0");
        assert!(matches!(
            error,
            RuntimeError::TypeMismatch { expected, actual }
                if expected == "assignment-compatible operands" && actual == "int and int"
        ));
    }

    /// Rejects floating-point division by zero consistently with integer division.
    #[test]
    fn float_division_by_zero_is_a_typed_runtime_error() {
        let error = dispatch_error("float = 10.0", "/= 0.0");
        assert!(matches!(
            error,
            RuntimeError::TypeMismatch { expected, actual }
                if expected == "assignment-compatible operands" && actual == "float and float"
        ));
    }
}

mod event_host {
    use std::collections::HashMap;

    use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
    use argui_dsl_ir::{ComponentId, PropertyId};
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
    use argui_dsl_semantic::DefinitionKind;
    use argui_runtime::{Entity, WindowEnvironment};
    use argui_ui::{UiEventKind, UiTree};

    fn compile(source: &str) -> CompiledProject {
        Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("tests do not load external assets".into()),
        )
        .unwrap()
    }

    fn members(compiled: &CompiledProject) -> (ComponentId, PropertyId) {
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
        let DefinitionKind::Component(semantic) = &definition.kind else {
            unreachable!();
        };
        let component = compiled
            .ir
            .components
            .iter()
            .find(|component| component.id.raw() == definition.id.raw())
            .unwrap();
        let value = semantic
            .properties
            .iter()
            .zip(&component.properties)
            .find(|(property, _)| property.name == "value")
            .map(|(_, property)| property.id)
            .unwrap();
        (component.id, value)
    }

    #[test]
    fn text_editor_input_and_submit_events_update_two_way_state() {
        let source = r#"import { TextEditor } from "@argui/native"
export component Main {
    in-out property value: string = "initial"
    TextEditor {
        value <=> value
        on input { value += "!" }
        on submit { value += "?" }
    }
}
"#;
        let compiled = compile(source);
        let (component, value) = members(&compiled);
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(component, []).unwrap();
        let entity = Entity::new(runtime);
        let mount = entity.mount().unwrap();
        let element = mount.render(WindowEnvironment::default()).unwrap();
        let mut tree = UiTree::new(element);
        let node = tree.node_id_at(0).expect("text editor should be the root");

        for event in tree.event_deliveries(node, UiEventKind::TextChanged("typed".into())) {
            mount.dispatch_event(&event).unwrap();
        }
        assert_eq!(
            mount.read(|runtime| {
                runtime.instance(root).unwrap().properties[&value]
                    .get()
                    .clone()
            }),
            DslValue::String("typed!".into())
        );

        for event in tree.event_deliveries(node, UiEventKind::Submitted("done".into())) {
            mount.dispatch_event(&event).unwrap();
        }
        assert_eq!(
            mount.read(|runtime| {
                runtime.instance(root).unwrap().properties[&value]
                    .get()
                    .clone()
            }),
            DslValue::String("typed!?".into())
        );
    }
}

mod render_more {
    use std::collections::HashMap;

    use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
    use argui_dsl_ir::{ComponentId, IrNode, PropertyId, SiteId};
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
    use argui_dsl_semantic::DefinitionKind;

    /// Compiles one render fixture without accessing project assets.
    fn compile(source: &str) -> CompiledProject {
        Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("render edge tests do not load external assets".into()),
        )
        .unwrap()
    }

    /// Resolves the exported component ID from semantic and lowered project data.
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

    /// Resolves a component property by its source-level name.
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
            .find(|(candidate, _)| candidate.name == name)
            .map(|(_, candidate)| candidate.id)
            .unwrap()
    }

    /// Collects native sites carrying explicit event bindings through control flow.
    fn event_sites(nodes: &[IrNode], output: &mut Vec<SiteId>) {
        for node in nodes {
            match node {
                IrNode::Element {
                    site,
                    events,
                    children,
                    ..
                } => {
                    if !events.is_empty() {
                        output.push(*site);
                    }
                    event_sites(children, output);
                }
                IrNode::Repeater { body, .. } => event_sites(body, output),
                IrNode::Conditional {
                    then_body,
                    else_body,
                    ..
                } => {
                    event_sites(then_body, output);
                    event_sites(else_body, output);
                }
                IrNode::Slot { .. } => {}
            }
        }
    }

    #[test]
    fn render_reaches_state_animation_and_nested_event_paths() {
        let compiled = compile(
            r#"import { Column, Text } from "@argui/ui"
import { Pressable } from "@argui/native"
export component Main {
    private property active: bool = false
    private property items: array<int> = [1]
    private property count: int = 0
    Column {
        gap: 8.0
        states { compact when active { gap: 16.0 } }
        animate gap { duration: 10ms }
        Text { content: "header" }
        if active {
            for item in items key item {
                Pressable { label: "row" on click { count += 1 } }
            }
        } else {
            Pressable { label: "fallback" on click { count += 10 } }
        }
    }
}
"#,
        );
        let main = component(&compiled);
        let active = property(&compiled, "active");
        let count = property(&compiled, "count");
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(main, []).unwrap();
        runtime.render().unwrap();
        assert!(!runtime.animations.is_empty());

        let definition = runtime
            .ir()
            .components
            .iter()
            .find(|candidate| candidate.id == main)
            .unwrap();
        let mut sites = Vec::new();
        event_sites(&definition.body, &mut sites);
        assert_eq!(sites.len(), 2);
        runtime
            .dispatch_native_event(root, sites[1], argui_schema::builtin::CLICK)
            .unwrap();
        assert_eq!(
            runtime.instance(root).unwrap().properties[&count].get(),
            &DslValue::Int(10)
        );

        runtime
            .set_property(root, active, DslValue::Bool(true))
            .unwrap();
        runtime.render().unwrap();
        runtime
            .dispatch_native_event(root, sites[0], argui_schema::builtin::CLICK)
            .unwrap();
        assert_eq!(
            runtime.instance(root).unwrap().properties[&count].get(),
            &DslValue::Int(11)
        );
    }
}
