use super::pages::AtlasPages;

/// Verifies padded shelf packing and protects every page referenced by the current frame.
#[test]
fn pages_pack_padded_glyphs_and_preserve_pinned_layers() {
    let mut pages = AtlasPages::new(8, 2);
    pages.begin_frame();
    let first = pages.allocate(2, 2).unwrap();
    assert_eq!(
        (first.page, first.origin, first.evicted),
        (0, [0, 0], false)
    );
    assert_eq!(pages.allocate(2, 2).unwrap().origin, [4, 0]);
    assert_eq!(pages.allocate(2, 2).unwrap().origin, [0, 4]);
    assert_eq!(pages.allocate(2, 2).unwrap().origin, [4, 4]);
    assert_eq!(pages.allocate(6, 6).unwrap().page, 1);
    assert_eq!(pages.used(), 2);
    assert!(pages.allocate(1, 1).is_none());
    pages.begin_frame();
    pages.pin(0);
    let recycled = pages.allocate(6, 6).unwrap();
    assert_eq!(
        (recycled.page, recycled.origin, recycled.evicted),
        (1, [0, 0], true)
    );
    assert!(pages.allocate(1, 1).is_none());
}

/// Verifies LRU page selection, bounded capacity, overflow rejection and absent-page safety.
#[test]
fn pages_evict_least_recently_used_layer_and_reject_oversized_glyphs() {
    let mut pages = AtlasPages::new(8, 3);
    assert_eq!(pages.used(), 0);
    pages.begin_frame();
    for expected in 0..3 {
        assert_eq!(pages.allocate(6, 6).unwrap().page, expected);
    }
    pages.begin_frame();
    pages.pin(0);
    pages.begin_frame();
    pages.pin(1);
    let recycled = pages.allocate(6, 6).unwrap();
    assert_eq!(recycled.page, 2);
    assert!(recycled.evicted);
    assert!(pages.allocate(7, 1).is_none());
    assert!(pages.allocate(1, 7).is_none());
    assert!(pages.allocate(u32::MAX, 1).is_none());
    assert!(pages.allocate(1, u32::MAX).is_none());
    pages.pin(999);
    assert!(AtlasPages::new(8, 0).allocate(1, 1).is_none());
}

/// Checks that a rejected tall row leaves space available for a smaller glyph.
#[test]
fn failed_shelf_reservation_does_not_discard_remaining_row_space() {
    let mut pages = AtlasPages::new(10, 1);
    pages.begin_frame();
    assert_eq!(pages.allocate(4, 6).unwrap().origin, [0, 0]);
    assert!(pages.allocate(4, 2).is_none());
    assert_eq!(pages.allocate(2, 2).unwrap().origin, [6, 0]);
}
