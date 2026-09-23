//! Typed lowering of animation drivers, stops, and state policies.

use argui_dsl_ir::{
    IrAnimationDriver, IrExpressionKind, IrTransitionPolicy, IrType, IrValue, lower,
};
use argui_dsl_semantic::CompilerDatabase;

/// Checks and lowers a small animation fixture with the canonical native schema.
///
/// * `source` — complete source of the entry module.
///
/// Returns the resolved IR with only this fixture's components, excluding appended
/// generated icon components. Panics on semantic or lowering errors.
fn compile(source: &str) -> argui_dsl_ir::IrProject {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    let file = database.set_file("ui/main.argui", source);
    let project = database.check();
    assert!(
        project.is_valid(),
        "unexpected semantic diagnostics: {:#?}",
        project.diagnostics
    );
    let schema = argui_schema::builtin::registry().unwrap();
    let mut ir = lower(&project, &schema).unwrap();
    ir.components
        .retain(|component| component.source.span.is_some_and(|span| span.file == file));
    ir.assets.retain(|asset| !asset.path.starts_with("@argui/"));
    ir
}

#[test]
fn lowering_distinguishes_empty_spring_and_timeline_drivers() {
    let project = compile(
        r#"import { Column } from "@argui/ui"
export component Main { Column { gap: 4.0 animate gap { spring {} } } }"#,
    );
    assert_eq!(
        project.components.last().unwrap().animations[0].driver,
        IrAnimationDriver::Spring
    );

    let project = compile(
        r#"import { Column } from "@argui/ui"
export component Main { Column { gap: 4.0 animate gap { duration: 10ms } } }"#,
    );
    assert_eq!(
        project.components.last().unwrap().animations[0].driver,
        IrAnimationDriver::Timeline
    );
}

#[test]
fn lowering_keeps_typed_keyframe_offsets_values_and_source_spans() {
    let project = compile(
        r#"import { Column } from "@argui/ui"
export component Main { Column { gap: 4.0 animate gap {
    duration: 400ms easing: ease-out
    keyframes { 0%: 0.0 50%: 20.0 100%: 4.0 }
} } }"#,
    );
    let animation = &project.components.last().unwrap().animations[0];
    assert_eq!(animation.driver, IrAnimationDriver::Timeline);
    assert_eq!(animation.keyframes.len(), 3);
    assert_eq!(
        animation
            .keyframes
            .iter()
            .map(|frame| frame.offset)
            .collect::<Vec<_>>(),
        vec![0.0, 0.5, 1.0]
    );
    assert!(
        animation
            .keyframes
            .iter()
            .all(|frame| frame.value.value_type == IrType::Float && frame.source.span.is_some())
    );
    assert!(animation.parameters.iter().any(|parameter| {
        parameter.name == "easing"
            && matches!(&parameter.value.kind, IrExpressionKind::Constant(IrValue::String(name)) if name == "ease-out")
    }));
}

#[test]
fn lowering_keeps_directional_state_policy_out_of_runtime_parameters() {
    let project = compile(
        r#"import { Container } from "@argui/native"
export component Main { in property expanded: bool = false Container {
    rotation: 0.0
    states { open when expanded { rotation: 90.0 } }
    animate rotation { transition: leave duration: 200ms easing: ease-out }
} }"#,
    );
    let component = project.components.last().unwrap();
    assert_eq!(component.states.len(), 1);
    assert_eq!(
        component.animations[0].transition,
        Some(IrTransitionPolicy::Leave)
    );
    assert!(
        component.animations[0]
            .parameters
            .iter()
            .all(|parameter| parameter.name != "transition")
    );
}

#[test]
fn lowering_preserves_enter_and_bidirectional_transition_policies() {
    for (spelling, expected) in [
        ("enter", IrTransitionPolicy::Enter),
        ("in-out", IrTransitionPolicy::InOut),
    ] {
        let source = format!(
            r#"import {{ Container }} from "@argui/native"
export component Main {{ in property expanded: bool = false Container {{
    rotation: 0.0
    states {{ open when expanded {{ rotation: 90.0 }} }}
    animate rotation {{ transition: {spelling} duration: 100ms }}
}} }}"#
        );
        let project = compile(&source);
        let animation = &project.components.last().unwrap().animations[0];
        assert_eq!(animation.transition, Some(expected));
        assert_eq!(animation.driver, IrAnimationDriver::Timeline);
        assert!(
            animation
                .parameters
                .iter()
                .all(|parameter| parameter.name != "transition")
        );
    }
}

/// A checked component whose target property disappeared reports the stale animation.
#[test]
fn lowering_rejects_animation_target_removed_after_checking() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/main.argui",
        "export component Main { private property progress: float = 0.0 animate progress { duration: 100ms } }",
    );
    let mut checked = (*database.check()).clone();
    assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
    let component = checked
        .modules
        .iter_mut()
        .find(|module| module.path == "ui/main.argui")
        .unwrap()
        .definitions
        .iter_mut()
        .find_map(|definition| match &mut definition.kind {
            argui_dsl_semantic::DefinitionKind::Component(component)
                if definition.name == "Main" =>
            {
                Some(component)
            }
            _ => None,
        })
        .unwrap();
    component
        .properties
        .retain(|property| property.name != "progress");
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&checked, &schema).unwrap_err();
    assert!(
        errors.iter().any(|error| error.message
            == "animation target property `progress` is unavailable during IR lowering"),
        "{errors:#?}"
    );
}
