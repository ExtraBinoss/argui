use argui_dsl_ir::{BinaryOperator, IrExpressionKind, IrType, lower};
use argui_dsl_semantic::{CompilerDatabase, DefinitionKind};

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

/// Rejects invalid animation targets before IR construction.
#[test]
fn lowering_rejects_semantically_invalid_animation_targets() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/edges.argui",
        r#"import { Text } from "@argui/ui"
export component Main {
    Text { content: "ok"
        animate content { duration: 10ms }
        animate missing { duration: 1ms }
    }
}"#,
    );
    let project = database.check();
    assert!(!project.is_valid());
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    let messages = errors
        .iter()
        .map(|error| error.message.as_str())
        .collect::<Vec<_>>();
    for expected in [
        "cannot animate `content`",
        "has no animatable property `missing`",
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

#[test]
fn lowering_preserves_float_arithmetic_without_integer_promotion() {
    let mut database = checked_database(
        r#"export component Main {
    private property amount: float = 1.25 + 2.5
}"#,
    );
    let project = database.check();
    let schema = argui_schema::builtin::registry().unwrap();
    let ir = lower(&project, &schema).unwrap();
    let property = &ir.components.last().unwrap().properties[0];
    assert_eq!(property.value_type, IrType::Float);
    let value = property.default.as_ref().unwrap();
    assert_eq!(value.value_type, IrType::Float);
    let IrExpressionKind::Binary {
        operator,
        left,
        right,
    } = &value.kind
    else {
        panic!("the default should remain a binary expression");
    };
    assert_eq!(*operator, BinaryOperator::Add);
    assert_eq!(left.value_type, IrType::Float);
    assert_eq!(right.value_type, IrType::Float);
}

#[test]
fn lowering_rejects_an_empty_asset_path_after_semantic_type_checking() {
    let mut database = checked_database(
        r#"export component Main {
    private property image: asset = asset("")
}"#,
    );
    let project = database.check();
    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message == "asset() requires a static string path")
    );
}

/// Returns a validated callback-call snapshot for stale-metadata checks.
fn callback_snapshot() -> argui_dsl_semantic::SemanticProject {
    let mut database = checked_database(
        r#"import { Button } from "@argui/ui"
export component Main {
    callback clicked(value: string)
    Button { text: "ready" on click { clicked("x") } }
}"#,
    );
    let checked = database.check();
    (*checked).clone()
}

#[test]
fn lowering_reports_a_callback_signature_stale_against_valid_syntax() {
    let mut project = callback_snapshot();
    let callback = project
        .modules
        .iter_mut()
        .flat_map(|module| &mut module.definitions)
        .find_map(|definition| match &mut definition.kind {
            DefinitionKind::Component(component) if definition.name == "Main" => {
                component.callbacks.first_mut()
            }
            _ => None,
        })
        .unwrap();
    callback.parameters.clear();

    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| { error.message == "callback `clicked` expects 0 arguments, received 1" })
    );
}

#[test]
fn lowering_reports_a_callback_name_stale_against_valid_syntax() {
    let mut project = callback_snapshot();
    let callback = project
        .modules
        .iter_mut()
        .flat_map(|module| &mut module.definitions)
        .find_map(|definition| match &mut definition.kind {
            DefinitionKind::Component(component) if definition.name == "Main" => {
                component.callbacks.first_mut()
            }
            _ => None,
        })
        .unwrap();
    callback.name = "renamed".to_string();

    let schema = argui_schema::builtin::registry().unwrap();
    let errors = lower(&project, &schema).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message == "unresolved function `clicked`")
    );
}
