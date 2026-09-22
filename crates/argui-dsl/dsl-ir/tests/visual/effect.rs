//! Stable typed IR for visual effect applications.

use super::*;
use argui_dsl_ir::IrValue;

/// The four generic visual primitives share one effect schema, preserve
/// declaration order, and materialize defaults for omitted arguments.
#[test]
fn effect_instances_lower_on_rectangle_path_svg_and_image() {
    let project = compile(
        r##"import { Rectangle, Path, Svg, Image } from "@argui/native"
export effect Glow {
    shader: "effects/glow.wgsl"
    parameter gain: float
    parameter tint: color = #ffffff
}
export component Main {
    private property strength: float = 0.7
    Rectangle { effect: Glow { gain: strength tint: #ff0000 } }
    Path {
        source: path(20.0, 20.0, [move_to(1.0, 1.0), line_to(19.0, 19.0)], true, 0.0, false)
        effect: Glow { gain: 0.4 }
    }
    Svg { source: asset("icons/star.svg") effect: Glow { gain: 0.5 } }
    Image { source: asset("images/photo.png") effect: Glow { gain: 0.6 } }
}"##,
    );
    let declaration = &project.effects[0];
    assert_eq!(declaration.parameters.len(), 2);
    assert_eq!(declaration.parameters[0].name, "gain");
    assert_eq!(declaration.parameters[1].name, "tint");
    let main = project
        .components
        .iter()
        .find(|component| component.body.len() == 4)
        .unwrap();
    let expected = [
        argui_schema::builtin::RECTANGLE,
        argui_schema::builtin::PATH,
        argui_schema::builtin::SVG,
        argui_schema::builtin::IMAGE,
    ];
    for (node, native) in main.body.iter().zip(expected) {
        let IrNode::Element {
            target: IrElementTarget::Native(id),
            effect: Some(binding),
            ..
        } = node
        else {
            panic!("expected a native visual element carrying Glow: {node:?}");
        };
        assert_eq!(*id, native);
        assert_eq!(binding.effect, declaration.id);
        assert_eq!(binding.parameters.len(), 2);
        assert_eq!(
            binding.parameters[0].parameter,
            declaration.parameters[0].id
        );
        assert_eq!(
            binding.parameters[1].parameter,
            declaration.parameters[1].id
        );
    }
    let IrNode::Element {
        effect: Some(first),
        ..
    } = &main.body[0]
    else {
        unreachable!()
    };
    assert!(matches!(
        first.parameters[0].value.kind,
        IrExpressionKind::PropertyRead(_)
    ));
    assert!(matches!(
        first.parameters[1].value.kind,
        IrExpressionKind::Constant(IrValue::Color(_))
    ));
    for node in &main.body[1..] {
        let IrNode::Element {
            effect: Some(binding),
            ..
        } = node
        else {
            unreachable!()
        };
        assert_eq!(
            binding.parameters[1].value,
            declaration.parameters[1].default.clone().unwrap()
        );
    }
}

/// Stale checked effect metadata is reported at the application site.
#[test]
fn effect_application_reports_missing_checked_declaration() {
    let mut database = argui_dsl_semantic::CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/main.argui",
        r#"import { Rectangle } from "@argui/native"
export effect Glow { shader: "glow.wgsl" parameter gain: float = 0.5 }
export component Main { Rectangle { effect: Glow {} } }"#,
    );
    let mut checked = (*database.check()).clone();
    assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
    let module = checked
        .modules
        .iter_mut()
        .find(|module| module.path == "ui/main.argui")
        .unwrap();
    module
        .definitions
        .retain(|definition| definition.name != "Glow");
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = argui_dsl_ir::lower(&checked, &schema).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message == "effect `Glow` is unavailable during IR lowering"),
        "{errors:#?}"
    );
}

/// A stale parameter name cannot silently consume another argument's value.
#[test]
fn effect_application_reports_missing_required_stale_parameter() {
    let mut database = argui_dsl_semantic::CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/main.argui",
        r#"import { Rectangle } from "@argui/native"
export effect Glow { shader: "glow.wgsl" parameter gain: float }
export component Main { Rectangle { effect: Glow { gain: 0.8 } } }"#,
    );
    let mut checked = (*database.check()).clone();
    assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
    let module = checked
        .modules
        .iter_mut()
        .find(|module| module.path == "ui/main.argui")
        .unwrap();
    let effect = module
        .definitions
        .iter_mut()
        .find_map(|definition| match &mut definition.kind {
            argui_dsl_semantic::DefinitionKind::Effect(effect) if definition.name == "Glow" => {
                Some(effect)
            }
            _ => None,
        })
        .unwrap();
    effect.parameters[0].name = "renamed".into();
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = argui_dsl_ir::lower(&checked, &schema).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message == "effect `Glow` requires parameter `renamed`"),
        "{errors:#?}"
    );
}
