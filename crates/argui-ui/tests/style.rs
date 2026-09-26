use argui_text::{TextAlign, TextOverflow};
use argui_ui::{
    AlignItems, Axes, BoxSizing, Dimension, Dimensions, Display, Element, FlexDirection, FlexWrap,
    JustifyContent, LengthPercentage, LengthPercentageAuto, Overflow, ScrollbarGutter, length,
    percent, sides,
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

#[test]
fn layout_builders_cover_overflow_dimensions_and_text_alignment() {
    let layout = Element::container([])
        .overflow(Axes {
            x: Overflow::Auto,
            y: Overflow::Scroll,
        })
        .scrollbar_gutter(ScrollbarGutter::Stable)
        .scrollbar_width(-4.0)
        .box_sizing(BoxSizing::ContentBox)
        .inset(sides(4.0, 8.0))
        .size(Dimensions {
            width: Dimension::length(320.0),
            height: Dimension::length(180.0),
        })
        .min_size(Dimensions {
            width: LengthPercentageAuto::length(120.0),
            height: LengthPercentageAuto::length(60.0),
        })
        .max_size(Dimensions {
            width: LengthPercentageAuto::length(640.0),
            height: LengthPercentageAuto::length(360.0),
        })
        .aspect_ratio(16.0 / 9.0)
        .gaps(Dimensions {
            width: LengthPercentage::length(6.0),
            height: LengthPercentage::length(10.0),
        })
        .flex_basis(Dimension::length(240.0));

    assert_eq!(layout.style.overflow.x, Overflow::Auto);
    assert_eq!(layout.style.overflow.y, Overflow::Scroll);
    assert_eq!(layout.style.scrollbar_gutter, ScrollbarGutter::Stable);
    assert_eq!(layout.style.scrollbar_width, 0.0);
    assert_eq!(layout.style.box_sizing, BoxSizing::ContentBox);
    assert_eq!(layout.style.inset, sides(4.0, 8.0));
    assert_eq!(layout.style.size.width, Dimension::length(320.0));
    assert_eq!(
        layout.style.min_size.width,
        LengthPercentageAuto::length(120.0)
    );
    assert_eq!(
        layout.style.max_size.height,
        LengthPercentageAuto::length(360.0)
    );
    assert_eq!(layout.style.aspect_ratio, Some(16.0 / 9.0));
    assert_eq!(layout.style.gap.width, LengthPercentage::length(6.0));
    assert_eq!(layout.style.gap.height, LengthPercentage::length(10.0));
    assert_eq!(layout.style.flex_basis, Dimension::length(240.0));

    let text = Element::text("align me")
        .text_align(TextAlign::Center)
        .text_overflow(TextOverflow::Ellipsis(Default::default()));
    if let argui_ui::ElementKind::Text { style, .. } = &text.kind {
        assert_eq!(style.align, TextAlign::Center);
        assert_eq!(style.overflow, TextOverflow::Ellipsis(Default::default()));
    } else {
        panic!("text builder must preserve a text element");
    }
}

use argui_ui::{LayoutInsets, PositionInsets, UiTree, WritingDirection};

#[test]
/// Logical sides follow an inherited direction on every supported inset kind.
fn logical_insets_follow_inherited_direction_without_changing_physical_sides() {
    for (direction, expected_left, expected_right) in [
        (WritingDirection::Ltr, 11.0, 17.0),
        (WritingDirection::Rtl, 17.0, 11.0),
    ] {
        let insets = LayoutInsets {
            top: 3.0,
            right: 0.0,
            bottom: 5.0,
            left: 0.0,
            start: Some(11.0),
            end: Some(17.0),
        };
        let root = Element::column([Element::container([])
            .layout_padding(insets)
            .layout_margin(insets)
            .layout_inset(PositionInsets {
                top: Some(3.0),
                start: Some(11.0),
                end: Some(17.0),
                ..PositionInsets::default()
            })])
        .direction_scope(direction);
        let tree = UiTree::new(root);
        let child = &tree.root().children[0];
        let style = tree.resolved_layout_style(tree.node_id_at(1).unwrap(), child);

        assert_eq!(
            style.padding.left,
            argui_ui::LengthPercentage::length(expected_left)
        );
        assert_eq!(
            style.padding.right,
            argui_ui::LengthPercentage::length(expected_right)
        );
        assert_eq!(
            style.margin.left,
            argui_ui::LengthPercentageAuto::length(expected_left)
        );
        assert_eq!(
            style.margin.right,
            argui_ui::LengthPercentageAuto::length(expected_right)
        );
        assert_eq!(
            style.inset.left,
            argui_ui::LengthPercentageAuto::length(expected_left)
        );
        assert_eq!(
            style.inset.right,
            argui_ui::LengthPercentageAuto::length(expected_right)
        );
        assert_eq!(style.inset.bottom, argui_ui::LengthPercentageAuto::auto());
        assert_eq!(style.padding.top, argui_ui::LengthPercentage::length(3.0));
        assert_eq!(
            style.padding.bottom,
            argui_ui::LengthPercentage::length(5.0)
        );
    }
}

#[test]
/// Missing absolute insets retain `auto` instead of stretching the element.
fn position_insets_preserve_unspecified_sides() {
    let root = Element::container([]).layout_inset(PositionInsets {
        start: Some(20.0),
        ..PositionInsets::default()
    });
    let tree = UiTree::new(root);
    let style = tree.resolved_layout_style(tree.node_ids()[0], tree.root());
    assert_eq!(
        style.inset.left,
        argui_ui::LengthPercentageAuto::length(20.0)
    );
    assert_eq!(style.inset.right, argui_ui::LengthPercentageAuto::auto());
    assert_eq!(style.inset.top, argui_ui::LengthPercentageAuto::auto());
    assert_eq!(style.inset.bottom, argui_ui::LengthPercentageAuto::auto());
}
