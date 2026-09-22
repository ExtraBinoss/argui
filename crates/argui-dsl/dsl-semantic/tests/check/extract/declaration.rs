//! Parser recovery at incomplete public declarations.

use argui_dsl_semantic::CompilerDatabase;

/// Incomplete declarations preserve diagnostics and never create anonymous API members.
#[test]
fn incomplete_member_declarations_do_not_create_anonymous_symbols() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/broken.argui",
        r#"export struct Record { : string name: string }
export enum Choice { , Known }
export component Main {
    private property : int
    callback (value: int)
    slot : template
}"#,
    );
    let project = database.check();
    assert!(!project.diagnostics.is_empty());
    let module = project
        .modules
        .iter()
        .find(|module| module.path == "ui/broken.argui")
        .unwrap();
    for definition in &module.definitions {
        match &definition.kind {
            argui_dsl_semantic::DefinitionKind::Struct(value) => {
                assert!(value.fields.iter().all(|field| !field.name.is_empty()));
            }
            argui_dsl_semantic::DefinitionKind::Enum(value) => {
                assert!(value.variants.iter().all(|(name, _)| !name.is_empty()));
            }
            argui_dsl_semantic::DefinitionKind::Component(value) => {
                assert!(
                    value
                        .properties
                        .iter()
                        .all(|member| !member.name.is_empty())
                );
                assert!(value.callbacks.iter().all(|member| !member.name.is_empty()));
                assert!(value.slots.iter().all(|member| !member.name.is_empty()));
            }
            _ => {}
        }
    }
}
