//! Incremental text editing across live native and standard-library components.

use std::collections::HashMap;

use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_ir::{ComponentId, PropertyId};
use argui_dsl_runtime::{DslValue, DynamicProperty, LivePackage, LiveRuntime};
use argui_dsl_semantic::DefinitionKind;
use argui_runtime::{Entity, WindowEnvironment};
use argui_ui::{ElementKind, TextEdit, UiEventKind, UiTree};

/// Finds the compiled root component and its named properties.
///
/// * `compiled` — compiled fixture containing a `Main` component.
///
/// Returns the root component ID and its property-name index.
fn members(
    compiled: &argui_dsl_compiler::CompiledProject,
) -> (ComponentId, HashMap<String, PropertyId>) {
    let definition = compiled
        .semantic
        .modules
        .iter()
        .find(|module| module.path == "ui/main.argui")
        .unwrap()
        .definitions
        .iter()
        .find(|definition| {
            definition.name == "Main" && matches!(definition.kind, DefinitionKind::Component(_))
        })
        .unwrap();
    let DefinitionKind::Component(semantic) = &definition.kind else {
        unreachable!()
    };
    let component = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == definition.id.raw())
        .unwrap();
    let properties = semantic
        .properties
        .iter()
        .zip(&component.properties)
        .map(|(property, lowered)| (property.name.clone(), lowered.id))
        .collect();
    (component.id, properties)
}

/// Direct live properties accept UTF-8 ranges without changing an invalid value.
#[test]
fn dynamic_property_applies_utf8_edits_in_place() {
    let mut property = DynamicProperty::new(
        PropertyId::from_raw(1),
        argui_dsl_ir::IrType::String,
        DslValue::String("é🙂".into()),
    );
    assert!(
        property
            .apply_text_edit(&TextEdit::new(2..2, "中"))
            .unwrap()
    );
    assert_eq!(property.get(), &DslValue::String("é中🙂".into()));
    assert_eq!(property.revision(), 1);
    assert!(property.apply_text_edit(&TextEdit::new(1..2, "x")).is_err());
    assert_eq!(property.get(), &DslValue::String("é中🙂".into()));
    assert_eq!(property.revision(), 1);
    assert!(property.apply_text_edit(&TextEdit::new(2..5, "")).unwrap());
    assert_eq!(property.get(), &DslValue::String("é🙂".into()));
    assert_eq!(property.revision(), 2);
}

/// An edit updates the controlled value and triggers TextArea.changed afterward.
#[test]
fn input_and_text_area_forward_incremental_edits_through_two_way_links() {
    for widget in ["Input", "TextArea"] {
        let callback = if widget == "TextArea" {
            "on changed { changes += 1 }"
        } else {
            ""
        };
        let source = format!(
            "import {{ {widget} }} from \"@argui/ui\"\nexport component Main {{\n    private property draft: string = \"é🙂\"\n    private property changes: int = 0\n    {widget} {{ value <=> draft {callback} }}\n}}"
        );
        let compiled = Compiler::compile(
            [SourceModule::new("ui/main.argui", source)],
            "ui/main.argui",
            |_| Err("no assets".into()),
        )
        .unwrap();
        let (component, properties) = members(&compiled);
        let package =
            LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
        let mut runtime = LiveRuntime::new(package).unwrap();
        let root = runtime.mount(component, []).unwrap();
        let entity = Entity::new(runtime);
        let mount = entity.mount().unwrap();
        let element = mount.render(WindowEnvironment::default()).unwrap();
        let mut tree = UiTree::new(element);
        let editor = (0..)
            .take_while(|index| tree.element_at(*index).is_some())
            .find(|index| {
                matches!(
                    tree.element_at(*index).unwrap().kind,
                    ElementKind::TextEditor { .. }
                )
            })
            .and_then(|index| tree.node_id_at(index))
            .unwrap();
        assert!(
            tree.event_deliveries(editor, UiEventKind::TextChanged("unused".into()))
                .is_empty()
        );
        for edit in [TextEdit::new(2..2, "中"), TextEdit::new(2..5, "")] {
            for event in tree.event_deliveries(editor, UiEventKind::TextEdited(edit)) {
                mount.dispatch_event(&event).unwrap();
            }
        }
        mount.read(|runtime| {
            let instance = runtime.instance(root).unwrap();
            assert_eq!(
                instance.properties[&properties["draft"]].get(),
                &DslValue::String("é🙂".into())
            );
            assert_eq!(
                instance.properties[&properties["changes"]].get(),
                &DslValue::Int(if widget == "TextArea" { 2 } else { 0 })
            );
        });
    }
}
