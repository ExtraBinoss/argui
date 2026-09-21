use argui_reactive::RetainedPropertyStore;

#[test]
fn local_writes_survive_rerenders_but_unmodified_defaults_follow_inputs() {
    let mut store = RetainedPropertyStore::<(&str, i64)>::new();
    store.begin_render();
    let first = store.defaulted(("button", 1), 7, false);
    let untouched = store.defaulted(("button", 1), 8, 1_i64);
    store.end_render();

    first.set(true);
    store.begin_render();
    let retained = store.defaulted(("button", 1), 7, false);
    let refreshed = store.defaulted(("button", 1), 8, 2_i64);
    store.end_render();
    assert!(retained.get());
    assert_eq!(retained.revision(), first.revision());
    assert_eq!(untouched.get(), 2);
    assert_eq!(refreshed.get(), 2);
    assert_eq!(store.len(), 2);
}

#[test]
fn controlled_values_win_and_unmounted_keys_are_pruned() {
    let mut store = RetainedPropertyStore::<(&str, i64)>::new();
    store.begin_render();
    let first = store.controlled(("switch", 1), 3, false);
    let other = store.defaulted(("switch", 2), 3, false);
    store.end_render();
    first.set(true);
    other.set(true);

    store.begin_render();
    let controlled = store.controlled(("switch", 1), 3, false);
    store.end_render();
    assert!(!controlled.get());
    assert_eq!(store.len(), 1);

    store.begin_render();
    store.end_render();
    assert!(store.is_empty());
}

/// A stable property site cannot silently change its Rust value type.
#[test]
#[should_panic(expected = "stable property ID must retain its Rust type")]
fn retained_property_rejects_type_change_at_the_same_site() {
    let mut store = RetainedPropertyStore::<&str>::new();
    store.begin_render();
    let _ = store.defaulted("item", 1, 42_i64);
    let _ = store.defaulted("item", 1, "forty two".to_owned());
}
