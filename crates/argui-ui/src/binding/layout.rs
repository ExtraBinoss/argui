use crate::{LayoutStyle, Length};

use super::LayoutTarget;

pub(crate) fn layout_value(style: &LayoutStyle, target: LayoutTarget) -> f32 {
    match target {
        LayoutTarget::WidthPx | LayoutTarget::WidthPercent => length_value(style.width),
        LayoutTarget::HeightPx | LayoutTarget::HeightPercent => length_value(style.height),
        LayoutTarget::MinWidthPx | LayoutTarget::MinWidthPercent => length_value(style.min_width),
        LayoutTarget::MinHeightPx | LayoutTarget::MinHeightPercent => {
            length_value(style.min_height)
        }
        LayoutTarget::MaxWidthPx | LayoutTarget::MaxWidthPercent => length_value(style.max_width),
        LayoutTarget::MaxHeightPx | LayoutTarget::MaxHeightPercent => {
            length_value(style.max_height)
        }
        LayoutTarget::PaddingLeft => style.padding.left,
        LayoutTarget::PaddingRight => style.padding.right,
        LayoutTarget::PaddingTop => style.padding.top,
        LayoutTarget::PaddingBottom => style.padding.bottom,
        LayoutTarget::Gap => style.gap,
        LayoutTarget::Grow => style.grow,
        LayoutTarget::Shrink => style.shrink,
        LayoutTarget::InsetLeftPx => length_value(style.inset.left),
        LayoutTarget::InsetRightPx => length_value(style.inset.right),
        LayoutTarget::InsetTopPx => length_value(style.inset.top),
        LayoutTarget::InsetBottomPx => length_value(style.inset.bottom),
    }
}

pub(crate) fn set_layout_value(style: &mut LayoutStyle, target: LayoutTarget, value: f32) {
    match target {
        LayoutTarget::WidthPx => style.width = Length::Px(value),
        LayoutTarget::WidthPercent => style.width = Length::Percent(value),
        LayoutTarget::HeightPx => style.height = Length::Px(value),
        LayoutTarget::HeightPercent => style.height = Length::Percent(value),
        LayoutTarget::MinWidthPx => style.min_width = Length::Px(value),
        LayoutTarget::MinWidthPercent => style.min_width = Length::Percent(value),
        LayoutTarget::MinHeightPx => style.min_height = Length::Px(value),
        LayoutTarget::MinHeightPercent => style.min_height = Length::Percent(value),
        LayoutTarget::MaxWidthPx => style.max_width = Length::Px(value),
        LayoutTarget::MaxWidthPercent => style.max_width = Length::Percent(value),
        LayoutTarget::MaxHeightPx => style.max_height = Length::Px(value),
        LayoutTarget::MaxHeightPercent => style.max_height = Length::Percent(value),
        LayoutTarget::PaddingLeft => style.padding.left = value,
        LayoutTarget::PaddingRight => style.padding.right = value,
        LayoutTarget::PaddingTop => style.padding.top = value,
        LayoutTarget::PaddingBottom => style.padding.bottom = value,
        LayoutTarget::Gap => style.gap = value,
        LayoutTarget::Grow => style.grow = value,
        LayoutTarget::Shrink => style.shrink = value,
        LayoutTarget::InsetLeftPx => style.inset.left = Length::Px(value),
        LayoutTarget::InsetRightPx => style.inset.right = Length::Px(value),
        LayoutTarget::InsetTopPx => style.inset.top = Length::Px(value),
        LayoutTarget::InsetBottomPx => style.inset.bottom = Length::Px(value),
    }
}

fn length_value(length: Length) -> f32 {
    match length {
        Length::Px(value) | Length::Percent(value) => value,
        Length::Auto => 0.0,
    }
}
