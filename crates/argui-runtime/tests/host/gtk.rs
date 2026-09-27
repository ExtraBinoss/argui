#[test]
/// Redraws skip idle, hidden, and minimized windows without unnecessary native queries.
fn redraw_requires_visible_active_window() {
    assert!(!super::redraw_allowed(
        false,
        || panic!("idle visibility query"),
        || panic!("idle minimized query")
    ));
    assert!(!super::redraw_allowed(
        true,
        || false,
        || panic!("hidden minimized query")
    ));
    assert!(!super::redraw_allowed(true, || true, || true));
    assert!(super::redraw_allowed(true, || true, || false));
}
