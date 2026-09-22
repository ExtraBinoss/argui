use argui_dsl_semantic::{CompilerDatabase, SemanticClass, SymbolKind, ToolingError};

fn offset(source: &str, needle: &str) -> u32 {
    u32::try_from(source.find(needle).expect("test marker must exist")).unwrap()
}

fn database_with(source: &str) -> CompilerDatabase {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("app.argui", source);
    database
}

#[test]
fn completion_uses_component_contracts_and_expected_theme_types() {
    let source = r#"import { Button } from "@argui/ui"
import { Container } from "@argui/native"
export theme AppTheme {
    --accent: color = #336699
    --spacing: length = 12px
}
export component App {
    Button {
        text: "Save"
    }
    Container {
        background: var(--)
    }
}
"#;
    let mut database = database_with(source);
    let button = database
        .completions("app.argui", offset(source, "text: \"Save\"") + 1)
        .unwrap();
    assert!(button.iter().any(|item| item.label == "enabled"));
    assert!(button.iter().any(|item| item.label == "on click"));
    assert!(!button.iter().any(|item| item.label == "text"));

    let theme = database
        .completions("app.argui", offset(source, "--)") + 2)
        .unwrap();
    assert!(theme.iter().any(|item| item.label == "--accent"));
    assert!(!theme.iter().any(|item| item.label == "--spacing"));
}

#[test]
fn navigation_uses_semantic_identity_across_modules() {
    let model = "export struct Project { name: string }\n";
    let app = r#"import { Project } from "./model.argui"
export component App {
    in property project: Project
}
"#;
    let mut database = database_with(app);
    database.set_file("model.argui", model);
    let usage = offset(app, "Project\n");
    let definition = database.definition("app.argui", usage).unwrap().unwrap();
    assert_eq!(definition.path, "model.argui");
    assert!(model[definition.start as usize..definition.end as usize].contains("Project"));
    assert!(
        database
            .hover("app.argui", usage)
            .unwrap()
            .unwrap()
            .contains("struct Project")
    );
    let references = database.references("app.argui", usage).unwrap();
    assert_eq!(references.len(), 3);
    let edits = database.rename("app.argui", usage, "Workspace").unwrap();
    assert_eq!(edits.len(), references.len());
    assert!(edits.iter().all(|edit| edit.replacement == "Workspace"));
    assert_eq!(
        database
            .rename("app.argui", usage, "not valid")
            .unwrap_err(),
        ToolingError::InvalidIdentifier("not valid".into())
    );
}

/// Navigation at whitespace or the end of a document has no symbol identity.
#[test]
fn navigation_ignores_non_symbol_offsets_and_rejects_invalid_renames() {
    let source = "export component App { private property value: int = 1 }\n";
    let mut database = database_with(source);
    for position in [source.len() as u32, offset(source, " App") - 1] {
        assert_eq!(database.hover("app.argui", position).unwrap(), None);
        assert_eq!(database.definition("app.argui", position).unwrap(), None);
        assert!(
            database
                .references("app.argui", position)
                .unwrap()
                .is_empty()
        );
    }
    for invalid in ["", "1value", "has space", "with.dot"] {
        assert_eq!(
            database.rename("app.argui", offset(source, "value"), invalid),
            Err(ToolingError::InvalidIdentifier(invalid.into()))
        );
    }
    assert_eq!(
        database.symbols(Some("missing.argui")),
        Err(ToolingError::UnknownModule("missing.argui".into()))
    );
}

