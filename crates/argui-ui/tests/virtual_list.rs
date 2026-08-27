use argui_ui::{Element, VirtualList};

#[test]
fn million_row_lists_only_build_the_visible_window() {
    let list = VirtualList::new(1_000_000, 32.0, 320.0).overscan(2);
    let window = list.window(16_000_000.0);
    let tree = list.build("million", 16_000_000.0, |index| {
        Element::text(format!("Row {index}"))
    });

    assert_eq!(window.range, 499_992..500_018);
    assert_eq!(window.total, 32_000_000.0);
    assert_eq!(tree.children[0].children.len(), window.range.len() + 2);
    assert!(tree.children[0].children.len() < 100);
}

#[test]
fn visible_windows_stay_stable_inside_a_scroll_chunk() {
    let list = VirtualList::new(1_000_000, 36.0, 260.0).overscan(3);

    assert_eq!(list.window(0.0).range, list.window(8.0 * 36.0).range);
    assert_ne!(list.window(0.0).range, list.window(9.0 * 36.0).range);
}

#[test]
fn bounded_lists_can_retain_every_row_for_paint_only_scrolling() {
    let list = VirtualList::new(109, 28.0, 236.0).overscan(109);

    assert_eq!(list.window(0.0).range, 0..109);
    assert_eq!(list.window(2_816.0).range, 0..109);
}
