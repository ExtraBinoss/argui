use argui_effects::registry;

#[test]
fn registry_contains_exactly_the_enabled_families() {
    let registry = registry().unwrap();
    let expected = usize::from(cfg!(feature = "artistic")) * 2
        + usize::from(cfg!(feature = "scroll")) * 2
        + usize::from(cfg!(feature = "liquid-glass"));
    assert_eq!(registry.definitions().len(), expected);
}
