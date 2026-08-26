use argui_platform::WindowConfig;

#[test]
fn default_window_is_a_decorated_resizable_surface() {
    let config = WindowConfig::default();

    assert_eq!(config.title, "Argui");
    assert!(config.decorations);
    assert!(config.resizable);
    assert!(!config.transparent);
    assert!(config.append_to_document);
    let _attributes = config.into_attributes();
}
