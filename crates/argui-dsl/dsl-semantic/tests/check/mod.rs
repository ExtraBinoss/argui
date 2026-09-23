//! Icon import resolution and feature-gated diagnostics.

#[path = "extract/animation/keyframes.rs"]
mod keyframe_edges;
#[path = "extract/theme.rs"]
mod theme_edges;

use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

/// Resolves imported icons and aliases without eagerly loading unrelated catalog entries.
#[cfg(feature = "icons")]
#[test]
fn icons_are_resolved_on_demand_with_aliases_and_precise_errors() {
    let source = "import { Star as Favorite, Spinner, NotATablerIcon } from \"@argui/icons\"\nexport component App { Favorite {} Spinner {} }\n";
    let mut database = CompilerDatabase::with_builtins().unwrap();
    let file = database.set_file("ui/app.argui", source);
    let project = database.check();
    let app = project
        .modules
        .iter()
        .find(|module| module.path == "ui/app.argui")
        .unwrap();
    assert!(app.scope.contains_key("Favorite"));
    assert!(app.scope.contains_key("Spinner"));
    assert!(!app.scope.contains_key("NotATablerIcon"));
    assert!(
        project
            .modules
            .iter()
            .any(|module| module.path == "@argui/icons/Star.argui")
    );
    // Check is imported by the built-in Checkbox; Abacus has no built-in consumers.
    assert!(argui_dsl_stdlib::icon_component_source("Abacus").is_some());
    assert!(
        !project
            .modules
            .iter()
            .any(|module| module.path == "@argui/icons/Abacus.argui")
    );
    let diagnostic = project
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("NotATablerIcon"))
        .unwrap();
    assert_eq!(diagnostic.code, DiagnosticCode::UnresolvedImport);
    assert_eq!(diagnostic.primary.file, file);
    let range = diagnostic.primary.range;
    assert_eq!(
        &source[u32::from(range.start()) as usize..u32::from(range.end()) as usize],
        "NotATablerIcon"
    );
}

#[cfg(feature = "icons")]
#[test]
fn icon_import_completion_exposes_full_catalog() {
    let source = "import { Star } from \"@argui/icons\"\n";
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("ui/app.argui", source);
    let offset = source.find("Star").unwrap() as u32 + 2;
    let completions = database.completions("ui/app.argui", offset).unwrap();
    assert!(completions.iter().any(|item| item.label == "StarFilled"));
    assert!(completions.iter().any(|item| item.label == "Spinner"));
    assert!(completions.len() >= 5945);
}

#[cfg(not(feature = "icons"))]
#[test]
fn disabled_icon_feature_is_diagnosed_at_import() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    let file = database.set_file("ui/app.argui", "import { Star } from \"@argui/icons\"\n");
    let project = database.check();
    let diagnostic = project
        .diagnostics
        .iter()
        .find(|item| item.message.contains("`icons` feature"))
        .unwrap();
    assert_eq!(diagnostic.primary.file, file);
    assert_eq!(diagnostic.code, DiagnosticCode::UnresolvedImport);
}

/// Covers optional/generic declarations and missing native properties.
#[test]
fn extracts_optional_declarations_and_reports_missing_native_inputs() {
    let source = r##"import { Path } from "@argui/native"
export struct Data {
    title: string?
    values: array<int>
}
export component App {
    private property data: optional<Data> = null
    Path { }
}
"##;
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("app.argui", source);
    let project = database.check();
    let codes = project
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();

    assert!(
        codes.contains(&DiagnosticCode::MissingProperty),
        "{codes:?}"
    );
    let app = project
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == "App")
        .expect("App declaration should survive diagnostics");
    assert!(matches!(
        &app.kind,
        argui_dsl_semantic::DefinitionKind::Component(component)
            if component.visual_sites == 1
    ));
}
