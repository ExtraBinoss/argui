use crate::{Dimension, LayoutStyle, LengthPercentage, LengthPercentageAuto};
use taffy::{ExpandedDimension, ExpandedLengthPercentage, ExpandedLengthPercentageAuto};

use super::LayoutTarget;

pub(crate) fn layout_value(style: &LayoutStyle, target: LayoutTarget) -> f32 {
    match target {
        LayoutTarget::WidthPx | LayoutTarget::WidthPercent => dimension_value(style.size.width),
        LayoutTarget::HeightPx | LayoutTarget::HeightPercent => dimension_value(style.size.height),
        LayoutTarget::MinWidthPx | LayoutTarget::MinWidthPercent => {
            auto_value(style.min_size.width)
        }
        LayoutTarget::MinHeightPx | LayoutTarget::MinHeightPercent => {
            auto_value(style.min_size.height)
        }
        LayoutTarget::MaxWidthPx | LayoutTarget::MaxWidthPercent => {
            auto_value(style.max_size.width)
        }
        LayoutTarget::MaxHeightPx | LayoutTarget::MaxHeightPercent => {
            auto_value(style.max_size.height)
        }
        LayoutTarget::PaddingLeft => length_value(style.padding.left),
        LayoutTarget::PaddingRight => length_value(style.padding.right),
        LayoutTarget::PaddingTop => length_value(style.padding.top),
        LayoutTarget::PaddingBottom => length_value(style.padding.bottom),
        LayoutTarget::Gap => length_value(style.gap.width),
        LayoutTarget::Grow => style.flex_grow,
        LayoutTarget::Shrink => style.flex_shrink,
        LayoutTarget::InsetLeftPx => auto_value(style.inset.left),
        LayoutTarget::InsetRightPx => auto_value(style.inset.right),
        LayoutTarget::InsetTopPx => auto_value(style.inset.top),
        LayoutTarget::InsetBottomPx => auto_value(style.inset.bottom),
    }
}

pub(crate) fn set_layout_value(style: &mut LayoutStyle, target: LayoutTarget, value: f32) {
    match target {
        LayoutTarget::WidthPx => style.size.width = Dimension::length(value),
        LayoutTarget::WidthPercent => style.size.width = Dimension::percent(value),
        LayoutTarget::HeightPx => style.size.height = Dimension::length(value),
        LayoutTarget::HeightPercent => style.size.height = Dimension::percent(value),
        LayoutTarget::MinWidthPx => style.min_size.width = LengthPercentageAuto::length(value),
        LayoutTarget::MinWidthPercent => {
            style.min_size.width = LengthPercentageAuto::percent(value)
        }
        LayoutTarget::MinHeightPx => style.min_size.height = LengthPercentageAuto::length(value),
        LayoutTarget::MinHeightPercent => {
            style.min_size.height = LengthPercentageAuto::percent(value)
        }
        LayoutTarget::MaxWidthPx => style.max_size.width = LengthPercentageAuto::length(value),
        LayoutTarget::MaxWidthPercent => {
            style.max_size.width = LengthPercentageAuto::percent(value)
        }
        LayoutTarget::MaxHeightPx => style.max_size.height = LengthPercentageAuto::length(value),
        LayoutTarget::MaxHeightPercent => {
            style.max_size.height = LengthPercentageAuto::percent(value)
        }
        LayoutTarget::PaddingLeft => style.padding.left = LengthPercentage::length(value),
        LayoutTarget::PaddingRight => style.padding.right = LengthPercentage::length(value),
        LayoutTarget::PaddingTop => style.padding.top = LengthPercentage::length(value),
        LayoutTarget::PaddingBottom => style.padding.bottom = LengthPercentage::length(value),
        LayoutTarget::Gap => {
            style.gap.width = LengthPercentage::length(value);
            style.gap.height = LengthPercentage::length(value);
        }
        LayoutTarget::Grow => style.flex_grow = value,
        LayoutTarget::Shrink => style.flex_shrink = value,
        LayoutTarget::InsetLeftPx => style.inset.left = LengthPercentageAuto::length(value),
        LayoutTarget::InsetRightPx => style.inset.right = LengthPercentageAuto::length(value),
        LayoutTarget::InsetTopPx => style.inset.top = LengthPercentageAuto::length(value),
        LayoutTarget::InsetBottomPx => style.inset.bottom = LengthPercentageAuto::length(value),
    }
}

fn dimension_value(value: Dimension) -> f32 {
    match value.expand() {
        ExpandedDimension::Length(value) | ExpandedDimension::Percent(value) => value,
        _ => 0.0,
    }
}

fn auto_value(value: LengthPercentageAuto) -> f32 {
    match value.expand() {
        ExpandedLengthPercentageAuto::Length(value)
        | ExpandedLengthPercentageAuto::Percent(value) => value,
        ExpandedLengthPercentageAuto::Auto => 0.0,
    }
}

fn length_value(value: LengthPercentage) -> f32 {
    match value.expand() {
        ExpandedLengthPercentage::Length(value) | ExpandedLengthPercentage::Percent(value) => value,
    }
}
