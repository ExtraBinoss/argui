//! Removal of mounted private components must not reject a compatible reload.

use super::*;

/// A removed child definition is unmounted while the root retains its state.
#[test]
fn reload_removes_mounted_child_definition() {
    let (first, old) = package(
        1,
        "import { Text } from \"@argui/native\"
         component Child { Text { content: \"child\" } }
         export component Main { private property count: int = 0 Child #child {} }",
    );
    let (second, new) = package(
        2,
        "import { Text } from \"@argui/native\"
         export component Main { private property count: int = 0 Text { content: str(count) } }",
    );
    let mut runtime = LiveRuntime::new(first).unwrap();
    let root = runtime.mount(old.component, []).unwrap();
    runtime
        .set_property(root, old.properties["count"], DslValue::Int(9))
        .unwrap();
    runtime.render().unwrap();
    let outcome = runtime.commit_reload(runtime.prepare_reload(second).unwrap());
    assert_eq!(outcome.migrated_instances, 1);
    assert_eq!(
        runtime.instance(root).unwrap().properties[&new.properties["count"]].get(),
        &DslValue::Int(9)
    );
    TestApp::new(runtime).assert_text("9");
}
