use crate::{NodeId, ScrollbarPartStyle, ScrollbarStyle, StyleCondition, VisualStates};

use super::super::{NodeSpec, ResolvedProperty, TransitionTarget};
use super::states::{ScopeStack, StateContext};
use super::{apply_target, quad_values, resolved};

pub(super) struct ScrollbarContext<'a> {
    pub node: NodeId,
    pub node_index: usize,
    pub enabled: bool,
    pub states: &'a StateContext,
    pub scope_stack: &'a ScopeStack,
    pub states_for: &'a dyn Fn(NodeId, crate::scroll::ScrollbarPart, bool) -> VisualStates,
    pub container_size: &'a dyn Fn(NodeId) -> Option<argui_core::Size>,
}

pub(super) fn collect<'a>(
    context: ScrollbarContext<'_>,
    scrollbar: &'a ScrollbarStyle,
    output: &mut Vec<NodeSpec<'a>>,
) {
    push(
        TransitionTarget::ScrollbarTrack(context.node),
        &scrollbar.track,
        (context.states_for)(
            context.node,
            crate::scroll::ScrollbarPart::Track,
            context.enabled,
        ),
        |condition, visual| {
            context.states.matches_part(
                condition,
                context.node_index,
                context.scope_stack,
                visual,
                &context.container_size,
            )
        },
        output,
    );
    push(
        TransitionTarget::ScrollbarThumb(context.node),
        &scrollbar.thumb,
        (context.states_for)(
            context.node,
            crate::scroll::ScrollbarPart::Thumb,
            context.enabled,
        ),
        |condition, visual| {
            context.states.matches_part(
                condition,
                context.node_index,
                context.scope_stack,
                visual,
                &context.container_size,
            )
        },
        output,
    );
}

fn push<'a>(
    target: TransitionTarget,
    part: &'a ScrollbarPartStyle,
    states: VisualStates,
    matches: impl Fn(&StyleCondition, VisualStates) -> bool,
    output: &mut Vec<NodeSpec<'a>>,
) {
    if part.transition.is_none() && !part.has_states() {
        return;
    }
    output.push(NodeSpec {
        target,
        matched: matched(part, states, &matches),
        values: target_values(part, states, &matches),
        transition: part.transition.as_ref(),
    });
}

fn matched(
    part: &ScrollbarPartStyle,
    states: VisualStates,
    matches: &impl Fn(&StyleCondition, VisualStates) -> bool,
) -> Vec<StyleCondition> {
    part.state_rules()
        .iter()
        .filter_map(|rule| matches(&rule.condition, states).then_some(rule.condition.clone()))
        .collect()
}

fn target_values(
    part: &ScrollbarPartStyle,
    states: VisualStates,
    matches: &impl Fn(&StyleCondition, VisualStates) -> bool,
) -> Vec<ResolvedProperty> {
    let matched = matched(part, states, matches);
    let mut values = resolved(quad_values(&part.base));
    for rule in part.state_rules() {
        if matched.contains(&rule.condition) {
            for property in rule.style.values() {
                apply_target(&mut values, property, Some(rule.condition.clone()));
            }
        }
    }
    values
}
