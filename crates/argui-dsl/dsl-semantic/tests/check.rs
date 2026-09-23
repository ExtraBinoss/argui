//! Semantic checker contracts and edge cases.

#[path = "check/extract/style.rs"]
mod style;

#[path = "check/statement.rs"]
mod statement;

#[path = "check/extract/animation.rs"]
mod animation;
#[path = "check/extract/declaration.rs"]
mod declaration;
#[path = "check/mod.rs"]
mod icons;

#[path = "check/extract/visual/slot.rs"]
mod slot;

#[path = "check/extract/visual/virtual_list.rs"]
mod virtual_list;

#[path = "check/extract/visual/binding.rs"]
mod observation;

#[path = "vector_path.rs"]
mod path;

#[path = "check/expression/gradient.rs"]
mod gradient;

#[path = "check/expression/scroll.rs"]
mod scroll;

#[path = "check/extract/visual/effect.rs"]
mod effect;

mod expressions {
    use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode, Type};

    #[test]
    fn string_builtins_validate_arity_and_argument_types() {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file(
            "app.argui",
            "export component App { private property valid: bool = contains(str(42), \"4\") private property invalid: bool = contains(1, true) private property missing: string = str() private property unsupported: string = str(#ffffff) }",
        );
        let checked = database.check();
        let diagnostics = &checked.diagnostics;
        assert_eq!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == DiagnosticCode::TypeMismatch)
                .count(),
            4,
            "{diagnostics:?}"
        );
    }

    #[test]
    fn gradient_builtins_accept_arrays_and_reject_unknown_color_spaces() {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file(
            "app.argui",
            "export component App { private property linear: brush = linear_gradient([#ff0000, #00ff00, #0000ff], [0.0, 0.5, 1.0], 45.0, \"oklab\") private property radial: brush = radial_gradient([#ff0000, #0000ff], [0.0, 1.0], 0.5, 0.5, 0.7, 0.7, \"srgb\") private property conic: brush = conic_gradient([#ff0000, #0000ff], [0.0, 1.0], 0.5, 0.5, 90.0, \"linear-srgb\") private property bad: brush = linear_gradient([#ffffff, #000000], [0.0, 1.0], 0.0, \"wrong\") private property bad_count: brush = linear_gradient([#ffffff, #000000], [0.0], 0.0, \"oklab\") private property bad_order: brush = linear_gradient([#ffffff, #000000], [0.8, 0.2], 0.0, \"oklab\") }",
        );
        let checked = database.check();
        assert_eq!(
            checked
                .diagnostics
                .iter()
                .filter(|issue| issue.code == DiagnosticCode::TypeMismatch)
                .count(),
            3,
            "{:?}",
            checked.diagnostics
        );
    }

    fn codes(database: &mut CompilerDatabase) -> Vec<DiagnosticCode> {
        database
            .check()
            .diagnostics
            .iter()
            .map(|item| item.code)
            .collect()
    }

    #[test]
    fn infers_literals_units_operators_conditionals_arrays_and_members() {
        let source = r##"export struct Project {
    name: string
    count: int
}
export theme AppTheme {
    --accent: color = #336699
    --spacing: length = 12px
    --duration: duration = 250ms
    --angle: angle = 90deg
}
export component App {
    private property flag: bool = true
    private property i: int = 2
    private property f: float = 1.5
    private property text: string = "a"
    private property project: Project
    private property not_flag: bool = !flag
    private property negated: int = -i
    private property positive: int = +i
    private property arithmetic: int = i + 3 * 2
    private property mixed: float = f + i
    private property compared: bool = i < 3
    private property equal: bool = i == 2
    private property different: bool = i != 4
    private property logic: bool = flag && false || true
    private property choice: int = flag ? i : 4
    private property names: array<string> = [text, "b"]
    Text {
        content: project.name
        background: var(--accent)
    }
}
"##;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        let project = database.check();
        assert!(
            project.diagnostics.iter().all(|diagnostic| {
                !matches!(
                    diagnostic.code,
                    DiagnosticCode::UnknownName
                        | DiagnosticCode::TypeMismatch
                        | DiagnosticCode::UnitMismatch
                )
            }),
            "unexpected expression diagnostics: {:#?}",
            project.diagnostics
        );
        let app = project
            .modules
            .iter()
            .find(|module| module.path == "app.argui")
            .unwrap()
            .definitions
            .iter()
            .find(|definition| definition.name == "App")
            .unwrap();
        let argui_dsl_semantic::DefinitionKind::Component(component) = &app.kind else {
            panic!("App should be a component");
        };
        assert!(
            component
                .properties
                .iter()
                .any(|property| { property.name == "mixed" && property.value_type == Type::Float })
        );
    }

    #[test]
    fn reports_expression_name_call_member_array_and_unit_failures() {
        let source = r##"export struct Project { name: string }
export theme AppTheme { --spacing: length = 12px }
export component App {
    private property project: Project
    private property count: int = missing_name
    private property bad_member: string = project.missing
    private property bad_array: array<int> = [1, "wrong"]
    private property bad_units: float = 12px + 1s
    private property bad_unary: bool = -count
    Text {
        content: tr("ok", "too many")
        width: var(--missing)
        height: var(--)
    }
}
"##;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        let codes = codes(&mut database);
        for expected in [
            DiagnosticCode::UnknownName,
            DiagnosticCode::TypeMismatch,
            DiagnosticCode::UnitMismatch,
        ] {
            assert!(codes.contains(&expected), "missing {expected:?}: {codes:?}");
        }
    }

    #[test]
    fn validates_builtin_and_callback_calls_with_arguments() {
        let source = r#"export component App {
    callback changed(value: int)
    private property count: int = 1
    private property callback_result: string = changed("wrong")
    private property asset_value: asset = asset()
    Text {
        content: tr(1)
    }
}
"#;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        let codes = codes(&mut database);
        assert!(codes.contains(&DiagnosticCode::TypeMismatch));
        assert!(codes.contains(&DiagnosticCode::InvalidAsset));
    }

    #[test]
    fn path_and_file_queries_cover_missing_and_removed_sources() {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        let file = database.set_file("./nested/../app.argui", "export component App {}");
        assert_eq!(database.file_id("app.argui"), Some(file));
        assert_eq!(database.file_path(file), Some("app.argui"));
        assert_eq!(database.source(file), Some("export component App {}"));
        assert!(database.parse(file).is_some());
        assert!(database.remove_path("app.argui"));
        assert!(!database.remove_path("app.argui"));
        assert_eq!(database.source(file), None);
        assert_eq!(database.file_path(file), None);
    }
}

