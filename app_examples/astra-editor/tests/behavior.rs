use argui::{
    accessibility::Role,
    core::{ColorScheme, Key, Modifiers, Point, ScrollDelta, Size},
};
use argui_example_astra_editor::{AstraEditor, workspace::Project};
use argui_testing::TestApp;

#[test]
fn editing_marks_the_document_dirty_and_session_save_clears_it() {
    let mut app = TestApp::new(AstraEditor::default());

    app.get_by_role(Role::TextArea, "Start writing…")
        .replace_text("fn main() { println!(\"edited\"); }")
        .unwrap();
    app.assert_text("Edited");
    app.click("save-document").unwrap();
    app.assert_text("Saved in the bundled workspace session");
    app.assert_text("Saved");
}

#[test]
fn tabs_switch_between_real_controlled_documents() {
    let mut app = TestApp::new(AstraEditor::default());

    app.click("tab::3").unwrap();

    app.assert_text("Cargo.toml");
    app.assert_text("TOML");
    app.assert_text("[package]");
}

#[test]
fn global_search_opens_a_matching_source_file() {
    let mut app = TestApp::new(AstraEditor::default());
    app.shortcut(
        Key::Character("f".into()),
        Modifiers {
            control: true,
            shift: true,
            ..Modifiers::default()
        },
    )
    .unwrap();
    assert!(app.bounds("workspace-search-panel").unwrap().size.height > 200.0);
    app.get_by_role(Role::SearchInput, "Search across files")
        .replace_text("EDITOR_RADIUS")
        .unwrap();

    app.assert_text("src/theme.rs");
    app.click("search-result::0").unwrap();
    app.assert_text("pub const EDITOR_RADIUS");
    app.assert_no_text("in-memory index");
}

#[test]
fn appearance_toggle_updates_the_runtime_color_scheme() {
    let mut app = TestApp::new(AstraEditor::default());

    app.click("toggle-theme").unwrap();

    assert_eq!(app.environment().color_scheme, ColorScheme::Dark);
}

#[test]
fn compact_layout_reveals_the_explorer_on_demand() {
    let mut app = TestApp::new(AstraEditor::default());
    app.resize(Size::new(390.0, 760.0)).unwrap();
    app.assert_no_text("EXPLORER");

    app.click("toggle-explorer").unwrap();

    app.assert_text("EXPLORER");
    app.assert_text("src");
}

#[test]
fn control_b_toggles_the_desktop_explorer() {
    let mut app = TestApp::new(AstraEditor::default());
    app.assert_text("EXPLORER");

    app.shortcut(
        Key::Character("b".into()),
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
    )
    .unwrap();
    app.assert_no_text("EXPLORER");

    app.shortcut(
        Key::Character("b".into()),
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
    )
    .unwrap();
    app.assert_text("EXPLORER");
}

#[test]
fn search_backdrop_is_modal_and_dismisses_without_clicking_through() {
    let mut app = TestApp::new(AstraEditor::default());
    app.click("global-search").unwrap();
    app.assert_text("in-memory index");

    app.click("workspace-search-backdrop").unwrap();

    app.assert_no_text("in-memory index");
    assert_eq!(app.environment().color_scheme, ColorScheme::Light);
}

#[test]
fn a_preloaded_highlighted_rust_file_scrolls_to_its_real_document_end() {
    let source = include_str!("../../docs-examples/src/examples/custom_elements.rs");
    assert!(source.lines().count() > 500);
    let project =
        Project::from_documents("argui", [("custom_elements.rs".into(), source.to_owned())]);
    assert!(project.documents[0].highlighted_content(false).is_some());
    let mut app = TestApp::new(AstraEditor::with_project(project));
    let editor = app.bounds("code-editor").unwrap();
    let center = Point::new(
        editor.origin.x + editor.size.width * 0.5,
        editor.origin.y + editor.size.height * 0.5,
    );

    app.wheel(center, ScrollDelta::Pixels(Point::new(0.0, -50_000.0)))
        .unwrap();

    let offset = app.scroll_offset("code-editor").unwrap();
    assert!(
        offset.y > 9_000.0,
        "editor stopped at {offset:?} inside {editor:?}"
    );
}
