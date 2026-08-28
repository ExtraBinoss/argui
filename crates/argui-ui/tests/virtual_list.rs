use argui_ui::{Element, VariableList, VirtualList};

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
fn variable_list_keeps_the_visible_anchor_when_earlier_rows_are_measured() {
    let mut list = VariableList::new(1_000_000, 30.0, 300.0);
    let offset = 30_000.0;
    let anchor = list.window(offset).range.start;
    let update = list.measure(anchor.saturating_sub(100), 90.0, offset);

    assert!(update.changed);
    assert_eq!(list.window(update.corrected_offset).range.start, anchor);
    assert!(list.window(update.corrected_offset).range.len() < 32);
}

#[test]
fn variable_list_handles_invalid_measurements_without_nan() {
    let mut list = VariableList::new(10, 0.0, 100.0);
    let update = list.measure(2, f32::NAN, 0.0);
    assert!(update.corrected_offset.is_finite());
    assert!(list.total_extent().is_finite());
    assert_eq!(list.item_extent(99), None);
    assert!(!list.is_measured(99));
    assert!(list.estimated_extent() > 0.0);
    let unchanged = list.measure(99, 12.0, 4.0);
    assert!(!unchanged.changed);
    let tree = list.build("variable", f32::INFINITY, |index| {
        Element::text(index.to_string())
    });
    assert!(tree.children[0].children.len() < 20);
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
