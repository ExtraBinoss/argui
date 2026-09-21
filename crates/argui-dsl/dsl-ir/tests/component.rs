use argui_dsl_ir::{ComponentId, IrElementTarget, IrNode, lower};
use argui_dsl_semantic::CompilerDatabase;
use argui_dsl_syntax::{FileId, TextRange};

/// Returns a validated component snapshot that callers can perturb independently.
fn checked_snapshot() -> argui_dsl_semantic::SemanticProject {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/snapshot.argui",
        r#"import { Text } from "@argui/ui"
export component Main { Text { content: "ready" } }"#,
    );
    let project = database.check();
    assert!(project.is_valid(), "{:#?}", project.diagnostics);
    (*project).clone()
}

#[test]
fn lowering_reports_a_component_whose_module_file_has_no_syntax_tree() {
    let mut project = checked_snapshot().clone();
    let module = project
        .modules
        .iter_mut()
        .find(|module| module.path == "ui/snapshot.argui")
        .unwrap();
    let original_span = module.definitions[0].span;
    module.file = FileId::from_raw(module.file.raw() + 1);
    let missing_file = module.file;
    assert!(project.syntax(missing_file).is_none());

    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "component syntax is unavailable" && error.span == original_span
    }));
}

#[test]
fn lowering_reports_a_component_whose_declaration_range_has_no_match() {
    let mut project = checked_snapshot().clone();
    let module = project
        .modules
        .iter_mut()
        .find(|module| module.path == "ui/snapshot.argui")
        .unwrap();
    let definition = &mut module.definitions[0];
    definition.span.range = TextRange::new(u32::MAX.into(), u32::MAX.into());
    let unmatched_span = definition.span;

    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "component syntax is unavailable" && error.span == unmatched_span
    }));
}

#[test]
fn template_slot_preserves_lazy_caller_repeater_and_owner() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/main.argui",
        r#"import { VList, Text } from "@argui/native"
component VirtualList {
    in property row_height: float
    in-out property scroll: float = 0.0
    slot rows: template
    VList { row_height: row_height offset <=> scroll rows }
}
export component Main {
    private property items: array<string> = ["one"]
    private property offset: float = 0.0
    VirtualList { row_height: 32.0 scroll <=> offset
        for item in items key item { Text { content: item } }
    }
}"#,
    );
    let checked = database.check();
    assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
    let schema = argui_schema::builtin::registry().unwrap();
    let project = lower(&checked, &schema).unwrap();
    let local = checked
        .modules
        .iter()
        .find(|module| module.path == "ui/main.argui")
        .unwrap();
    let component_id = |name| {
        ComponentId::from_raw(
            local
                .definitions
                .iter()
                .find(|definition| definition.name == name)
                .unwrap()
                .id
                .raw(),
        )
    };
    let wrapper_id = component_id("VirtualList");
    let caller_id = component_id("Main");
    let wrapper = project
        .components
        .iter()
        .find(|component| component.id == wrapper_id)
        .expect("lowered VirtualList missing");
    let caller = project
        .components
        .iter()
        .find(|component| component.id == caller_id)
        .expect("lowered Main missing");
    assert_eq!(wrapper.slots.len(), 1);
    assert_eq!(wrapper.template_slots, wrapper.slots);
    let IrNode::Element {
        target: IrElementTarget::Native(_),
        children,
        ..
    } = &wrapper.body[0]
    else {
        panic!("wrapper must own the native VList");
    };
    let [IrNode::Slot { slot, .. }] = children.as_slice() else {
        panic!("VList must contain one lazy template slot");
    };
    assert_eq!(*slot, wrapper.template_slots[0]);
    let IrNode::Element {
        target: IrElementTarget::Component(id),
        children,
        ..
    } = &caller.body[0]
    else {
        panic!("caller must instantiate the wrapper");
    };
    assert_eq!(*id, wrapper.id);
    let [
        IrNode::Repeater {
            model, key, body, ..
        },
    ] = children.as_slice()
    else {
        panic!("caller must retain one repeater rather than materialized children");
    };
    assert_eq!(model.source.component, Some(caller.id));
    assert_eq!(key.source.component, Some(caller.id));
    assert_eq!(body.len(), 1);
}
