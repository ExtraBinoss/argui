//! Slot contracts fail at authored source spans.
use argui_dsl_semantic::CompilerDatabase;

/// Slot names, projection form, and fallback cycles are checked before lowering.
#[test]
fn invalid_slot_projection_is_diagnosed() {
    for source in [
        "component C { slot body } export component Main { C { slot missing {} } }",
        "component C { slot body } export component Main { C { slot body {} slot body {} } }",
        "component C { slot body } export component Main { C { slot body {} Text {} } }",
        "component C {} export component Main { C { Text {} } }",
        "export component Main { Column { missing } }",
        "export component Main { slot a { a } Column { a } }",
        "export component Main { slot a { b } slot b { a } Column { a } }",
        "component C { slot body } export component Main { C { slot body { Missing {} } } }",
        "export component Main { slot body { Text { content: 3 } } Column { body } }",
    ] {
        let mut database = CompilerDatabase::with_builtins().unwrap();
        database.set_file(
            "main.argui",
            format!("import {{ Text, Column }} from \"@argui/native\"\n{source}"),
        );
        assert!(!database.check().is_valid(), "accepted {source}");
    }
}
