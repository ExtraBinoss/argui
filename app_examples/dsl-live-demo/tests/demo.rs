use argui::ui::Role;
use argui_example_dsl_live_demo::Main;
use argui_testing::TestApp;

/// The AOT demo shows editable content and accessible interactive controls.
#[test]
fn demo_renders_the_hot_reload_surface() {
    let app = TestApp::new(Main::new());
    app.assert_text("Argui Live Studio");
    app.assert_text("Edit this title or button label in ui/main.argui to see the change.");
    app.assert_text("Say hello");
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics.role == Role::TextInput
            && node.semantics.label.as_deref() == Some("Your name")
    }));
}

/// Generated properties and callbacks update the demo without placeholder state.
#[test]
fn demo_updates_title_input_and_button() {
    let root = Main::new();
    root.set_title("Updated live title".into());
    let mut app = TestApp::new(root);
    app.assert_text("Updated live title");
    app.get_by_role(Role::TextInput, "Your name")
        .replace_text("Ada")
        .unwrap();
    app.assert_text("Ada");
    app.get_by_role(Role::Button, "Say hello").click().unwrap();
    app.assert_text("Hello from Argui!");
}

/// Live and generated rendering agree for the same authored DSL source.
#[cfg(feature = "argui-live")]
#[test]
fn live_render_matches_generated_demo() {
    /// Compares visible structure while ignoring renderer-owned identities and callbacks.
    ///
    /// `live` and `generated` are corresponding nodes from the two compilation modes.
    fn assert_visual_parity(live: &argui::ui::Element, generated: &argui::ui::Element) {
        assert_eq!(live.kind, generated.kind);
        assert_eq!(live.style, generated.style);
        assert_eq!(live.paint, generated.paint);
        assert_eq!(live.semantics, generated.semantics);
        assert_eq!(live.children.len(), generated.children.len());
        for (live_child, generated_child) in live.children.iter().zip(&generated.children) {
            assert_visual_parity(live_child, generated_child);
        }
    }

    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{LivePackage, LiveRuntime};
    use std::collections::HashMap;

    let source = include_str!("../ui/main.argui");
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("demo has no assets".into()),
    )
    .unwrap();
    let root = compiled
        .semantic
        .modules
        .iter()
        .find(|module| module.path == "ui/main.argui")
        .unwrap()
        .definitions
        .iter()
        .find(|definition| definition.name == "Main")
        .unwrap();
    let component = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == root.id.raw())
        .unwrap()
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut live = LiveRuntime::new(package).unwrap();
    live.mount(component, []).unwrap();
    assert_visual_parity(&live.render().unwrap(), &Main::new().render());
}
