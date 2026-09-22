//! Static authored paths become embedded vector assets in typed IR.

use super::*;

/// The asset expression and inline bytes share one stable generated identity.
#[test]
fn path_call_lowers_to_a_validated_inline_vector_asset() {
    let project = compile(
        r#"import { Path } from "@argui/native"
export component Main {
    Path #shape {
        source: path(20.0, 20.0, [move_to(1.0, 1.0), line_to(19.0, 1.0), line_to(19.0, 19.0), close_path()], true, 2.0, true)
        width: 20px
        height: 20px
    }
}"#,
    );
    let asset = project
        .assets
        .iter()
        .find(|asset| asset.inline_bytes.is_some())
        .expect("path expression should generate an inline asset");
    assert_eq!(asset.kind, AssetKind::Vector);
    assert!(asset.path.starts_with("__generated__/path/"));
    let svg = std::str::from_utf8(asset.inline_bytes.as_ref().unwrap()).unwrap();
    assert!(svg.contains("fill-rule=\"evenodd\""));
    assert!(svg.contains("stroke-width=\"2\""));
    assert!(
        project
            .components
            .iter()
            .flat_map(|part| &part.body)
            .any(|node| {
                let IrNode::Element { properties, .. } = node else {
                    return false;
                };
                properties.iter().any(|binding| {
            matches!(binding.value.kind, IrExpressionKind::Asset(id) if id == asset.id)
        })
            })
    );
}

/// Invalid command ordering keeps the geometry failure source located.
#[test]
fn invalid_path_command_order_is_reported_during_lowering() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/main.argui",
        r#"import { Path } from "@argui/native"
export component Main {
    Path { source: path(20.0, 20.0, [line_to(5.0, 5.0)], true, 0.0, false) }
}"#,
    );
    let project = database.check();
    assert!(project.is_valid(), "{:#?}", project.diagnostics);
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("path command 0")),
        "{errors:#?}"
    );
}