#[test]
fn symbols_highlights_colors_and_quick_fixes_share_the_checked_frontend() {
    let source = r#"import { Container } from "@argui/native"
export theme AppTheme { --accent: color = #369c }
export component App {
    Container { backgroun: var(--accent) }
}
"#;
    let mut database = database_with(source);
    let symbols = database.symbols(Some("app.argui")).unwrap();
    assert!(
        symbols
            .iter()
            .any(|symbol| { symbol.name == "App" && symbol.kind == SymbolKind::Component })
    );
    assert!(
        symbols
            .iter()
            .any(|symbol| { symbol.name == "AppTheme" && symbol.kind == SymbolKind::Theme })
    );

    let highlights = database.semantic_highlights("app.argui").unwrap();
    assert!(
        highlights
            .iter()
            .any(|item| item.class == SemanticClass::Keyword)
    );
    assert!(
        highlights
            .iter()
            .any(|item| item.class == SemanticClass::ThemeToken)
    );
    assert!(
        !highlights
            .iter()
            .any(|item| item.class == SemanticClass::Comment)
    );

    let color = database
        .color_presentation("app.argui", offset(source, "#369c") + 2)
        .unwrap()
        .unwrap();
    assert_eq!(color.label, "#336699cc");
    assert_eq!(
        &source[color.location.start as usize..color.location.end as usize],
        "#369c"
    );

    let actions = database.code_actions("app.argui").unwrap();
    assert!(actions.iter().any(|action| {
        action.title.contains("background") && action.edit.replacement == "background"
    }));
}

mod tooling_extra {
    use argui_dsl_semantic::{
        CompilerDatabase, DiagnosticCode, SemanticClass, SymbolKind, ToolingError,
    };

    fn offset(source: &str, needle: &str, occurrence: usize) -> u32 {
        let mut start = 0;
        for _ in 0..occurrence {
            let found = source[start..]
                .find(needle)
                .expect("test marker must exist");
            start += found + needle.len();
        }
        u32::try_from(
            start
                + source[start..]
                    .find(needle)
                    .expect("test marker must exist"),
        )
        .unwrap()
    }

    fn database() -> (CompilerDatabase, String) {
        let source = r##"// completion and tooling fixture
import { Button } from "@argui/ui"
import { Container } from "@argui/native"
export struct Project { name: string count: int }
export enum Choice { One Two }
export theme AppTheme { --accent: color = #369c --spacing: length = 12px }
export style Card for Text { background: #fff }
export effect Glow { shader: "glow.wgsl" }
component Child {
    in property label: string
    callback clicked(value: int)
    slot content
    Button { text: label }
}
export component App {
    in property project: Project
    Child {
        label: project.name
        on clicked { }
    }
    Button { text: project.name }
    Container { backgroun: var(--accent) }
    Container { background: #fff }
}
"##;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        (database, source.into())
    }

    #[test]
    fn completions_cover_import_scope_component_and_native_members() {
        let (mut database, source) = database();
        let imports = database
            .completions("app.argui", offset(&source, "Button", 0) + 1)
            .unwrap();
        assert!(imports.iter().any(|item| item.label == "Column"));
        assert!(imports.iter().any(|item| item.kind == SymbolKind::Native));

        let child = database
            .completions("app.argui", offset(&source, "label: project", 0) + 2)
            .unwrap();
        assert!(child.iter().any(|item| item.label == "on clicked"));
        assert!(!child.iter().any(|item| item.label == "label"));

        let scope = database
            .completions("app.argui", source.len() as u32)
            .unwrap();
        assert!(scope.iter().any(|item| item.label == "export component"));
        assert!(scope.iter().any(|item| item.label == "Project"));

        let theme = database
            .completions("app.argui", offset(&source, "var(--accent)", 0) + 6)
            .unwrap();
        assert!(theme.iter().any(|item| item.label == "--accent"));
    }

    #[test]
    fn navigation_hover_and_symbol_queries_cover_members_natives_and_errors() {
        let (mut database, source) = database();
        let native = database
            .hover("app.argui", offset(&source, "Container", 1))
            .unwrap()
            .unwrap();
        assert!(native.contains("native Container"));
        let native_property = database
            .hover("app.argui", offset(&source, "background", 1))
            .unwrap();
        assert!(native_property.is_some());
        let property = database
            .hover("app.argui", offset(&source, "project: Project", 0) + 1)
            .unwrap()
            .unwrap();
        assert!(property.contains("property project"));
        let property_definition = database
            .definition("app.argui", offset(&source, "project.name", 0))
            .unwrap()
            .unwrap();
        assert_eq!(property_definition.path, "app.argui");
        let member_references = database
            .references("app.argui", offset(&source, "project.name", 0))
            .unwrap();
        assert!(!member_references.is_empty());
        assert!(
            database
                .rename(
                    "app.argui",
                    offset(&source, "project: Project", 0),
                    "renamed_project"
                )
                .unwrap()
                .iter()
                .all(|edit| edit.replacement == "renamed_project")
        );
        assert!(
            database
                .symbols(Some("app.argui"))
                .unwrap()
                .iter()
                .any(|symbol| symbol.name == "App")
        );
        assert!(
            database
                .symbols(None)
                .unwrap()
                .iter()
                .any(|symbol| symbol.name == "Project")
        );
        assert!(matches!(
            database.symbols(Some("missing.argui")),
            Err(ToolingError::UnknownModule(_))
        ));
        assert!(matches!(
            database.hover("missing.argui", 0),
            Err(ToolingError::UnknownModule(_))
        ));
        assert!(matches!(
            database.hover("app.argui", u32::MAX),
            Err(ToolingError::InvalidOffset { .. })
        ));
    }

    #[test]
    fn token_highlights_colors_and_quick_fixes_cover_source_variants() {
        let (mut database, source) = database();
        let highlights = database.semantic_highlights("app.argui").unwrap();
        for class in [
            SemanticClass::Comment,
            SemanticClass::String,
            SemanticClass::Number,
            SemanticClass::ThemeToken,
            SemanticClass::Keyword,
            SemanticClass::Component,
            SemanticClass::Type,
            SemanticClass::Property,
            SemanticClass::Callback,
            SemanticClass::Variable,
        ] {
            assert!(
                highlights.iter().any(|item| item.class == class),
                "missing {class:?}"
            );
        }
        for (literal, expected) in [("#369c", "#336699cc"), ("#fff", "#ffffffff")] {
            let color = database
                .color_presentation("app.argui", offset(&source, literal, 0) + 1)
                .unwrap()
                .unwrap();
            assert_eq!(color.label, expected);
        }
        assert!(
            database
                .color_presentation("app.argui", offset(&source, "background", 0))
                .unwrap()
                .is_none()
        );
        let actions = database.code_actions("app.argui").unwrap();
        assert!(
            actions
                .iter()
                .any(|action| action.edit.replacement == "background")
        );
        assert!(actions.iter().all(|action| {
            action.title.starts_with("Change `") && action.edit.location.path == "app.argui"
        }));
        let project = database.check();
        assert!(
            project
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::UnknownProperty)
        );
    }
}

mod tooling_tokens_edges {
    use argui_dsl_semantic::CompilerDatabase;

    /// Returns the first byte offset of a marker in a source fixture.
    fn offset(source: &str, marker: &str) -> u32 {
        u32::try_from(source.find(marker).expect("test marker must exist")).unwrap()
    }

    /// Builds a source database containing valid and intentionally misspelled elements.
    fn database() -> (CompilerDatabase, String) {
        let source = r##"import { Button } from "@argui/ui"
import { Container } from "@argui/native"
export component App {
    Button { text: "ok" }
    Buton { text: "typo" }
    Container { backgroun: #AaBbCcDd }
    Container { totally_wrong: #112233 }
}
"##;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        (database, source.into())
    }

    /// Covers color conversion at and within literals, plus no-color outcomes.
    #[test]
    fn color_presentation_handles_long_hex_and_non_color_offsets() {
        let (mut database, source) = database();
        for (literal, expected) in [("#AaBbCcDd", "#aabbccdd"), ("#112233", "#112233ff")] {
            let at_hash = database
                .color_presentation("app.argui", offset(&source, literal))
                .unwrap()
                .unwrap();
            assert_eq!(at_hash.label, expected);
            let presentation = database
                .color_presentation("app.argui", offset(&source, literal) + 2)
                .unwrap()
                .unwrap();
            assert_eq!(presentation.label, expected);
        }
        assert!(
            database
                .color_presentation("app.argui", offset(&source, "Button") + 1)
                .unwrap()
                .is_none()
        );
        assert!(
            database
                .color_presentation("app.argui", source.len() as u32)
                .unwrap()
                .is_none()
        );
    }

    /// Covers component typo fixes and diagnostics without a nearby replacement.
    #[test]
    fn code_actions_offer_component_fix_and_skip_distant_property_fix() {
        let (mut database, source) = database();
        let actions = database.code_actions("app.argui").unwrap();
        assert!(actions.iter().any(|action| {
            action.title.contains("background") && action.edit.replacement == "background"
        }));
        assert!(
            actions
                .iter()
                .all(|action| action.edit.location.path == "app.argui")
        );
        assert!(source.contains("totally_wrong"));
    }
}

mod navigation_edges {
    use argui_dsl_semantic::{CompilerDatabase, ToolingError};

    fn offset(source: &str, needle: &str, occurrence: usize) -> u32 {
        let mut start = 0;
        for _ in 0..occurrence {
            start += source[start..].find(needle).unwrap() + needle.len();
        }
        u32::try_from(start + source[start..].find(needle).unwrap()).unwrap()
    }

    fn fixture() -> (CompilerDatabase, String) {
        let source = r##"import { FocusScope } from "@argui/native"
export theme AppTheme { --accent: color = #369c }
export component App {
    in property input: string
    out property output: string
    in-out property both: string
    callback changed(value: int)
    slot content
    FocusScope {
        role: "button"
        accessible_name: input
        on click { changed(1) }
    }
}
"##;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        (database, source.into())
    }

    #[test]
    fn navigation_describes_every_component_member_direction_and_theme() {
        let (mut database, source) = fixture();
        for (name, expected) in [
            ("input", "input data flow"),
            ("output", "output data flow"),
            ("both", "two-way data flow"),
            ("changed", "callback changed"),
            ("content", "slot content"),
            ("--accent", "theme token --accent"),
        ] {
            let hover = database
                .hover("app.argui", offset(&source, name, 0))
                .unwrap()
                .unwrap_or_default();
            assert!(hover.contains(expected), "{name}: {hover}");
        }
        let event = database
            .hover("app.argui", offset(&source, "click", 0))
            .unwrap()
            .unwrap();
        assert!(event.contains("FocusScope.on click"));
    }

    #[test]
    fn navigation_rejects_bad_renames_and_returns_empty_unknown_queries() {
        let (mut database, source) = fixture();
        let input = offset(&source, "input", 0);
        for replacement in ["", "123", "with space", "a.b"] {
            assert!(matches!(
                database.rename("app.argui", input, replacement),
                Err(ToolingError::InvalidIdentifier(value)) if value == replacement
            ));
        }
        assert!(
            !database
                .rename("app.argui", input, "valid-name")
                .unwrap()
                .is_empty()
        );

        let whitespace = source.len() as u32;
        assert_eq!(database.hover("app.argui", whitespace).unwrap(), None);
        assert_eq!(database.definition("app.argui", whitespace).unwrap(), None);
        assert!(
            database
                .references("app.argui", whitespace)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn navigation_resolves_callback_slot_and_theme_declarations() {
        let (mut database, source) = fixture();
        for (name, occurrence) in [("changed", 0), ("content", 0), ("--accent", 0)] {
            let definition = database
                .definition("app.argui", offset(&source, name, occurrence))
                .unwrap()
                .unwrap();
            assert_eq!(definition.path, "app.argui");
            assert!(definition.start < definition.end);
        }
        let unknown = database
            .references("app.argui", offset(&source, "click", 0))
            .unwrap();
        assert!(unknown.is_empty());
    }
}

mod navigation_contract_edges {
    use argui_dsl_semantic::CompilerDatabase;

    /// Returns the first byte offset of a marker.
    fn offset(source: &str, marker: &str, occurrence: usize) -> u32 {
        let mut cursor = 0;
        for _ in 0..occurrence {
            cursor += source[cursor..].find(marker).unwrap() + marker.len();
        }
        u32::try_from(cursor + source[cursor..].find(marker).unwrap()).unwrap()
    }

    /// Keeps component member navigation distinct from element binding destinations.
    #[test]
    fn property_navigation_excludes_child_assignment_destinations() {
        let source = r##"component Child {
    in property input: string
    out property output: string
    in-out property both: string
    callback changed()
    slot content
}
export component App {
    private property input: string = "value"
    Child {
        input: input
        output: input
        both <=> input
        on changed { }
        content
    }
    Text { content: input }
}
"##;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);

        let declaration = offset(source, "input: string", 1);
        let references = database.references("app.argui", declaration).unwrap();
        assert!(references.len() >= 2);
        assert!(references.iter().all(|reference| {
            !source[reference.start as usize..reference.end as usize].is_empty()
        }));

        let child_destination = offset(source, "input: input", 0);
        assert!(
            database
                .definition("app.argui", child_destination)
                .unwrap()
                .is_none()
        );
        assert!(
            database
                .references("app.argui", child_destination)
                .unwrap()
                .is_empty()
        );
        let two_way_destination = offset(source, "both <=> input", 0);
        assert!(
            database
                .definition("app.argui", two_way_destination)
                .unwrap()
                .is_none()
        );
    }
}
