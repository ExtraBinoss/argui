use std::collections::HashSet;

use argui_ui::EventType;

#[test]
/// The complete event vocabulary is stable and valid for public JSON payloads.
fn every_native_event_has_a_unique_lower_camel_wire_name() {
    let mut names = HashSet::new();
    for event in EventType::ALL {
        let name = event.wire_name();
        assert!(
            name.chars()
                .next()
                .is_some_and(|character| character.is_ascii_lowercase())
        );
        assert!(
            name.chars()
                .all(|character| character.is_ascii_alphanumeric())
        );
        assert!(names.insert(name), "duplicate event kind: {name}");
    }
}
