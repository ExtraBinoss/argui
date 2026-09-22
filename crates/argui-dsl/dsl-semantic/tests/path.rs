//! Project-root import path resolution.

use argui_dsl_semantic::CompilerDatabase;

/// A bare import from a nested module resolves at the project root.
#[test]
fn bare_module_import_does_not_inherit_importer_directory() {
    let mut database = CompilerDatabase::with_builtins().unwrap();
    database.set_file("shared.argui", "export component Badge { }");
    database.set_file(
        "ui/main.argui",
        "import { Badge } from \"shared.argui\" export component Main { Badge {} }",
    );
    let checked = database.check();
    assert!(checked.is_valid(), "{:#?}", checked.diagnostics);
}
