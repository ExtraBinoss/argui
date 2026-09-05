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
        VList::new("list", 20.0, 100.0, offset).build_with_header(
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
    assert!(scrolled.children.iter().all(|child| child.scroll.is_none()));
}
