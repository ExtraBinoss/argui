use crate::{
    NodeId, ScrollbarPartStyle, ScrollbarStyle, StatePropertyValue, VisualState, VisualStates,
};

use super::super::{NodeSpec, TransitionTarget};
use super::{apply_target, quad_values};

pub(super) fn collect<'a>(
    node: NodeId,
    enabled: bool,
    scrollbar: &'a ScrollbarStyle,
    states_for: &dyn Fn(NodeId, crate::scroll::ScrollbarPart, bool) -> VisualStates,
    output: &mut Vec<NodeSpec<'a>>,
) {
    push(
        TransitionTarget::ScrollbarTrack(node),
        &scrollbar.track,
        states_for(node, crate::scroll::ScrollbarPart::Track, enabled),
        output,
    );
    push(
        TransitionTarget::ScrollbarThumb(node),
        &scrollbar.thumb,
        states_for(node, crate::scroll::ScrollbarPart::Thumb, enabled),
        output,
    );
}

fn push<'a>(
    target: TransitionTarget,
    part: &'a ScrollbarPartStyle,
    states: VisualStates,
    output: &mut Vec<NodeSpec<'a>>,
) {
    if part.transition.is_none() && !part.has_states() {
        return;
    }
    output.push(NodeSpec {
        target,
        states,
        values: target_values(part, states),
        transition: part.transition.as_ref(),
    });
}

fn target_values(part: &ScrollbarPartStyle, states: VisualStates) -> Vec<StatePropertyValue> {
    let mut values = quad_values(&part.base);
    for state in [
        VisualState::Focused,
        VisualState::Hovered,
        VisualState::Pressed,
        VisualState::Disabled,
    ] {
        if states.contains(state)
            && let Some(style) = part.state_style(state)
        {
            for property in style.values() {
                apply_target(&mut values, property);
            }
        }
    }
    values
}