mod expression_edges {
    use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

    #[test]
    fn expression_checker_reports_mixed_comparisons_calls_and_conditional_edges() {
        let source = r##"export struct Project { name: string }
export theme AppTheme { --accent: color = #369c }
export component App {
    private property flag: bool = true
    private property project: Project
    private property percent: percentage = 25%
    private property angle: angle = 1rad
    private property duration: duration = 1s
    private property compare_bad: bool = "a" < 1
    private property compare_mixed: bool = 1 < 1.5
    private property equality_bad: bool = "a" == 1
    private property logic_bad: bool = "a" && flag
    private property arithmetic_bad: int = flag + 1
    private property conditional_float: float = flag ? 1 : 1.5
    private property conditional_bad: int = flag ? 1 : "bad"
    private property empty: array<int> = []
    private property bad_member: string = flag.name
    private property callback_bad: string = missing_callback(1)
    callback changed(value: int)
    private property callback_count: int = changed()
    private property unknown_var: color = var(--missing)
    Text { content: project.name background: var(--accent) }
}
"##;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        let project = database.check();
        let diagnostics = &project.diagnostics;
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::TypeMismatch),
            "expected type mismatch diagnostics: {diagnostics:#?}"
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::UnknownName),
            "expected unknown-name diagnostics: {diagnostics:#?}"
        );
    }

    #[test]
    fn expression_checker_accepts_unit_suffixes_and_numeric_widening() {
        let source = r##"export component App {
    private property flag: bool = true
    private property percent: percentage = 25%
    private property angle: angle = 1rad
    private property duration: duration = 1s
    private property mixed: float = flag ? 1 : 1.5
    private property reversed: float = flag ? 1.5 : 1
    private property empty: array<int> = []
    Text { width: 12px }
}
"##;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        let project = database.check();
        let diagnostics = &project.diagnostics;
        assert!(
            !diagnostics.iter().any(|diagnostic| {
                matches!(
                    diagnostic.code,
                    DiagnosticCode::TypeMismatch | DiagnosticCode::UnitMismatch
                )
            }),
            "unexpected numeric diagnostics: {diagnostics:#?}"
        );
    }

    #[test]
    fn malformed_number_reports_its_exact_source_span() {
        let source = "import { Column } from \"@argui/ui\"\nexport component App { Column { gap: 100zzz8.0 } }";
        let mut database = CompilerDatabase::with_builtins().unwrap();
        let file = database.set_file("ui/main.argui", source);
        let project = database.check();
        let diagnostic = project
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == DiagnosticCode::InvalidNumber)
            .expect("malformed literal should be diagnosed");
        assert_eq!(diagnostic.primary.file, file);
        let start = u32::from(diagnostic.primary.range.start()) as usize;
        let end = u32::from(diagnostic.primary.range.end()) as usize;
        assert_eq!(&source[start..end], "100zzz8.0");
        assert!(diagnostic.message.contains("100zzz8.0"));
    }
}

