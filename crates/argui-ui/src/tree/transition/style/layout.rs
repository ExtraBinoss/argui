use crate::binding::LayoutTarget;
use crate::binding::layout::set_layout_value;
use crate::state::StateValue;
use crate::{NodeId, PropertyKey};

use super::super::{TransitionRegistry, TransitionTarget};

pub(super) fn targets(style: &crate::LayoutStyle) -> Vec<LayoutTarget> {
    let mut values = vec![
        LayoutTarget::PaddingLeft,
        LayoutTarget::PaddingRight,
        LayoutTarget::PaddingTop,
        LayoutTarget::PaddingBottom,
        LayoutTarget::Gap,
        LayoutTarget::Grow,
        LayoutTarget::Shrink,
    ];
    add_dimension(
        &mut values,
        style.size.width,
        LayoutTarget::WidthPx,
        LayoutTarget::WidthPercent,
    );
    add_dimension(
        &mut values,
        style.size.height,
        LayoutTarget::HeightPx,
        LayoutTarget::HeightPercent,
    );
    add_auto_length(
        &mut values,
        style.min_size.width,
        LayoutTarget::MinWidthPx,
        LayoutTarget::MinWidthPercent,
    );
    add_auto_length(
        &mut values,
        style.min_size.height,
        LayoutTarget::MinHeightPx,
        LayoutTarget::MinHeightPercent,
    );
    add_auto_length(
        &mut values,
        style.max_size.width,
        LayoutTarget::MaxWidthPx,
        LayoutTarget::MaxWidthPercent,
    );
    add_auto_length(
        &mut values,
        style.max_size.height,
        LayoutTarget::MaxHeightPx,
        LayoutTarget::MaxHeightPercent,
    );
    add_auto_length(
        &mut values,
        style.inset.left,
        LayoutTarget::InsetLeftPx,
        LayoutTarget::InsetLeftPx,
    );
    add_auto_length(
        &mut values,
        style.inset.right,
        LayoutTarget::InsetRightPx,
        LayoutTarget::InsetRightPx,
    );
    add_auto_length(
        &mut values,
        style.inset.top,
        LayoutTarget::InsetTopPx,
        LayoutTarget::InsetTopPx,
    );
    add_auto_length(
        &mut values,
        style.inset.bottom,
        LayoutTarget::InsetBottomPx,
        LayoutTarget::InsetBottomPx,
    );
    values
}

fn add_dimension(
    values: &mut Vec<LayoutTarget>,
    value: crate::Dimension,
    pixels: LayoutTarget,
    percent: LayoutTarget,
) {
    match value.expand() {
        taffy::ExpandedDimension::Length(_) => values.push(pixels),
        taffy::ExpandedDimension::Percent(_) => values.push(percent),
        _ => {}
    }
}

fn add_auto_length(
    values: &mut Vec<LayoutTarget>,
    value: crate::LengthPercentageAuto,
    pixels: LayoutTarget,
    percent: LayoutTarget,
) {
    match value.expand() {
        taffy::ExpandedLengthPercentageAuto::Length(_) => values.push(pixels),
        taffy::ExpandedLengthPercentageAuto::Percent(_) => values.push(percent),
        _ => {}
    }
}

pub(in crate::tree) fn apply(
    registry: &TransitionRegistry,
    node: NodeId,
    style: &mut crate::LayoutStyle,
) {
    registry.visit(
        TransitionTarget::Element(node),
        |key| key.impact() == crate::BindingImpact::Layout,
        |key, value| match (key, value) {
            (PropertyKey::LayoutStyle, StateValue::LayoutStyle(value)) => *style = *value,
            (PropertyKey::Layout(target), StateValue::F32(value)) => {
                set_layout_value(style, *target, value);
            }
            _ => {}
        },
    );
}
