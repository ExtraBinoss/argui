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
    assert_eq!(built.scroll.as_ref().unwrap().effects, vec![effect]);
    assert_eq!(built.children.len(), base.children.len() + 1);
    assert!(built.children.len() < 30);
}