mod expression_valid_edges {
    use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

    /// Exercises every valid public expression family through semantic checking.
    #[test]
    fn accepts_literals_paths_calls_members_operators_and_repeater_locals() {
        let source = r##"export struct Project {
    name: string
    count: int
}
export theme AppTheme {
    --accent: color = #336699
}
export component App {
    private property flag: bool = true
    private property i: int = 2
    private property f: float = 1.5
    private property text: string = "a"
    private property project: Project
    private property projects: array<Project> = []
    private property callback_text: string = str(1)
    private property translated: string = tr("editor.title")
    private property icon: asset = asset("icons/app.svg")
    private property color: color = var(--accent)
    private property optional: optional<string> = null
    private property false_value: bool = false
    private property not_flag: bool = !flag
    private property positive: int = +i
    private property negative: int = -i
    private property text_plus: string = text + "b"
    private property modulo: int = i % 2
    private property less_equal: bool = i <= 2
    private property greater_equal: bool = f >= 1
    private property equal: bool = i == 2
    private property different: bool = i != 3
    private property logic: bool = flag && false || true
    private property choice_int: int = flag ? i : 4
    private property choice_float: float = flag ? 1.5 : 1
    private property names: array<string> = [text, "b"]
    callback changed(value: int) -> string
    Text { content: project.name background: var(--accent) }
    for item in projects key item.name { Text { content: item.name } }
}
"##;
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file("app.argui", source);
        let project = database.check();
        let expression_errors = project.diagnostics.iter().filter(|diagnostic| {
            matches!(
                diagnostic.code,
                DiagnosticCode::UnknownName
                    | DiagnosticCode::TypeMismatch
                    | DiagnosticCode::UnitMismatch
            )
        });
        assert!(
            expression_errors.count() == 0,
            "unexpected expression diagnostics: {:#?}",
            project.diagnostics
        );
    }
}

mod declaration_edges {
    use argui_dsl_semantic::{CompilerDatabase, DefinitionKind, DiagnosticCode, Type};

