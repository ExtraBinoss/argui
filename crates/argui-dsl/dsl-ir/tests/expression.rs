use argui_dsl_ir::lower;
use argui_dsl_semantic::CompilerDatabase;

/// Builds a database for a source that deliberately exercises lowering-only checks.
fn checked_database(source: &str) -> CompilerDatabase {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("ui/edges.argui", source);
    let project = database.check();
    assert!(
        project.is_valid(),
        "unexpected semantic diagnostics: {:#?}",
        project.diagnostics
    );
    database
}

/// Reports errors that semantic analysis intentionally defers to IR lowering.
#[test]
fn lowering_validates_animation_expression_boundaries() {
    let mut database = checked_database(
        r#"import { Text } from "@argui/ui"
export struct Item { id: int }
export component Main {
    in property item: Item
    in-out property query: string = ""
    callback notify(value: string) -> bool
    Text { content: "ok"
        animate content {
            missing_call: missing()
            missing_token: var(--missing)
            wrong_arity: notify()
            missing_path: absent
            unknown_struct_field: item.unknown
            scalar_member: query.missing
            empty_asset: asset("")
            mixed_number: 2 + 1.5
        }
        animate missing { duration: 1ms }
    }
}"#,
    );
    let project = database.check();
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    let messages = errors
        .iter()
        .map(|error| error.message.as_str())
        .collect::<Vec<_>>();
    for expected in [
        "unresolved function `missing`",
        "unresolved theme token `--missing`",
        "callback `notify` expects 1 arguments, received 0",
        "unresolved expression name `absent`",
        "unresolved struct field `unknown`",
        "member read base is not a struct",
        "asset() requires a static string path",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing lowering error {expected:?}: {messages:?}"
        );
    }
}

/// Reports a native-schema mismatch instead of silently lowering a guessed target.
#[test]
fn lowering_rejects_native_elements_missing_from_the_schema() {
    let mut database = checked_database(
        r#"import { Text } from "@argui/ui"
export component Main { Text { content: "ok" } }"#,
    );
    let project = database.check();
    let errors = lower(&project, &argui_schema::SchemaRegistry::new()).unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("native schema `Text` is unavailable during IR lowering")
    }));
}
