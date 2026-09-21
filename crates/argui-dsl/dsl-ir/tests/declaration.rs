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
