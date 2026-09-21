use std::fs;

use argui_cli::{complete, format, new_project, schema, symbols};

#[test]
fn formatter_check_and_write_modes_are_deterministic() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("main.argui");
    fs::write(
        &path,
        "import { Text } from \"@argui/ui\"\nexport component Main{Text{text:\"Hi\"}}",
    )
    .unwrap();

    let error = format(root.path(), &[], true).unwrap_err();
    assert!(error.to_string().contains("need formatting"));
    let first = format(root.path(), &[], false).unwrap();
    assert_eq!(first["changed"][0], "main.argui");
    let second = format(root.path(), &[], true).unwrap();
    assert_eq!(second["changed"].as_array().unwrap().len(), 0);
}

#[test]
/// Refuses to rewrite a syntactically invalid DSL source.
fn formatter_rejects_invalid_source_without_changing_it() {
    let root = tempfile::tempdir().unwrap();
    let invalid = "export component Main {";
    fs::write(root.path().join("main.argui"), invalid).unwrap();
    let error = format(root.path(), &["main.argui".into()], false).unwrap_err();
    assert!(error.to_string().contains("cannot format invalid source"));
    assert_eq!(
        fs::read_to_string(root.path().join("main.argui")).unwrap(),
        invalid
    );
}

#[test]
fn new_command_scaffolds_a_dsl_first_release_application() {
    let root = tempfile::tempdir().unwrap();
    let created = new_project(root.path(), "hello-argui").unwrap();
    assert!(created.join("ui/main.argui").is_file());
    assert!(
        fs::read_to_string(created.join("src/main.rs"))
            .unwrap()
            .contains("argui::include_ui!")
    );
    assert!(
        fs::read_to_string(created.join("Cargo.toml"))
            .unwrap()
            .contains("argui-dsl-build")
    );
    assert!(new_project(root.path(), "hello-argui").is_err());
}

#[test]
fn json_commands_expose_standard_components_and_semantic_context() {
    let root = tempfile::tempdir().unwrap();
    let source = r#"import { Button } from "@argui/ui"
export component Main {
    Button {
        text: "Save"
    }
}
"#;
    fs::write(root.path().join("main.argui"), source).unwrap();

    let button = schema(Some("Button")).unwrap();
    assert_eq!(button["components"][0]["kind"], "component");
    let items = complete(root.path(), "main.argui", "4:9").unwrap();
    assert!(
        items["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| { item["label"] == "enabled" })
    );
    let workspace = symbols(root.path(), None).unwrap();
    assert!(
        workspace["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| { item["name"] == "Main" })
    );
}

#[test]
fn formatter_selects_files_and_directories_without_crossing_project_boundaries() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("ui/nested")).unwrap();
    fs::create_dir_all(root.path().join("ui/.git")).unwrap();
    fs::create_dir_all(root.path().join("ui/node_modules/pkg")).unwrap();
    fs::write(
        root.path().join("ui/nested/card.argui"),
        "export component Card{Text{content:\"card\"}}",
    )
    .unwrap();
    fs::write(
        root.path().join("ui/.git/ignored.argui"),
        "export component Ignored{ }",
    )
    .unwrap();
    fs::write(
        root.path().join("ui/node_modules/pkg/ignored.argui"),
        "export component Ignored{ }",
    )
    .unwrap();
    fs::write(root.path().join("ui/notes.txt"), "not DSL").unwrap();

    let selected = format(
        root.path(),
        &["ui".into(), "ui/nested/card.argui".into()],
        false,
    )
    .unwrap();
    assert_eq!(selected["changed"].as_array().unwrap().len(), 1);
    assert!(selected["changed"][0] == "ui/nested/card.argui");
    assert!(
        fs::read_to_string(root.path().join("ui/nested/card.argui"))
            .unwrap()
            .contains("export component Card")
    );

    let outside = tempfile::tempdir().unwrap();
    let outside_path = outside.path().join("outside.argui");
    fs::write(&outside_path, "export component Outside {}\n").unwrap();
    let outside_error = format(
        root.path(),
        &[outside_path.to_string_lossy().into_owned()],
        false,
    )
    .unwrap_err();
    assert!(outside_error.to_string().contains("outside project root"));

    fs::write(root.path().join("notes.txt"), "not DSL").unwrap();
    let extension_error = format(root.path(), &["notes.txt".into()], false).unwrap_err();
    assert!(
        extension_error
            .to_string()
            .contains("must be .argui files or directories")
    );
}

