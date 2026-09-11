use argui_core::{CaretAffinity, TextPosition};

use crate::{NodeId, SelectionGranularity};

use super::{EditResult, TextInputFilter, TextInputState, TextSelection};

pub(crate) struct RetainedInput {
    pub node: NodeId,
    pub value: String,
    pub multiline: bool,
    pub read_only: bool,
    pub filter: TextInputFilter,
    pub privacy: super::TextPrivacy,
    pub history: Option<super::HistoryConfig>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextInputStates {
    states: Vec<(NodeId, TextInputState)>,
    drag: Option<TextDrag>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TextDrag {
    node: NodeId,
    granularity: SelectionGranularity,
    origin: (usize, usize),
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

    pub fn sync(&mut self, inputs: impl IntoIterator<Item = RetainedInput>) {
        let inputs = inputs.into_iter().collect::<Vec<_>>();
        self.states
            .retain(|(node, _)| inputs.iter().any(|input| input.node == *node));
        if self.drag.is_some_and(|drag| self.get(drag.node).is_none()) {
            self.drag = None;
        }
        for RetainedInput {
            node,
            value,
            multiline,
            read_only,
            filter,
            privacy,
            history,
        } in inputs
        {
            if let Some(state) = self.get_mut(node) {
                state.sync(&value, multiline, read_only, filter);
            } else {
                self.states.push((
                    node,
                    TextInputState::new(value, multiline, read_only, filter),
                ));
            }
            self.get_mut(node)
                .expect("editor was inserted")
                .set_privacy(privacy);
            if let Some(history) = history {
                self.get_mut(node)
                    .expect("editor was inserted")
                    .configure_history(history);
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
        self.begin_selection(node, position, extend, SelectionGranularity::Character)
    }

    pub fn begin_selection(
        &mut self,
        node: NodeId,
        position: TextPosition,
        extend: bool,
        granularity: SelectionGranularity,
    ) -> Option<EditResult> {
        let state = self.get_mut(node)?;
        let (start, end) = state.unit_range(position, granularity);
        let result = if granularity == SelectionGranularity::Character {
            state.place_position(position, extend)
        } else if extend {
            let anchor = state.anchor.map_or(state.cursor, |position| position.index);
            state.select_range(anchor, if start < anchor { start } else { end })
        } else {
            state.select_range(start, end)
        };
        let origin = state.selection().unwrap_or((state.cursor, state.cursor));
        self.drag = Some(TextDrag {
            node,
            granularity,
            origin,
        });
        Some(result)
    }

    pub fn drag(&mut self, node: NodeId, index: usize) -> Option<EditResult> {
        self.drag_position(node, TextPosition::new(index, CaretAffinity::Before))
    }

    pub fn drag_position(&mut self, node: NodeId, position: TextPosition) -> Option<EditResult> {
        let drag = self.drag.filter(|drag| drag.node == node)?;
        let state = self.get_mut(node)?;
        if drag.granularity == SelectionGranularity::Character {
            return Some(state.place_position(position, true));
        }
        let (start, end) = state.unit_range(position, drag.granularity);
        Some(if start < drag.origin.0 {
            state.select_range(drag.origin.1, start)
        } else {
            state.select_range(drag.origin.0, end.max(drag.origin.1))
        })
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
