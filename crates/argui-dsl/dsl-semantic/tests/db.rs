use argui_dsl_semantic::{CompilerDatabase, DefinitionKind, DiagnosticCode, Type};

fn valid_database() -> CompilerDatabase {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "ui/models.argui",
        r#"export struct Project {
    id: string
    name: string
}
"#,
    );
    database.set_file(
        "ui/project-card.argui",
        r#"import { Text } from "@argui/ui"
import { Project } from "./models.argui"
export component ProjectCard {
    in property project: Project
    callback open(id: string)
    Text { content: project.name }
}
"#,
    );
    database.set_file(
        "ui/dashboard.argui",
        r#"import { Column, Text } from "@argui/ui"
import { Project } from "./models.argui"
import { ProjectCard } from "./project-card.argui"
export component Dashboard {
    in property projects: model<Project>
    callback open_project(id: string)
    Column {
        gap: 12.0
        for project in projects key project.id {
            ProjectCard {
                project: project
                on open { open_project(project.id) }
            }
        }
        Text { content: "Projects" }
    }
}
"#,
    );
    database
}

#[test]
fn multi_file_project_resolves_types_components_callbacks_and_schema_properties() {
    let mut database = valid_database();
    let project = database.check();
    assert!(
        project.is_valid(),
        "unexpected diagnostics: {:#?}",
        project.diagnostics
    );
    assert_eq!(project.modules.len(), 4);
    let dashboard = project
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == "Dashboard")
        .unwrap();
    let DefinitionKind::Component(component) = &dashboard.kind else {
        panic!("Dashboard should be a component");
    };
    assert_eq!(component.properties.len(), 1);
    assert!(matches!(component.properties[0].value_type, Type::Model(_)));
    assert_eq!(component.repeaters, 1);
    assert_eq!(component.visual_sites, 3);
}

#[test]
fn parse_and_semantic_queries_reuse_unchanged_files() {
    let mut database = valid_database();
    let first = database.check();
    let after_first = database.stats();
    assert_eq!(after_first.parse_executions, 4);
    assert_eq!(after_first.semantic_executions, 1);
    assert!(std::sync::Arc::ptr_eq(&first, &database.check()));
    assert_eq!(database.stats(), after_first);

    database.set_file(
        "ui/models.argui",
        "export struct Project { id: string name: string archived: bool }",
    );
    let _ = database.check();
    assert_eq!(database.stats().parse_executions, 5);
    assert_eq!(database.stats().semantic_executions, 2);
}

#[test]
fn analyzer_reports_cycles_units_schema_errors_and_unkeyed_repeaters_together() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file(
        "a.argui",
        r#"import { B } from "./b.argui"
import { Text } from "@argui/ui"
export component A {
    private property left: float = right
    private property right: float = left
    in property values: model<string>
    Text { content: 12px unknown: true }
    for value in values { Text { content: value } }
}
export theme Cyclic {
    --a: color = var(--b)
    --b: color = var(--a)
}
export effect Bad { shader: "shader.txt" }
"#,
    );
    database.set_file(
        "b.argui",
        r#"import { A } from "./a.argui"
export component B { A {} }
"#,
    );
    let project = database.check();
    let codes = project
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    for expected in [
        DiagnosticCode::ImportCycle,
        DiagnosticCode::BindingCycle,
        DiagnosticCode::ThemeCycle,
        DiagnosticCode::TypeMismatch,
        DiagnosticCode::UnknownProperty,
        DiagnosticCode::MissingRepeaterKey,
        DiagnosticCode::InvalidEffect,
    ] {
        assert!(
            codes.contains(&expected),
            "missing {expected:?}: {:#?}",
            project.diagnostics
        );
    }
}

#[test]
fn private_and_missing_imports_are_source_located() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("private.argui", "struct Hidden { value: string }");
    database.set_file(
        "consumer.argui",
        r#"import { Hidden, Missing } from "./private.argui"
export component Consumer {}
"#,
    );
    let project = database.check();
    assert!(
        project
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::PrivateImport)
    );
    assert!(
        project
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::UnresolvedImport)
    );
}

#[test]
fn database_file_identity_and_removal_preserve_canonical_paths() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    let file = database.set_file("./ui/../ui/app.argui", "export component App {}");
    assert_eq!(database.file_id("ui/app.argui"), Some(file));
    assert_eq!(database.file_path(file), Some("ui/app.argui"));
    assert_eq!(database.source(file), Some("export component App {}"));
    assert!(database.parse(file).is_some());
    assert!(database.remove_file(file));
    assert!(!database.remove_file(file));
    assert!(!database.remove_path("ui/app.argui"));
    assert_eq!(database.file_id("ui/app.argui"), None);
    assert_eq!(database.source(file), None);
    assert_eq!(database.file_path(file), None);
    assert!(database.parse(file).is_none());
}

/// Verifies that replacing a file with identical text preserves the query generation.
#[test]
fn identical_file_updates_keep_cached_queries_valid() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    let source = "export component App {}";
    let file = database.set_file("app.argui", source);
    let _ = database.check();
    let before = database.stats();

    assert_eq!(database.set_file("./app.argui", source), file);
    assert_eq!(database.stats(), before);
    assert!(database.check().is_valid());
    assert_eq!(database.stats(), before);
}
