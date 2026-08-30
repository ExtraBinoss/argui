use argui_ui::{
    AlignItems, Display, Element, FlexDirection, FlexWrap, JustifyContent, length, percent, sides,
};

#[test]
fn layout_builders_expose_css_shaped_flex_properties() {
    let element = Element::container([])
        .display(Display::Flex)
        .width(length(320.0))
        .height(percent(1.0))
        .flex_direction(FlexDirection::Row)
        .flex_wrap(FlexWrap::Wrap)
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::SPACE_BETWEEN)
        .padding(sides(12.0, 8.0))
        .gap(6.0)
        .grow(1.0)
        .shrink(0.0);

    assert_eq!(element.style.display, Display::Flex);
    assert_eq!(element.style.size.width, length(320.0));
    assert_eq!(element.style.size.height, percent(1.0));
    assert_eq!(element.style.flex_direction, FlexDirection::Row);
    assert_eq!(element.style.flex_wrap, FlexWrap::Wrap);
    assert_eq!(element.style.align_items, Some(AlignItems::CENTER));
    assert_eq!(
        element.style.justify_content,
        Some(JustifyContent::SPACE_BETWEEN)
    );
    assert_eq!(element.style.padding, sides(12.0, 8.0));
    assert_eq!(element.style.gap.width, length(6.0));
    assert_eq!(element.style.gap.height, length(6.0));
    assert_eq!(element.style.flex_grow, 1.0);
    assert_eq!(element.style.flex_shrink, 0.0);
}

#[test]
fn explicit_zero_minimums_allow_nested_scroll_hosts_to_shrink() {
    let element = Element::container([])
        .min_width(length(0.0))
        .min_height(length(0.0));
    assert_eq!(element.style.min_size.width, length(0.0));
    assert_eq!(element.style.min_size.height, length(0.0));
}
