use argui_dsl_ir::{SourceInfo, lower};
use argui_dsl_semantic::CompilerDatabase;

#[test]
fn dropping_a_source_span_keeps_component_and_site_ownership() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/source.argui",
        r#"import { Text } from "@argui/ui"
export component Main { Text { content: "hello" } }"#,
    );
    let project = database.check();
    assert!(
        project.is_valid(),
        "unexpected diagnostics: {:#?}",
        project.diagnostics
    );
    let schema = argui_schema::builtin::registry().unwrap();
    let ir = lower(&project, &schema).unwrap();
    let component = ir.components.last().unwrap();
    let source = match &component.body[0] {
        argui_dsl_ir::IrNode::Element { source, .. } => source.clone(),
        _ => panic!("fixture body should contain an element"),
    };

    assert!(source.span.is_some());
    assert_eq!(source.component, Some(component.id));
    assert!(source.site.is_some());
    let without_span = SourceInfo::without_span(source.clone());
    assert_eq!(without_span.span, None);
    assert_eq!(without_span.component, source.component);
    assert_eq!(without_span.site, source.site);
}
