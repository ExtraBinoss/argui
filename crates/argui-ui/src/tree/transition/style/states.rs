use crate::{
    Element, NodeId, State, StateName, StateScopeId, StateSelector, StyleCondition, VisualState,
    VisualStates,
};

#[derive(Default)]
pub(super) struct ScopeStack {
    states: Vec<usize>,
    containers: Vec<usize>,
}

#[derive(Debug)]
pub(super) struct StateContext {
    nodes: Vec<NodeStates>,
    scopes: Vec<ResolvedScope>,
    scope_at: Vec<Option<usize>>,
    containers: Vec<ResolvedContainer>,
    container_at: Vec<Option<usize>>,
}

impl StateContext {
    pub(super) fn collect(
        root: &Element,
        ids: &[NodeId],
        states_for: &dyn Fn(NodeId) -> VisualStates,
    ) -> Self {
        let mut context = Self {
            nodes: Vec::with_capacity(ids.len()),
            scopes: Vec::new(),
            scope_at: vec![None; ids.len()],
            containers: Vec::new(),
            container_at: vec![None; ids.len()],
        };
        let mut index = 0;
        context.collect_element(root, &mut index, ids, states_for);
        context
    }

    pub(super) fn enter(&self, node: usize, stack: &mut ScopeStack) {
        if let Some(scope) = self.scope_at[node] {
            stack.states.push(scope);
        }
        if let Some(container) = self.container_at[node] {
            stack.containers.push(container);
        }
    }

    pub(super) fn exit(&self, next_node: usize, stack: &mut ScopeStack) {
        if stack
            .states
            .last()
            .is_some_and(|scope| self.scopes[*scope].end == next_node)
        {
            stack.states.pop();
        }
        if stack
            .containers
            .last()
            .is_some_and(|container| self.containers[*container].end == next_node)
        {
            stack.containers.pop();
        }
    }

    pub(super) fn matched(
        &self,
        element: &Element,
        node: usize,
        stack: &ScopeStack,
        container_size: &dyn Fn(NodeId) -> Option<argui_core::Size>,
    ) -> Vec<StyleCondition> {
        element
            .conditional_styles
            .rules()
            .iter()
            .filter_map(|rule| {
                self.matches_condition(&rule.condition, node, stack, container_size)
                    .then_some(rule.condition.clone())
            })
            .collect()
    }

    pub(super) fn matches_part(
        &self,
        condition: &StyleCondition,
        node: usize,
        stack: &ScopeStack,
        part_visual: VisualStates,
        container_size: &dyn Fn(NodeId) -> Option<argui_core::Size>,
    ) -> bool {
        condition.matches(
            &|selector| match selector {
                StateSelector::Own(State::Visual(state)) => part_visual.contains(state),
                _ => self.matches(selector, node, stack),
            },
            &|query| self.matches_container(query, stack, container_size),
        )
    }

    fn collect_element(
        &mut self,
        element: &Element,
        index: &mut usize,
        ids: &[NodeId],
        states_for: &dyn Fn(NodeId) -> VisualStates,
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
        if let Some(id) = element.container_scope {
            let container = self.containers.len();
            self.containers.push(ResolvedContainer {
                id,
                node: ids[root],
                end: *index,
            });
            self.container_at[root] = Some(container);
        }
        aggregate
    }

    fn matches(&self, selector: StateSelector, node: usize, stack: &ScopeStack) -> bool {
        let states = match selector {
            StateSelector::Own(_) => &self.nodes[node],
            StateSelector::Scope { scope, .. } => {
                let Some(index) = stack
                    .states
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

    pub(super) fn matches_condition(
        &self,
        condition: &StyleCondition,
        node: usize,
        stack: &ScopeStack,
        container_size: &dyn Fn(NodeId) -> Option<argui_core::Size>,
    ) -> bool {
        condition.matches(&|selector| self.matches(selector, node, stack), &|query| {
            self.matches_container(query, stack, container_size)
        })
    }

    fn matches_container(
        &self,
        query: crate::ContainerQuery,
        stack: &ScopeStack,
        container_size: &dyn Fn(NodeId) -> Option<argui_core::Size>,
    ) -> bool {
        stack
            .containers
            .iter()
            .rev()
            .map(|index| &self.containers[*index])
            .find(|container| container.id == query.scope())
            .and_then(|container| container_size(container.node))
            .is_some_and(|size| query.matches(size.width, size.height))
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

#[derive(Clone, Copy, Debug)]
struct ResolvedContainer {
    id: crate::ContainerScopeId,
    node: NodeId,
    end: usize,
}
