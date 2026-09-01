use argui_core::{CaretAffinity, TextPosition};

use crate::NodeId;

use super::{EditResult, TextInputFilter, TextInputState, TextSelection};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextInputStates {
    states: Vec<(NodeId, TextInputState)>,
    drag: Option<NodeId>,
}

impl TextInputStates {
    pub fn get(&self, node: NodeId) -> Option<&TextInputState> {
        self.states
            .iter()
            .find_map(|(id, state)| (*id == node).then_some(state))
    }

    pub fn get_mut(&mut self, node: NodeId) -> Option<&mut TextInputState> {
        self.states
            .iter_mut()
            .find_map(|(id, state)| (*id == node).then_some(state))
    }

    pub fn should_reveal_cursor(&self, node: NodeId) -> bool {
        self.get(node).is_some_and(|state| state.reveal_cursor)
    }

    pub fn request_cursor_reveal(&mut self, node: NodeId) {
        if let Some(state) = self.get_mut(node) {
            state.reveal_cursor = true;
        }
    }

    pub fn clear_cursor_reveals(&mut self) {
        for (_, state) in &mut self.states {
            state.reveal_cursor = false;
        }
    }

    pub fn sync(
        &mut self,
        inputs: impl IntoIterator<Item = (NodeId, String, bool, bool, TextInputFilter)>,
    ) {
        let inputs = inputs.into_iter().collect::<Vec<_>>();
        self.states
            .retain(|(node, _)| inputs.iter().any(|(id, _, _, _, _)| id == node));
        if self.drag.is_some_and(|node| self.get(node).is_none()) {
            self.drag = None;
        }
        for (node, value, multiline, read_only, filter) in inputs {
            if let Some(state) = self.get_mut(node) {
                state.sync(&value, multiline, read_only, filter);
            } else {
                self.states.push((
                    node,
                    TextInputState::new(value, multiline, read_only, filter),
                ));
            }
        }
    }

    pub fn place(&mut self, node: NodeId, index: usize, extend: bool) -> Option<EditResult> {
        self.place_position(
            node,
            TextPosition::new(index, CaretAffinity::Before),
            extend,
        )
    }

    pub fn place_position(
        &mut self,
        node: NodeId,
        position: TextPosition,
        extend: bool,
    ) -> Option<EditResult> {
        let result = self.get_mut(node)?.place_position(position, extend);
        self.drag = Some(node);
        Some(result)
    }

    pub fn drag(&mut self, node: NodeId, index: usize) -> Option<EditResult> {
        self.drag_position(node, TextPosition::new(index, CaretAffinity::Before))
    }

    pub fn drag_position(&mut self, node: NodeId, position: TextPosition) -> Option<EditResult> {
        if self.drag != Some(node) {
            return None;
        }
        self.get_mut(node)
            .map(|state| state.place_position(position, true))
    }

    pub fn move_to(&mut self, node: NodeId, index: usize, extend: bool) -> Option<EditResult> {
        self.move_to_position(
            node,
            TextPosition::new(index, CaretAffinity::Before),
            extend,
        )
    }

    pub fn move_to_position(
        &mut self,
        node: NodeId,
        position: TextPosition,
        extend: bool,
    ) -> Option<EditResult> {
        self.get_mut(node)
            .map(|state| state.place_position(position, extend))
    }

    pub fn select(&mut self, node: NodeId, selection: TextSelection) -> Option<EditResult> {
        self.get_mut(node).map(|state| state.select(selection))
    }

    pub const fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub fn release(&mut self) -> bool {
        self.drag.take().is_some()
    }
}