#[test]
fn completion_validates_one_based_unicode_positions_and_module_membership() {
    let root = tempfile::tempdir().unwrap();
    let source = "import { Text } from \"@argui/ui\"\nexport component Main {\n    Text { content: \"é\" }\n}\n";
    fs::write(root.path().join("main.argui"), source).unwrap();

    let valid = complete(root.path(), "main.argui", "3:5").unwrap();
    assert_eq!(
        valid["offset"],
        (source.find("    Text").unwrap() + 4) as u64
    );

    for position in ["missing", "0:1", "1:0", "9:1", "9:2", "3:99"] {
        let error = complete(root.path(), "main.argui", position).unwrap_err();
        assert!(
            !error.to_string().is_empty(),
            "position {position} had no error"
        );
    }
    let missing = complete(root.path(), "unknown.argui", "1:1").unwrap_err();
    assert!(missing.to_string().contains("not part of the project"));

    let after_final_newline = complete(root.path(), "main.argui", "5:1").unwrap();
    assert_eq!(after_final_newline["offset"], source.len() as u64);
    fs::write(root.path().join("main.argui"), "export component Main {}").unwrap();
    assert!(complete(root.path(), "main.argui", "2:1").is_err());
}

#[test]
fn symbols_support_document_selection_and_stable_kind_labels() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("main.argui"),
        r#"export struct Model { value: string }
export enum State { ready, busy }
export component Main {
    in property title: string
    callback close()
    slot footer
}
export theme Theme { --accent: color = #ffffff }
export style Heading for Text { font-size: 12px }
export effect Glow { shader: "shaders/glow.wgsl" }
"#,
    )
    .unwrap();

    let workspace = symbols(root.path(), None).unwrap();
    let kinds = workspace["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|symbol| symbol["kind"].as_str())
        .collect::<Vec<_>>();
    for expected in ["struct", "enum", "component", "theme", "style", "effect"] {
        assert!(
            kinds.contains(&expected),
            "missing symbol kind {expected}: {kinds:?}"
        );
    }

    let document = symbols(root.path(), Some("main.argui")).unwrap();
    assert!(
        document["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .all(|symbol| { symbol["path"] == "main.argui" })
    );
    let outside = symbols(root.path(), Some("../main.argui")).unwrap_err();
    assert!(outside.to_string().contains("path is not project-relative"));
}

#[test]
fn schema_exposes_native_fields_and_rejects_unknown_components() {
    let all = schema(None).unwrap();
    let native = all["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|component| component["kind"] == "native")
        .expect("built-in schema should include a native component");
    assert!(native["id"].is_number());
    assert!(native["properties"].is_array());
    assert!(native["events"].is_array());
    assert!(native["slots"].is_array());
    assert!(native["documentation"].is_string());

    let error = schema(Some("not-a-component")).unwrap_err();
    assert_eq!(error.to_string(), "unknown component `not-a-component`");
}

/// A selected native component and a missing document exercise both query forms.
#[test]
fn queries_select_native_schema_and_report_unknown_document() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.argui"), "export component Main {}\n").unwrap();
    let all = schema(None).unwrap();
    let name = all["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|component| component["kind"] == "native")
        .unwrap()["name"]
        .as_str()
        .unwrap();
    let selected = schema(Some(name)).unwrap();
    assert!(
        selected["components"]
            .as_array()
            .unwrap()
            .iter()
            .any(|component| component["kind"] == "native" && component["name"] == name)
    );

    assert!(symbols(root.path(), Some("missing.argui")).is_err());
}

/// Completion positions are Unicode scalar indices, including the end of a line.
#[test]
fn completion_handles_unicode_line_end_and_malformed_numeric_coordinates() {
    let root = tempfile::tempdir().unwrap();
    let source = "export component Main { Text { content: \"é\" } }\n";
    fs::write(root.path().join("main.argui"), source).unwrap();
    let column = source.trim_end_matches('\n').chars().count() + 1;
    let end_of_first_line = complete(root.path(), "main.argui", &format!("1:{column}")).unwrap();
    assert_eq!(end_of_first_line["offset"], (source.len() - 1) as u64);
    for position in ["x:1", "1:x", "1:1:2", "1:999", "2:2"] {
        assert!(
            complete(root.path(), "main.argui", position).is_err(),
            "{position}"
        );
    }
}

/// Explicit formatter inputs reject nonexistent and extensionless paths.
#[test]
fn formatter_rejects_missing_inputs_and_reports_all_drifted_files() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("ui")).unwrap();
    for name in ["a.argui", "b.argui"] {
        fs::write(
            root.path().join("ui").join(name),
            "export component Main{Text{content:\"Hi\"}}",
        )
        .unwrap();
    }
    let error = format(root.path(), &["ui".into()], true).unwrap_err();
    let report = error.to_string();
    assert!(report.contains("2 DSL file(s) need formatting"));
    assert!(report.contains("ui/a.argui, ui/b.argui"));
    assert!(format(root.path(), &["missing.argui".into()], false).is_err());
    assert!(format(root.path(), &["missing".into()], false).is_err());
}
