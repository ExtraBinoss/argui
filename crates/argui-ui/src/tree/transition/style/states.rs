use crate::{
    Element, NodeId, State, StateName, StateScopeId, StateSelector, VisualState, VisualStates,
};

pub(super) type ScopeStack = Vec<usize>;

#[derive(Debug)]
pub(super) struct StateContext {
    nodes: Vec<NodeStates>,
    scopes: Vec<ResolvedScope>,
    scope_at: Vec<Option<usize>>,
}

impl StateContext {
    pub(super) fn collect(
        root: &Element,
        ids: &[NodeId],
        states_for: &impl Fn(NodeId) -> VisualStates,
    ) -> Self {
        let mut context = Self {
            nodes: Vec::with_capacity(ids.len()),
            scopes: Vec::new(),
            scope_at: vec![None; ids.len()],
        };
        let mut index = 0;
        context.collect_element(root, &mut index, ids, states_for);
        context
    }

    pub(super) fn enter(&self, node: usize, stack: &mut ScopeStack) {
        if let Some(scope) = self.scope_at[node] {
            stack.push(scope);
        }
    }

    pub(super) fn exit(&self, next_node: usize, stack: &mut ScopeStack) {
        if stack
            .last()
            .is_some_and(|scope| self.scopes[*scope].end == next_node)
        {
            stack.pop();
        }
    }

    pub(super) fn matched(
        &self,
        element: &Element,
        node: usize,
        stack: &ScopeStack,
    ) -> Vec<StateSelector> {
        element
            .state_styles
            .rules()
            .iter()
            .filter_map(|rule| {
                self.matches(rule.selector, node, stack)
                    .then_some(rule.selector)
            })
            .collect()
    }

    pub(super) fn matches_part(
        &self,
        selector: StateSelector,
        node: usize,
        stack: &ScopeStack,
        part_visual: VisualStates,
    ) -> bool {
        match selector {
            StateSelector::Own(State::Visual(state)) => part_visual.contains(state),
            _ => self.matches(selector, node, stack),
        }
    }

    fn collect_element(
        &mut self,
        element: &Element,
        index: &mut usize,
        ids: &[NodeId],
        states_for: &impl Fn(NodeId) -> VisualStates,
    ) -> VisualStates {
        let root = *index;
        let mut visual = states_for(ids[root]);
        if element
            .interaction
            .as_ref()
            .is_some_and(|interaction| !interaction.enabled)
        {
            visual.insert(VisualState::Disabled);
        }
        self.nodes.push(NodeStates {
            visual,
            named: element.active_states.clone(),
        });
        *index += 1;
        let mut aggregate = visual;
        for child in &element.children {
            let child_states = self.collect_element(child, index, ids, states_for);
            for state in [
                VisualState::Hovered,
                VisualState::Focused,
                VisualState::FocusVisible,
                VisualState::Pressed,
            ] {
                if child_states.contains(state) {
                    aggregate.insert(state);
                }
            }
        }
        if let Some(id) = element.state_scope {
            let scope = self.scopes.len();
            self.scopes.push(ResolvedScope {
                id,
                end: *index,
                states: NodeStates {
                    visual: aggregate,
                    named: element.active_states.clone(),
                },
            });
            self.scope_at[root] = Some(scope);
        }
        aggregate
    }

    fn matches(&self, selector: StateSelector, node: usize, stack: &ScopeStack) -> bool {
        let states = match selector {
            StateSelector::Own(_) => &self.nodes[node],
            StateSelector::Scope { scope, .. } => {
                let Some(index) = stack
                    .iter()
                    .rev()
                    .find(|index| self.scopes[**index].id == scope)
                else {
                    return false;
                };
                &self.scopes[*index].states
            }
        };
        let state = match selector {
            StateSelector::Own(state) | StateSelector::Scope { state, .. } => state,
        };
        match state {
            State::Visual(state) => states.visual.contains(state),
            State::Named(state) => states.named.contains(&state),
        }
    }
}

#[derive(Clone, Debug)]
struct NodeStates {
    visual: VisualStates,
    named: Vec<StateName>,
}

#[derive(Clone, Debug)]
struct ResolvedScope {
    id: StateScopeId,
    end: usize,
    states: NodeStates,
}
