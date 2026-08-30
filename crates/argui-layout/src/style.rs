use argui_ui::{Align, Direction, Edges, Justify, LayoutStyle, Length, Position, Wrap};
use taffy::{
    AlignItems, Dimension, FlexDirection, FlexWrap, JustifyContent, LengthPercentage,
    LengthPercentageAuto, Style,
    geometry::{Rect, Size},
};

pub(crate) fn taffy_style(style: &LayoutStyle) -> Style {
    Style {
        size: Size {
            width: dimension(style.width),
            height: dimension(style.height),
        },
        min_size: Size {
            width: min_max_dimension(style.min_width),
            height: min_max_dimension(style.min_height),
        },
        max_size: Size {
            width: min_max_dimension(style.max_width),
            height: min_max_dimension(style.max_height),
        },
        flex_direction: match style.direction {
            Direction::Row => FlexDirection::Row,
            Direction::Column => FlexDirection::Column,
        },
        flex_wrap: match style.wrap {
            Wrap::NoWrap => FlexWrap::NoWrap,
            Wrap::Wrap => FlexWrap::Wrap,
            Wrap::Reverse => FlexWrap::WrapReverse,
        },
        align_items: Some(match style.align {
            Align::Start => AlignItems::START,
            Align::Center => AlignItems::CENTER,
            Align::End => AlignItems::END,
            Align::Stretch => AlignItems::STRETCH,
        }),
        justify_content: Some(match style.justify {
            Justify::Start => JustifyContent::START,
            Justify::Center => JustifyContent::CENTER,
            Justify::End => JustifyContent::END,
            Justify::SpaceBetween => JustifyContent::SPACE_BETWEEN,
        }),
        position: match style.position {
            Position::Relative => taffy::Position::Relative,
            Position::Absolute => taffy::Position::Absolute,
        },
        inset: Rect {
            left: min_max_dimension(style.inset.left),
            right: min_max_dimension(style.inset.right),
            top: min_max_dimension(style.inset.top),
            bottom: min_max_dimension(style.inset.bottom),
        },
        padding: padding(style.padding),
        gap: Size {
            width: LengthPercentage::length(style.gap),
            height: LengthPercentage::length(style.gap),
        },
        flex_grow: style.grow,
        flex_shrink: style.shrink,
        ..Style::default()
    }
}

const fn dimension(value: Length) -> Dimension {
    match value {
        Length::Auto => Dimension::auto(),
        Length::Px(value) => Dimension::length(value),
        Length::Percent(value) => Dimension::percent(value),
    }
}

const fn min_max_dimension(value: Length) -> LengthPercentageAuto {
    match value {
        Length::Auto => LengthPercentageAuto::auto(),
        Length::Px(value) => LengthPercentageAuto::length(value),
        Length::Percent(value) => LengthPercentageAuto::percent(value),
    }
}

const fn padding(edges: Edges) -> Rect<LengthPercentage> {
    Rect {
        left: LengthPercentage::length(edges.left),
        right: LengthPercentage::length(edges.right),
        top: LengthPercentage::length(edges.top),
        bottom: LengthPercentage::length(edges.bottom),
    }
}
