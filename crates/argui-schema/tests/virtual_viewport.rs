use argui_schema::VirtualViewportStore;
use argui_ui::RetainedIdentity;

/// Ordinary trees consume no layout iteration, even after a virtual window unmounts.
#[test]
fn idle_and_unmounted_trees_do_not_scan_layout() {
    let mut store = VirtualViewportStore::new();
    let bounds = std::iter::from_fn(|| panic!("an unsubscribed tree must not inspect layout"));
    assert!(!store.update(bounds));
    store.begin_render();
    store.height(&RetainedIdentity::new(1, 2));
    store.end_render();
    store.begin_render();
    store.end_render();
    assert!(!store.update(std::iter::from_fn(|| panic!("removed viewport retained"))));
}

/// Only finite, changing heights of mounted windows invalidate their rows.
#[test]
fn tracks_only_mounted_viewport_heights() {
    let first = RetainedIdentity::new(1, 2);
    let second = RetainedIdentity::new(1, 3);
    let other = RetainedIdentity::new(2, 2);
    let mut store = VirtualViewportStore::new();
    store.begin_render();
    assert_eq!(store.height(&first), 0.0);
    assert_eq!(store.height(&second), 0.0);
    store.end_render();
    assert!(store.update([(&first, 100.0), (&second, 200.0), (&other, 400.0)]));
    assert!(!store.update([(&first, 100.0), (&second, 200.0), (&other, 500.0)]));
    assert!(!store.update([(&first, f32::NAN), (&second, -1.0)]));
    store.begin_render();
    assert_eq!(store.height(&second), 200.0);
    store.end_render();
    assert!(!store.update([(&first, 120.0)]));
    assert!(store.update([(&second, 220.0)]));
}
