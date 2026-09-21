use argui_dsl_parser::parse;
use argui_dsl_syntax::ast::{AstNode, Declaration};

const DASHBOARD: &str = r#"import { Button, Column, Text as Label, Input } from "@argui/ui"
import { ProjectCard } from "./project-card.argui"

export struct Project {
    id: string
    name: string
}

export enum Status { idle, loading, failed }

export component Dashboard {
    in property title: string
    in-out property search: string = ""
    in property projects: model<Project>
    callback create_project()
    callback open_project(id: string) -> bool
    slot footer

    Column #root {
        gap: var(--space-lg)
        padding: var(--space-lg)
        Text { text: title + "!" }
        Input { value <=> search placeholder: tr("dashboard.search") }
        for project in projects key project.id {
            ProjectCard {
                project: project
                on open { open_project(project.id) }
            }
        }
        if loading { Spinner {} } else { footer }
        Button { on click { create_project() } }
    }

    states {
        compact when width < 600px { sidebar-width: 0px }
    }
    animate sidebar-width { spring { stiffness: 220 damping: 24 } }
}

export theme AppTheme {
    --background: color = #0b0c10
    --space-lg: length = 24px
    dark { --background: #000000 }
}

export style Heading1 for Text {
    font-size: 28px
    foreground: var(--foreground)
    hovered { foreground: #ffffff }
}

export effect Glow {
    shader: "../shaders/glow.wgsl"
    parameter intensity: float = 0.35
    parameter tint: color = var(--accent)
}
"#;

#[test]
fn proposed_language_surface_round_trips_losslessly() {
    let parsed = parse(DASHBOARD);
    assert_eq!(parsed.syntax().to_string(), DASHBOARD);
    assert!(
        parsed.diagnostics().is_empty(),
        "unexpected diagnostics: {:#?}",
        parsed.diagnostics()
    );
    let declarations = parsed.tree().declarations().collect::<Vec<_>>();
    assert_eq!(declarations.len(), 8);
    let component = declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Component(component) => Some(component),
            _ => None,
        })
        .unwrap();
    assert_eq!(component.name().unwrap().text(), "Dashboard");
    assert_eq!(component.properties().count(), 3);
    assert!(component.elements().count() >= 6);
    assert!(!component.syntax().text_range().is_empty());
}

#[test]
fn incomplete_source_recovers_into_a_lossless_tree_with_multiple_diagnostics() {
    let source = "export component Broken {\n Column {\n text: user.\n if loading { Spinner {\n";
    let parsed = parse(source);
    assert_eq!(parsed.syntax().to_string(), source);
    assert!(parsed.diagnostics().len() >= 2);
    assert_eq!(parsed.tree().declarations().count(), 1);
}

#[test]
fn syntax_errors_do_not_hide_later_declarations() {
    let source = "garbage !\nexport struct Good { value: string }\nexport enum State { ready }";
    let parsed = parse(source);
    assert_eq!(parsed.syntax().to_string(), source);
    assert!(!parsed.is_valid());
    assert_eq!(parsed.tree().declarations().count(), 2);
}

#[test]
fn parser_accepts_all_declaration_members_and_expression_forms() {
    let source = r#"import { Foo as Renamed, Bar } from "@argui/native";
export struct Data { first: optional<string> = null, second: array<int> = [1, 2]; }
enum State { idle, busy; }
component Example {
    property plain: int = 1
    in property title: string
    out property status: string
    in-out property query: string = ""
    private property flags: bool = !false && true || false
    callback fire(value: string, count: int) -> bool
    callback ping()
    slot content

    Container #root {
        width: (1 + 2) * 3;
        padding: 2.0
        content <=> query;
        on click {
            let query = query
            query += "!"
            query -= "?"
            query *= "*"
            query /= "/"
            return fire("done", 1)
        }
        for value in [1, 2] key value {
            Text { text: value ? "yes" : "no" }
        }
        if flags { content } else if query == "" { Text {} } else { content }
        states { compact when flags { gap: 1.0 } }
        animate gap { spring { stiffness: 1.0 damping: 2.0 } }
    }
}
export theme Theme {
    --accent: color = #369
    --space: length = 8px
    dark { --accent: #ffffff }
}
style Heading for Text {
    width: 12.0
    hovered { width: 14.0 }
}
export effect Glow {
    shader: "effects/glow.wgsl"
    parameter amount: float = 1.0e-2
    parameter tint: color
}
"#;
    let parsed = parse(source);
    assert_eq!(parsed.syntax().to_string(), source);
    assert!(
        parsed.diagnostics().is_empty(),
        "unexpected diagnostics: {:#?}",
        parsed.diagnostics()
    );
    assert_eq!(parsed.tree().declarations().count(), 7);
}

#[test]
fn parser_reports_missing_expression_and_delimiters_at_eof() {
    let source = "export component Broken { property value: string = if { Text { text: \"x\" }";
    let parsed = parse(source);
    assert_eq!(parsed.syntax().to_string(), source);
    assert!(!parsed.is_valid());
    assert!(parsed.diagnostics().len() >= 3);
}

#[test]
fn parser_recovers_from_reserved_aggregate_members_without_stalling() {
    let source = "enum Modes { on, active, }";
    let parsed = parse(source);
    assert_eq!(parsed.syntax().to_string(), source);
    assert!(!parsed.is_valid());
    assert!(!parsed.diagnostics().is_empty());
}

#[test]
fn parser_recovers_invalid_import_items_and_optional_aliases() {
    let source = "import { Foo as, Bar, , Baz } from \"./module.argui\";";
    let parsed = parse(source);
    assert_eq!(parsed.syntax().to_string(), source);
    assert!(!parsed.is_valid());
    assert!(parsed.diagnostics().len() >= 2);
}

/// Accepts an import list whose final item is immediately followed by its brace.
#[test]
fn parser_accepts_import_lists_without_a_trailing_comma() {
    let source = "import { Text } from \"@argui/ui\"";
    let parsed = parse(source);
    assert_eq!(parsed.syntax().to_string(), source);
    assert!(
        parsed.is_valid(),
        "unexpected diagnostics: {:?}",
        parsed.diagnostics()
    );
}

/// Recovers a missing expression when the assignment reaches end of input.
#[test]
fn parser_reports_an_expression_missing_at_end_of_file() {
    let source = "export component Broken { property value: string =";
    let parsed = parse(source);
    assert_eq!(parsed.syntax().to_string(), source);
    assert!(!parsed.is_valid());
    assert!(
        parsed
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.message.contains("expected expression"))
    );
}