    /// Exercises declaration extraction for all supported public declaration kinds.
    #[test]
    fn extracts_nested_types_modes_assets_and_duplicate_diagnostics() {
        let source = r##"export struct Data {
    value: optional<string>
    values: array<int>
    value: string
}
export enum Mode { light light }
component Child {
    in property required: MissingType
    callback changed(value: MissingType) -> bool
    slot content
}
export theme Theme {
    --base: color = #369
    --derived: color = var(--base)
    dark { --derived: #fff }
    --base: string = "duplicate"
}
export style Card for Text {
    width: 2px
    hovered { width: 3px }
}
export effect Glow {
    shader: "glow.wgsl"
    parameter amount: MissingType
    parameter amount: int
}
export component App {
    in property required: string
    private property maybe: optional<MissingType> = null
    private property list: array<Data> = []
    private property model: model<Data>
    private property child: Child
    private property icon: asset = asset("icons/a.svg")
    private property icon_again: asset = asset("icons/a.svg")
    callback done(value: Data) -> string
    slot body
    Child { required: "ok" }
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
            codes.contains(&DiagnosticCode::DuplicateMember),
            "{codes:?}"
        );
        assert!(codes.contains(&DiagnosticCode::UnknownType), "{codes:?}");

        let definitions = project
            .modules
            .iter()
            .flat_map(|module| &module.definitions)
            .collect::<Vec<_>>();
        assert!(definitions.iter().any(|definition| {
        matches!(&definition.kind, DefinitionKind::Struct(structure) if structure.fields.len() == 3)
    }));
        assert!(definitions.iter().any(|definition| {
        matches!(&definition.kind, DefinitionKind::Theme(theme) if theme.tokens.len() == 3 && theme.modes == vec!["dark"])
    }));
        assert!(definitions.iter().any(|definition| {
        matches!(&definition.kind, DefinitionKind::Style(style) if style.target == "Text" && style.properties.len() == 1)
    }));
        assert!(definitions.iter().any(|definition| {
        matches!(&definition.kind, DefinitionKind::Effect(effect) if effect.parameters.len() == 2)
    }));
        let app = definitions
            .iter()
            .find(|definition| definition.name == "App")
            .unwrap();
        let DefinitionKind::Component(component) = &app.kind else {
            panic!("App should be a component");
        };
        assert!(component.properties.iter().any(|property| {
            property.name == "model" && matches!(&property.value_type, Type::Model(_))
        }));
        assert_eq!(component.callbacks.len(), 1);
    }
}

mod visual_edges {
    use argui_dsl_semantic::{CompilerDatabase, DiagnosticCode};

    /// Reports visual-tree contract failures while continuing through nested nodes.
    #[test]
    fn validates_native_user_component_repeater_condition_and_event_edges() {
        let source = r##"import { FocusScope, Text, TextInput } from "@argui/native"
component Child {
    in property required: string
    in property input: string
    private property writable: string = ""
    callback clicked()
    Text { content: required }
}
export component App {
    in property source: string
    out property output: string
    private property values: array<string> = ["one"]
    private property scalar: int = 1
    Child { required: source }
    Child { }
    TextInput {
        value <=> source
        on input { output = source }
    }
    FocusScope {
        accessible_name: "go"
        accessible_name: "duplicate"
        on missing { }
    }
    for item in values { Text { content: item } }
    for item in scalar key item { Text { content: item } }
    if true { Text { content: "yes" } } else { Text { content: "no" } }
    if 1 { Text { content: "bad" } }
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
        for expected in [
            DiagnosticCode::DuplicateMember,
            DiagnosticCode::UnknownEvent,
            DiagnosticCode::MissingProperty,
            DiagnosticCode::InvalidTwoWayBinding,
            DiagnosticCode::MissingRepeaterKey,
            DiagnosticCode::TypeMismatch,
        ] {
            assert!(codes.contains(&expected), "missing {expected:?}: {codes:?}");
        }
    }
}

#[path = "check/branch_edges.rs"]
mod branch_edges;
