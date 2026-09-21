use argui::ui::Role;
use argui_dsl_aot_fixture::Main;
use argui_testing::TestApp;

#[test]
fn fixture_application_config_has_one_default_window() {
    let config = argui_dsl_aot_fixture::application_config();
    assert_eq!(config.identity.display_name, "Argui DSL fixture");
    assert_eq!(config.windows.len(), 1);
    assert!(config.validate().is_ok());
}

#[test]
fn generated_root_constructs_a_native_tree() {
    let root = Main::new();
    root.set_title("Dashboard".to_string());
    let element = root.render();
    assert!(!element.children.is_empty());
}

#[test]
/// Exercises generated component state and the source-ID translation fallback.
fn generated_root_exposes_state_and_translation_fallbacks() {
    let root = Main::default();
    assert!(!root.title().is_empty());
    assert!(!root.set_title(root.title()));
    assert!(root.set_title("Dashboard".to_string()));
    assert_eq!(root.title(), "Dashboard");

    root.set_translator(|_| None);
    root.clear_translator();
    let app = TestApp::new(root);
    app.assert_text("status.ready");
}

#[test]
/// Confirms that generated consumers can enumerate the fixture's effect definitions.
fn generated_effect_definitions_are_available_to_consumers() {
    assert!(argui_dsl_aot_fixture::effect_definitions().is_empty());
}

#[test]
fn generated_root_routes_native_events_through_the_model_runtime() {
    let root = Main::new();
    root.set_title("Dashboard".to_string());
    root.set_translator(|id| (id == "status.ready").then(|| "Ready".to_string()));
    let mut app = TestApp::new(root);

    app.assert_text("Ready");
    let button_label = app
        .semantics()
        .nodes
        .iter()
        .find(|node| node.semantics.role == Role::Button)
        .and_then(|node| node.semantics.label.clone())
        .expect("fixture button needs an accessible label");
    app.get_by_role(Role::Button, button_label).click().unwrap();

    app.assert_no_text("Ready");
}
