use argui_widgets::{Typeahead, TypeaheadConfig, unicode_prefix};
use std::time::Duration;

#[test]
fn search_refines_cycles_skips_disabled_and_expires_without_frames() {
    let items = [
        Some("Éclair"),
        None,
        Some("Étude"),
        Some("École"),
        Some("Banane"),
    ];
    let config = TypeaheadConfig {
        timeout: Duration::from_secs(1),
    };
    let mut search = Typeahead::new(config);
    let mut find = |input, millis, active| {
        search.search(
            input,
            Duration::from_millis(millis),
            active,
            items.len(),
            |i| items[i],
            unicode_prefix,
        )
    };
    assert_eq!(find("é", 0, None), Some(0));
    assert_eq!(find("é", 100, Some(0)), Some(2));
    assert_eq!(find("é", 200, Some(2)), Some(3));
    assert_eq!(find("Éc", 1200, Some(3)), Some(3));
    assert_eq!(find("l", 1300, Some(3)), Some(0));
    assert_eq!(find("b", 2400, Some(0)), Some(4));
    assert_eq!(find("z", 3500, Some(4)), None);
    assert_eq!(find("", 3600, None), None);
    assert_eq!(find("\n", 3600, None), None);
    assert_eq!(find("é", 0, Some(99)), Some(0));
    assert_eq!(search.query(), "é");
    search.clear();
    assert_eq!(search.query(), "");
    assert_eq!(
        search.search("x", Duration::ZERO, Some(0), 0, |_| None, unicode_prefix),
        None
    );
}
