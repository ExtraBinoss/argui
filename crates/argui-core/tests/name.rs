use std::{collections::HashSet, sync::Arc};

use argui_core::Name;

#[test]
fn static_and_owned_names_have_content_identity() {
    let static_name = Name::from_static("editor.save");
    let owned_name = Name::from_owned(String::from("editor.save"));

    assert_eq!(static_name, owned_name);
    assert_eq!(static_name.as_str(), "editor.save");
    assert_eq!(static_name.to_string(), "editor.save");
}

#[test]
fn static_and_owned_names_share_hash_semantics() {
    let mut names = HashSet::new();
    names.insert(Name::from_static("dialog.confirm"));
    names.insert(Name::from_owned(String::from("dialog.confirm")));

    assert_eq!(names.len(), 1);
    assert!(names.contains("dialog.confirm"));
}

#[test]
fn shared_storage_is_accepted_without_copying_text() {
    let storage: Arc<str> = Arc::from("theme.surface");
    let name = Name::from(Arc::clone(&storage));

    assert_eq!(name.as_str(), storage.as_ref());
    assert_eq!(Arc::strong_count(&storage), 2);
}
