//! Runtime element rendering and render edge cases.

#[path = "render/virtual_list.rs"]
mod virtual_list;

#[path = "render/identity.rs"]
mod assets;

#[path = "render/evaluate.rs"]
mod observation;

#[path = "render/instance.rs"]
mod path;

#[path = "render/state.rs"]
mod focus;

#[path = "render/motion.rs"]
mod motion;

#[path = "render/effect.rs"]
mod effect;

#[path = "render/child_reference.rs"]
mod child_reference;

mod gradient {
    use std::collections::HashMap;

    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{LivePackage, LiveRuntime};
    use argui_paint::Fill;
    use argui_ui::{Element, ElementKind};

    /// Finds the rendered text editor inside its generated root element.
    ///
    /// * `element` — current subtree root.
    ///
    /// Returns the first editor, if present.
    fn editor(element: &Element) -> Option<&Element> {
        if matches!(element.kind, ElementKind::TextEditor { .. }) {
            return Some(element);
        }
        element.children.iter().find_map(editor)
    }

    /// Live rendering preserves conic selection fill and repeated radial caret geometry.
    #[test]
    fn live_text_editor_accepts_generic_gradient_visuals() {
        let source = r#"import { TextInput } from "@argui/native"
export component Main {
    TextInput {
        value: "Select this text"
        label: "Gradient editor"
        selection_fill: conic_gradient([#ff0000, #00ff00, #0000ff], [0.0, 0.4, 1.0], 0.5, 0.5, 45.0, "oklab")
        selection_radius: 7.0
        caret_fill: radial_gradient([#ffffff, #00ff00], [0.0, 1.0], 0.5, 0.5, 0.7, 0.7, "srgb")
        caret_width: 4.0
        caret_height: 4.0
        caret_radius: 2.0
        caret_count: 3
        caret_blink: false
    }
}"#;
        let compiled = Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("no assets".into()),
        )
        .unwrap();
        let root_component = argui_dsl_ir::ComponentId::from_raw(
            compiled
                .semantic
                .modules
                .iter()
                .find(|module| module.path == "ui/main.argui")
                .unwrap()
                .definitions
                .iter()
                .find(|definition| definition.name == "Main")
                .unwrap()
                .id
                .raw(),
        );
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.mount(root_component, []).unwrap();
        let rendered = runtime.render().unwrap();
        let editor = editor(&rendered).expect("the editor is mounted");
        assert!(matches!(
            editor.selection_highlight.as_ref().unwrap().background,
            Fill::Conic(_)
        ));
        assert_eq!(
            editor
                .selection_highlight
                .as_ref()
                .unwrap()
                .radii
                .as_array(),
            [7.0; 4]
        );
        let ElementKind::TextEditor { caret, .. } = &editor.kind else {
            unreachable!();
        };
        assert_eq!(caret.visual.primitives.len(), 3);
        assert!(matches!(
            caret.visual.primitives[0].paint.background,
            Some(Fill::Radial(_))
        ));
        assert!(!caret.is_animated());
    }
}

mod render_behavior {
    use std::collections::HashMap;

    use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
    use argui_dsl_ir::{ComponentId, PropertyId};
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime, RuntimeError};
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

    fn members(compiled: &CompiledProject, name: &str) -> Members {
        let definition = compiled
            .semantic
            .modules
            .iter()
            .filter(|module| module.path != "@argui/ui")
            .flat_map(|module| &module.definitions)
            .find(|definition| {
                definition.name == name && matches!(definition.kind, DefinitionKind::Component(_))
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
    fn render_covers_conditional_repeater_slots_and_animation_state() {
        let source = r#"import { Card, Column, Text } from "@argui/ui"
export struct Item { id: int label: string }
component Child {
    in property item: Item
    Text { content: item.label }
}
export component Main {
    in property items: model<Item>
    private property flag: bool = true
    Column {
        gap: 8.0
        animate gap { duration: 10ms }
        Card { Text { content: "slot" } }
        for item in items key item.id { Child { item: item } }
        if flag { Text { content: "yes" } } else { Text { content: "no" } }
    }
}
"#;
        let compiled = compile(source);
        let main = members(&compiled, "Main");
        let fields = compiled.ir.structs[0]
            .fields
            .iter()
            .map(|field| field.id)
            .collect::<Vec<_>>();
        let item = |id: i64, label: &str| {
            DslValue::Struct(std::collections::BTreeMap::from([
                (fields[0], DslValue::Int(id)),
                (fields[1], DslValue::String(label.into())),
            ]))
        };
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime
            .mount(
                main.component,
                [(
                    main.properties["items"],
                    DslValue::Array(vec![item(1, "one"), item(2, "two")]),
                )],
            )
            .unwrap();
        runtime.render().unwrap();
        assert!(runtime.inspect().animations > 0);
        let flag = main.properties["flag"];
        runtime
            .set_property(root, flag, DslValue::Bool(false))
            .unwrap();
        runtime.render().unwrap();
        runtime
            .set_property(
                root,
                main.properties["items"],
                DslValue::Array(vec![item(2, "two")]),
            )
            .unwrap();
        runtime.render().unwrap();
    }

    #[test]
    fn render_errors_preserve_the_last_valid_element_and_report_invalid_state() {
        let source = r#"import { Text } from "@argui/ui"
export component Main { private property text: string = "ok" Text { content: text } }
"#;
        let compiled = compile(source);
        let main = members(&compiled, "Main");
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(main.component, []).unwrap();
        runtime.render().unwrap();
        assert_eq!(runtime.last_error(), None);
        let mut invalid = runtime.ir().clone();
        invalid.components[0].body.clear();
        let next =
            LivePackage::prepare(2, runtime.public_api_hash(), invalid, HashMap::new()).unwrap();
        let prepared = runtime.prepare_reload(next).unwrap();
        let _ = runtime.commit_reload(prepared);
        assert_eq!(runtime.root(), Some(root));
        assert!(runtime.render().is_ok());
        assert_eq!(runtime.last_error(), None);
        assert!(matches!(
            runtime.set_property(root, PropertyId::from_raw(99), DslValue::String("x".into())),
            Err(RuntimeError::MissingProperty(99))
        ));
    }

    #[test]
    fn dispatch_native_event_routes_nested_bindings_and_reports_unknown_sites() {
        let source = r#"import { TouchArea } from "@argui/native"
export component Main {
    private property count: int = 0
    TouchArea { on click { count += 1 } }
}
"#;
        let compiled = compile(source);
        let main = members(&compiled, "Main");
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(main.component, []).unwrap();
        runtime.render().unwrap();
        let definition = runtime
            .ir()
            .components
            .iter()
            .find(|component| component.id == main.component)
            .expect("mounted component must have an IR definition");
        let site = match &definition.body[0] {
            argui_dsl_ir::IrNode::Element { site, .. } => *site,
            _ => panic!("expected native element"),
        };
        runtime
            .dispatch_native_event(root, site, argui_schema::builtin::CLICK)
            .unwrap();
        assert_eq!(
            runtime.instance(root).unwrap().properties[&main.properties["count"]].get(),
            &DslValue::Int(1)
        );
        assert!(matches!(
            runtime.dispatch_native_event(root, site, argui_schema::EventId::from_raw(99)),
            Err(RuntimeError::Schema(message)) if message.contains("not bound")
        ));
        assert!(matches!(
            runtime.dispatch_native_event(
                argui_dsl_runtime::InstanceId::from_raw(99),
                site,
                argui_schema::builtin::CLICK,
            ),
            Err(RuntimeError::MissingComponent(99))
        ));
    }
}

mod render_edges {
    use std::collections::HashMap;
    use std::sync::Arc;

    use argui_dsl_compiler::{CompiledProject, Compiler, SourceModule};
    use argui_dsl_ir::{
        ComponentId, ExpressionId, IrElementTarget, IrExpressionKind, IrNode, IrValue,
        PropertyTargetId,
    };
    use argui_dsl_runtime::{LivePackage, LiveRuntime, RuntimeError};
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
        let DefinitionKind::Component(_) = &definition.kind else {
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
        }
    }

    #[test]
    fn render_repeater_retains_string_keys() {
        let source = r#"import { Text } from "@argui/ui"
export component Main {
    private property items: array<string> = ["first", "second"]
    for item in items key item { Text { content: item } }
}
"#;
        let compiled = compile(source);
        let main = members(&compiled);
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(main.component, []).unwrap();
        assert!(runtime.render().is_ok());
        assert_eq!(runtime.root(), Some(root));
    }

    #[test]
    fn render_reports_missing_expression_without_losing_the_mounted_root() {
        let source = r#"import { Text } from "@argui/ui"
export component Main {
    private property text: string = "ok"
    Text { content: text }
}
"#;
        let compiled = compile(source);
        let main = members(&compiled);
        let mut package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let definition = Arc::get_mut(&mut package.ir)
            .unwrap()
            .components
            .iter_mut()
            .find(|component| component.id == main.component)
            .unwrap();
        let IrNode::Element { properties, .. } = &mut definition.body[0] else {
            panic!("expected Text element");
        };
        properties[0].value.id = ExpressionId::from_raw(999_999);
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(main.component, []).unwrap();
        let result = runtime.render();
        assert!(matches!(
            result,
            Err(RuntimeError::MissingExpression(999_999))
        ));
        assert_eq!(runtime.root(), Some(root));
    }

    #[test]
    fn render_reports_invalid_repeater_and_conditional_values() {
        let source = r#"import { Text } from "@argui/ui"
export component Main {
    private property items: array<int> = []
    private property flag: bool = true
    for item in items key item { Text { content: "item" } }
    if flag { Text { content: "yes" } }
}
"#;

        let compiled = compile(source);
        let main = members(&compiled);
        let mut ir = compiled.ir;
        let definition = ir
            .components
            .iter_mut()
            .find(|component| component.id == main.component)
            .unwrap();
        let IrNode::Repeater { model, .. } = &mut definition.body[0] else {
            panic!("expected repeater");
        };
        model.kind = IrExpressionKind::Constant(IrValue::Bool(true));
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.mount(main.component, []).unwrap();
        assert!(matches!(
            runtime.render(),
            Err(RuntimeError::TypeMismatch { expected, actual })
                if expected == "model/array" && actual == "bool"
        ));
        let mut valid = compile(source).ir;
        let definition = valid
            .components
            .iter_mut()
            .find(|component| component.id == main.component)
            .unwrap();
        let IrNode::Conditional { condition, .. } = &mut definition.body[1] else {
            panic!("expected conditional");
        };
        condition.kind = IrExpressionKind::Constant(IrValue::Int(1));
        let package = LivePackage::prepare(2, 1, valid, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.mount(main.component, []).unwrap();
        assert!(matches!(
            runtime.render(),
            Err(RuntimeError::TypeMismatch { expected, actual })
                if expected == "bool" && actual == "int"
        ));
    }

    #[test]
    fn render_rejects_unknown_natives_and_children_for_leaf_natives() {
        let source = r#"import { Container, Text } from "@argui/native"
export component Main {
    Container { Text { content: "nested" } }
}
"#;
        let compiled = compile(source);
        let main = members(&compiled);
        let mut package =
            LivePackage::prepare(3, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let definition = Arc::get_mut(&mut package.ir)
            .unwrap()
            .components
            .iter_mut()
            .find(|component| component.id == main.component)
            .unwrap();
        let IrNode::Element { target, .. } = &mut definition.body[0] else {
            panic!("expected container element");
        };
        *target = IrElementTarget::Native(argui_schema::NativeTypeId::from_raw(999_999));
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.mount(main.component, []).unwrap();
        assert!(matches!(
            runtime.render(),
            Err(RuntimeError::Schema(message)) if message.contains("unknown native")
        ));

        let compiled = compile(source);
        let main = members(&compiled);
        let mut package =
            LivePackage::prepare(4, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let definition = Arc::get_mut(&mut package.ir)
            .unwrap()
            .components
            .iter_mut()
            .find(|component| component.id == main.component)
            .unwrap();
        let IrNode::Element {
            target, children, ..
        } = &mut definition.body[0]
        else {
            panic!("expected container element");
        };
        assert!(!children.is_empty());
        *target = IrElementTarget::Native(argui_schema::builtin::TEXT_INPUT);
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.mount(main.component, []).unwrap();
        let result = runtime.render();
        assert!(matches!(
            result,
            Err(RuntimeError::Schema(message)) if message.contains("does not accept children")
        ));
    }

    #[test]
    fn render_reports_component_binding_shape_errors() {
        let source = r#"import { Text } from "@argui/ui"
component Child {
    in-out property value: string = "child"
    Text { content: value }
}
export component Main {
    private property value: string = "parent"
    Child { value: value }
}
"#;
        let compiled = compile(source);
        let main = members(&compiled);
        let mut package =
            LivePackage::prepare(5, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let definition = Arc::get_mut(&mut package.ir)
            .unwrap()
            .components
            .iter_mut()
            .find(|component| component.id == main.component)
            .unwrap();
        let IrNode::Element { properties, .. } = &mut definition.body[0] else {
            panic!("expected child element");
        };
        properties[0].target = PropertyTargetId::Native(argui_schema::PropertyId::from_raw(1));
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.mount(main.component, []).unwrap();
        assert!(matches!(
            runtime.render(),
            Err(RuntimeError::Schema(message)) if message.contains("assigned to a component")
        ));

        let compiled = compile(source);
        let main = members(&compiled);
        let mut package =
            LivePackage::prepare(6, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let definition = Arc::get_mut(&mut package.ir)
            .unwrap()
            .components
            .iter_mut()
            .find(|component| component.id == main.component)
            .unwrap();
        let IrNode::Element { properties, .. } = &mut definition.body[0] else {
            panic!("expected child element");
        };
        properties[0].two_way = true;
        properties[0].value.kind = IrExpressionKind::Constant(IrValue::String("constant".into()));
        let mut runtime = LiveRuntime::new(package).unwrap();
        runtime.mount(main.component, []).unwrap();
        assert!(matches!(
            runtime.render(),
            Err(RuntimeError::InvalidBytecode(message))
                if message.contains("two-way binding source")
        ));
    }
}
