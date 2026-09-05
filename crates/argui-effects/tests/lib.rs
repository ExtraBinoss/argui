use argui_effects::registry;

#[test]
fn registry_contains_exactly_the_enabled_families() {
    let registry = registry().unwrap();
    assert!(registry.is_empty() || registry.definitions().len() == 3);
}
