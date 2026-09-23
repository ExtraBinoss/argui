use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

/// Collects diagnostic codes from one semantic project snapshot.
fn codes(database: &mut CompilerDatabase) -> Vec<DiagnosticCode> {
    database
        .check()
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

/// Covers duplicate definitions and every unresolved import route.
#[test]
fn reports_duplicate_and_import_resolution_edges() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "@argui/ui",
        r#"import { MissingSelf } from "@argui/ui"
export component Ui {}
"#,
    );
    database.set_file(
        "app.argui",
        r#"import { MissingNative } from "@argui/native"
import { MissingUi } from "@argui/ui"
import { Outside } from "../../outside.argui"
import { Missing } from "./missing.argui"
export component App {}
export component App {}
"#,
    );
    let codes = codes(&mut database);

    assert!(
        codes.contains(&DiagnosticCode::DuplicateDefinition),
        "{codes:?}"
    );
    assert!(
        codes.contains(&DiagnosticCode::UnresolvedImport),
        "{codes:?}"
    );
}

/// Covers built-in calls, callback arity/types, numeric units, and short-circuit errors.
#[test]
fn reports_expression_boundary_diagnostics() {
    let source = r##"import { Text } from "@argui/ui"
export theme Theme { --accent: color = #336699 }
export component App {
callback changed(value: int) -> string
private property duration: duration = 1deg
private property units: float = 1deg + 1s
private property tr_count: string = tr("one", "two")
private property tr_type: string = tr(1)
private property asset_type: asset = asset(1)
private property asset_count: asset = asset("one", "two")
private property callback_count: string = changed()
private property callback_type: string = changed("bad")
private property arithmetic_rhs_bad: int = 1 + true
private property comparison_bad: bool = "a" < "b"
private property string_plus_bad: string = "a" + 1
private property bad_not: bool = !1
private property unknown_member: string = missing.field
Text { content: tr("ok") background: var(--accent) }
}
"##;
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("app.argui", source);
    let codes = codes(&mut database);

    assert!(codes.contains(&DiagnosticCode::TypeMismatch), "{codes:?}");
    assert!(codes.contains(&DiagnosticCode::InvalidAsset), "{codes:?}");
    assert!(codes.contains(&DiagnosticCode::UnitMismatch), "{codes:?}");
}
