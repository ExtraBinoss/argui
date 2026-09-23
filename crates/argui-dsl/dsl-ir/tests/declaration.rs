use argui_dsl_ir::lower;
use argui_dsl_semantic::CompilerDatabase;

#[test]
fn native_style_and_element_require_the_checked_schema_during_lowering() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/style.argui",
        r#"import { Row } from "@argui/ui"
export style Spaced for Row { gap: 8.0 }
export component Main { Row { gap: 4.0 } }"#,
    );
    let checked = database.check();
    assert!(checked.is_valid(), "{:#?}", checked.diagnostics);

    let errors = lower(&checked, &argui_schema::SchemaRegistry::new()).unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("native schema `Row` is unavailable during IR lowering")
    }));
}

/// Root-relative effect assets retain their exact path independently of generated icons.
#[test]
fn lowering_resolves_relative_assets_from_a_root_module() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    let source_file = database.set_file(
        "main.argui",
        r#"export effect RootEffect {
    shader: "effects/root.wgsl"
}"#,
    );
    let project = database.check();
    assert!(project.is_valid(), "unexpected semantic diagnostics");
    let schema = argui_schema::builtin::registry().unwrap();
    let ir = lower(&project, &schema).unwrap();
    let effects = ir
        .effects
        .iter()
        .filter(|effect| {
            effect
                .source
                .span
                .is_some_and(|span| span.file == source_file)
        })
        .collect::<Vec<_>>();
    assert_eq!(effects.len(), 1);
    let assets = ir
        .assets
        .iter()
        .filter(|asset| asset.id == effects[0].shader)
        .collect::<Vec<_>>();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].path, "effects/root.wgsl");
    assert_eq!(assets[0].kind, argui_dsl_ir::AssetKind::Shader);
}
