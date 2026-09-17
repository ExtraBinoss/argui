use argui::ui::TextEdit;
use argui_example_astra_editor::workspace::{EntryKind, Project};

#[test]
fn virtual_documents_form_a_stable_preorder_tree() {
    let project = Project::from_documents(
        "sample",
        [
            ("src/view.rs".into(), "view".into()),
            ("Cargo.toml".into(), "manifest".into()),
            ("src/main.rs".into(), "main".into()),
        ],
    );

    assert_eq!(project.documents.len(), 3);
    assert_eq!(project.entries[0].key, "src");
    assert_eq!(project.entries[0].kind, EntryKind::Directory);
    assert_eq!(project.entries[1].key, "src/main.rs");
    assert_eq!(project.entries[2].key, "src/view.rs");
    assert_eq!(project.entries[3].key, "Cargo.toml");
    assert_eq!(project.document_for_key("src/view.rs"), Some(0));
    assert!(
        project.documents[0]
            .highlighted_content(false)
            .unwrap()
            .is_rich()
    );
}

#[test]
fn search_index_follows_controlled_edits() {
    let mut project = Project::from_documents(
        "sample",
        [(
            "src/main.rs".into(),
            "fn main() {\n    launch();\n}\n".into(),
        )],
    );

    let matches = project.search("LAUNCH", 10);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].line, 2);

    let start = project.documents[0].content.find("launch").unwrap();
    project.documents[0]
        .apply_edit(&TextEdit::new(start..start + "launch".len(), "render"))
        .unwrap();
    assert!(project.search("launch", 10).is_empty());
    assert_eq!(project.search("render", 10)[0].line, 2);
    assert!(project.documents[0].is_dirty());
    project.documents[0].mark_saved();
    assert!(!project.documents[0].is_dirty());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_scan_prunes_build_outputs_and_binary_files() {
    let temporary = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temporary.path().join("src")).unwrap();
    std::fs::create_dir_all(temporary.path().join("target")).unwrap();
    std::fs::write(temporary.path().join("src/lib.rs"), "pub fn render() {}\n").unwrap();
    std::fs::write(temporary.path().join("target/generated.rs"), "ignored\n").unwrap();
    std::fs::write(temporary.path().join("image.png"), [0, 159, 146, 150]).unwrap();

    let project = Project::from_path(temporary.path().to_path_buf(), || false).unwrap();

    assert_eq!(project.documents.len(), 1);
    assert_eq!(project.documents[0].path, "src/lib.rs");
    assert_eq!(project.root.as_deref(), Some(temporary.path()));
    assert!(project.documents[0].highlighted_content(false).is_none());
}
