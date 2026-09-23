//! Retained child output references in typed IR.

use super::*;

/// An identified child exposes its public output as a typed read from that site.
#[test]
fn child_output_read_uses_the_identified_child_site() {
    let ir = compile(
        r#"import { Text } from "@argui/native"
component Child {
    out property label: string = "ready"
    Text { content: label }
}
export component Main {
    Text { content: child.label }
    Child #child {}
}"#,
    );
    let main = ir.components.last().unwrap();
    let IrNode::Element { properties, .. } = &main.body[0] else {
        panic!("expected the output reader");
    };
    let IrNode::Element { site, .. } = &main.body[1] else {
        panic!("expected the identified child");
    };
    assert!(matches!(
        &properties[0].value.kind,
        IrExpressionKind::ChildPropertyRead { site: read_site, .. } if read_site == site
    ));
    assert_eq!(properties[0].value.value_type, IrType::String);
}

/// Conditional and repeated children have no single stable output to reference.
#[test]
fn dynamic_child_sites_are_excluded_from_static_output_references() {
    let ir = compile(
        r#"import { Text } from "@argui/native"
component Child {
    out property label: string = "ready"
    Text { content: label }
}
export component Main {
    private property visible: bool = true
    private property items: array<string> = ["one"]
    if visible { Child #conditional {} }
    for item in items key item { Child #repeated {} }
}"#,
    );
    let main = ir.components.last().unwrap();
    assert!(main.referenced_child_sites().is_empty());
}

/// A stale native import cannot resolve a scroll target to an unrelated site.
#[test]
fn scroll_target_removed_from_checked_native_scope_is_rejected() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/main.argui",
        r#"import { Flickable, TouchArea } from "@argui/native"
export component Main {
    TouchArea { on moved { scroll_to(#viewport, 0px, 12px) } }
    Flickable #viewport { width: 100px height: 100px }
}"#,
    );
    let mut checked = (*database.check()).clone();
    assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
    let module = checked
        .modules
        .iter_mut()
        .find(|module| module.path == "ui/main.argui")
        .unwrap();
    module.native_scope.remove("Flickable");
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&checked, &schema).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message == "unresolved scroll target `#viewport`"),
        "{errors:#?}"
    );
}

/// Native observation resolves to the identified visual site.
#[test]
fn observed_native_read_uses_the_referenced_site_and_schema_property() {
    let ir = compile(
        r#"import { Rectangle, TouchArea } from "@argui/native"
export component Main {
    Rectangle { opacity: touch.pressed ? 0.5 : 1.0 }
    TouchArea #touch {}
}"#,
    );
    let main = ir
        .components
        .iter()
        .find(|component| {
            matches!(
                component.body.as_slice(),
                [
                    IrNode::Element {
                        target: IrElementTarget::Native(argui_schema::builtin::RECTANGLE),
                        ..
                    },
                    IrNode::Element {
                        target: IrElementTarget::Native(argui_schema::builtin::TOUCH_AREA),
                        ..
                    }
                ]
            )
        })
        .unwrap();
    let IrNode::Element { properties, .. } = &main.body[0] else {
        panic!("expected Rectangle");
    };
    let IrNode::Element { site, .. } = &main.body[1] else {
        panic!("expected TouchArea");
    };
    let IrExpressionKind::Conditional { condition, .. } = &properties[0].value.kind else {
        panic!("expected reactive conditional");
    };
    assert!(matches!(
        &condition.kind,
        IrExpressionKind::ObservedRead {
            site: observed,
            property: argui_schema::builtin::PRESSED,
            observation: argui_dsl_ir::IrObservation::Pressed,
        } if *observed == *site
    ));
}
