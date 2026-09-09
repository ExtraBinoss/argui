use argui_core::Modifiers;
use argui_widgets::{Collection, CollectionItem, DuplicateItemId, ListState};

fn items(ids: &[&str]) -> Collection {
    Collection::new(ids.iter().map(|id| CollectionItem::new(*id, *id))).unwrap()
}

#[test]
fn identities_are_unique_and_do_not_depend_on_labels() {
    assert_eq!(
        Collection::new([
            CollectionItem::new("a", "one"),
            CollectionItem::new("a", "two")
        ])
        .unwrap_err(),
        DuplicateItemId("a".into())
    );
    let collection = Collection::new([
        CollectionItem::new("a", "same"),
        CollectionItem::new("b", "same"),
    ])
    .unwrap();
    assert_eq!(collection.index_of("b"), Some(1));
    assert_eq!(collection.index_of("missing"), None);
    assert_eq!(collection.get(9), None);
    assert!(!collection.is_empty());
}

#[test]
fn selection_survives_reordering_and_removes_deleted_identities() {
    let original = items(&["a", "b", "c", "d"]);
    let mut state = ListState::default();
    state.select(1, &original, true, Modifiers::default());
    state.select(
        3,
        &original,
        true,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(state.selected, ["b".into(), "c".into(), "d".into()].into());
    let reordered = items(&["d", "a", "c", "b"]);
    let before = state.clone();
    state.reconcile(&reordered);
    assert_eq!(state, before);
    state.select(
        2,
        &reordered,
        true,
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(state.selected, ["b".into(), "d".into()].into());
    state.reconcile(&items(&["a", "d"]));
    assert_eq!(state.active.as_deref(), Some("a"));
    assert_eq!(state.selected, ["d".into()].into());
    state.reconcile(&Collection::default());
    assert_eq!(state, ListState::default());
}

#[test]
fn disabled_ranges_and_command_shift_preserve_only_enabled_selections() {
    let collection = Collection::new([
        CollectionItem::new("a", "A"),
        CollectionItem {
            enabled: false,
            ..CollectionItem::new("b", "B")
        },
        CollectionItem::new("c", "C"),
    ])
    .unwrap();
    let mut state = ListState::default();
    state.select(0, &collection, true, Modifiers::default());
    state.select(
        2,
        &collection,
        true,
        Modifiers {
            control: true,
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(state.selected, ["a".into(), "c".into()].into());
    let before = state.clone();
    state.select(1, &collection, true, Modifiers::default());
    state.select(9, &collection, true, Modifiers::default());
    assert_eq!(state, before);
}
