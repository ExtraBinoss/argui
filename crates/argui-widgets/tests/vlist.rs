use argui_core::{Color, ColorScheme};
use argui_ui::{Element, ScrollbarGutter};
use argui_widgets::{VList, shadcn};

fn has(root: &Element, key: &str) -> bool {
    root.key.as_deref() == Some(key) || root.children.iter().any(|child| has(child, key))
}

#[test]
fn header_shares_the_row_scrollport_and_offsets_the_virtual_window() {
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Dark);
    let build = |offset| {
        VList::new("list", 20.0, 100.0, offset)
            .propagation(argui_ui::ScrollPropagation::Contain)
            .build_with_header(
                1000,
                theme,
                Element::text("Header").keyed("header"),
                200.0,
                |index| Element::text(index.to_string()).keyed(format!("row-{index}")),
            )
    };
    let first = build(100.0);
    assert!(has(&first, "row-0"));
    let scrolled = build(800.0);
    assert!(has(&scrolled, "header"));
    assert!(has(&scrolled, "row-30"));
    assert!(!has(&scrolled, "row-0"));
    assert!(!has(&scrolled, "row-900"));
    assert_eq!(scrolled.style.scrollbar_gutter, ScrollbarGutter::Stable);
    assert!(scrolled.scroll.is_some());
    assert_eq!(
        scrolled.scroll.as_ref().unwrap().propagation,
        argui_ui::ScrollPropagation::Contain
    );
    assert!(scrolled.children.iter().all(|child| child.scroll.is_none()));
}

#[test]
fn effects_are_opt_in_and_survive_headers_without_mounting_more_rows() {
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Dark);
    let plain = VList::new("list", 20.0, 100.0, 900.0);
    let effect =
        argui_ui::ScrollEffect::new(argui_ui::LayerStyle::new(Default::default()).opacity(0.5));
    let styled = plain.clone().effect(effect.clone());
    let base = plain.build(10_000, theme, |index| Element::text(index.to_string()));
    let built = styled.build_with_header(10_000, theme, Element::text("Header"), 0.0, |index| {
        Element::text(index.to_string())
    });
    assert!(base.scroll.as_ref().unwrap().effects.is_empty());
    assert_eq!(
        base.scroll.as_ref().unwrap().propagation,
        argui_ui::ScrollPropagation::Chain
    );
    assert_eq!(built.scroll.as_ref().unwrap().effects, vec![effect]);
    assert_eq!(built.children.len(), base.children.len() + 1);
    assert!(built.children.len() < 30);
}

#[test]
fn fixed_vlist_only_invalidates_when_its_mounted_window_changes() {
    let list = VList::new("list", 20.0, 100.0, 0.0);

    assert!(!list.window_changed(10_000, 0.0, 80.0));
    assert!(list.window_changed(10_000, 0.0, 120.0));
}

#[test]
fn variable_measurements_survive_rebuild_resize_and_selection() {
    use argui_ui::{VirtualAlignment, VirtualList};
    use argui_widgets::ListState;
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Light);
    let mut config = VirtualList::variable(1000, 20.0, 100.0);
    let row = |index: usize| Element::text(index.to_string());
    let first = VList::variable("variable", &config, 0.0).build(1000, theme, row);
    let update = first.children[0].children[1]
        .virtual_item()
        .as_ref()
        .unwrap()
        .measure_layout(60.0, 400.0);
    assert!(update.changed);
    assert_eq!(update.corrected_offset, 440.0);
    assert_eq!(config.item_extent(0), Some(60.0));
    let mut list = VList::variable("variable", &config, update.corrected_offset);
    list.viewport = 200.0;
    assert_eq!(list.config(1000).viewport_extent(), 200.0);
    assert_eq!(list.config(1000).item_extent(0), Some(60.0));
    assert_eq!(
        list.config(1000).scroll_to(999, VirtualAlignment::End, 0.0),
        19840.0
    );
    let mut state = ListState::default();
    let items = argui_widgets::Collection::new(
        (0..1000).map(|i| argui_widgets::CollectionItem::new(i.to_string(), i.to_string())),
    )
    .unwrap();
    state.select(20, &items, true, Default::default());
    let built = list.build_list(&items, &state, true, theme, row);
    assert!(has(&built, "variable::row::20"));
    assert!(!has(&built, "variable::row::999"));
    assert!(built.children[0].children.len() < 32);
    config.insert(0, 2);
    config.remove(1..2);
    assert_eq!(config.item_extent(1), Some(60.0));
    assert_eq!(
        VList::variable("variable", &config, 0.0)
            .config(1001)
            .item_count(),
        1001
    );
}

#[test]
#[should_panic(expected = "variable list count must match")]
fn variable_lists_reject_a_count_that_disagrees_with_retained_measurements() {
    let config = argui_ui::VirtualList::variable(3, 20.0, 100.0);
    let _ = VList::variable("list", &config, 0.0).config(4);
}
