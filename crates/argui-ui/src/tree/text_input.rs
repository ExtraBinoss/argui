use crate::text_input::TextInputState;
use crate::{InteractionUpdate, NodeId, SelectionGranularity, TextSelectionRequest, UiTree};
use argui_core::{ImeInput, TextPosition};

impl UiTree {
    #[must_use]
    pub fn text_input_value(&self, node: NodeId) -> Option<&str> {
        self.text_inputs.get(node).map(TextInputState::value)
    }

    #[must_use]
    pub fn text_input_display(&self, node: NodeId) -> Option<String> {
        self.text_inputs.get(node).map(TextInputState::display)
    }

    #[must_use]
    pub fn text_input_cursor(&self, node: NodeId) -> Option<usize> {
        self.text_inputs
            .get(node)
            .map(TextInputState::display_cursor)
    }

    #[must_use]
    pub fn text_input_position(&self, node: NodeId) -> Option<TextPosition> {
        self.text_inputs
            .get(node)
            .map(TextInputState::display_position)
    }

    #[must_use]
    pub fn text_input_selection(&self, node: NodeId) -> Option<(usize, usize)> {
        self.text_inputs
            .get(node)
            .and_then(TextInputState::selection)
    }

    #[must_use]
    pub fn text_input_selection_positions(
        &self,
        node: NodeId,
    ) -> Option<(TextPosition, TextPosition)> {
        self.text_inputs
            .get(node)
            .and_then(TextInputState::selection_positions)
    }

    #[must_use]
    pub fn text_input_should_reveal_cursor(&self, node: NodeId) -> bool {
        self.text_inputs.should_reveal_cursor(node)
    }

    pub fn select_text(&mut self, request: TextSelectionRequest) -> InteractionUpdate {
        let node = match request.target {
            crate::FocusTarget::Node(node) => node,
            crate::FocusTarget::Key(key) => {
                let Some(node) = self
                    .node_ids
                    .iter()
                    .copied()
                    .find(|node| self.key_for(*node) == Some(key.as_str()))
                else {
                    return InteractionUpdate::default();
                };
                node
            }
        };
        self.text_inputs
            .select(node, request.selection)
            .map_or_else(InteractionUpdate::default, |result| {
                self.text_input_update(node, result)
            })
    }
    pub fn mark_text_input_layout_clean(&mut self) {
        self.text_inputs.clear_cursor_reveals();
    }

    #[must_use]
    pub fn resolved_caret_frame(
        &self,
        node: NodeId,
        style: &crate::CaretStyle,
    ) -> crate::CaretFrame {
        style.sample(self.caret.elapsed(node))
    }

    pub fn ime_input(&mut self, input: ImeInput) -> InteractionUpdate {
        let Some(node) = self.interaction.focused() else {
            return InteractionUpdate::default();
        };
        let input = if self.input_available(node) {
            input
        } else {
            ImeInput::Disabled
        };
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.ime(input);
        self.text_input_update(node, result)
    }

    pub fn paste_text(&mut self, target: Option<NodeId>, text: &str) -> InteractionUpdate {
        let Some(node) = target.or_else(|| self.interaction.focused()) else {
            return InteractionUpdate::default();
        };
        if !self.input_available(node) {
            return InteractionUpdate::default();
        }
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.paste(text);
        self.text_input_update(node, result)
    }

    /// Replace a controlled editor's content as a user edit, not an external model reset.
    pub fn replace_text_input(&mut self, node: NodeId, value: &str) -> InteractionUpdate {
        if !self.input_available(node) {
            return InteractionUpdate::default();
        }
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.replace_value(value);
        self.text_input_update(node, result)
    }

    pub fn place_text_cursor(
        &mut self,
        node: NodeId,
        index: usize,
        extend: bool,
    ) -> InteractionUpdate {
        let Some(result) = self.text_inputs.place(node, index, extend) else {
            return InteractionUpdate::default();
        };
        self.text_input_update(node, result)
    }

    pub fn place_text_position(
        &mut self,
        node: NodeId,
        position: TextPosition,
        extend: bool,
    ) -> InteractionUpdate {
        let Some(result) = self.text_inputs.place_position(node, position, extend) else {
            return InteractionUpdate::default();
        };
        self.text_input_update(node, result)
    }

    pub fn drag_text_cursor(&mut self, node: NodeId, index: usize) -> InteractionUpdate {
        let Some(result) = self.text_inputs.drag(node, index) else {
            return InteractionUpdate::default();
        };
        self.text_input_update(node, result)
    }

    pub fn drag_text_position(
        &mut self,
        node: NodeId,
        position: TextPosition,
    ) -> InteractionUpdate {
        let Some(result) = self.text_inputs.drag_position(node, position) else {
            return InteractionUpdate::default();
        };
        self.text_input_update(node, result)
    }

    pub fn move_text_cursor(
        &mut self,
        node: NodeId,
        index: usize,
        extend: bool,
    ) -> InteractionUpdate {
        let Some(result) = self.text_inputs.move_to(node, index, extend) else {
            return InteractionUpdate::default();
        };
        self.text_input_update(node, result)
    }

    pub fn move_text_position(
        &mut self,
        node: NodeId,
        position: TextPosition,
        extend: bool,
    ) -> InteractionUpdate {
        let Some(result) = self.text_inputs.move_to_position(node, position, extend) else {
            return InteractionUpdate::default();
        };
        self.text_input_update(node, result)
    }

    #[must_use]
    pub const fn text_cursor_dragging(&self) -> bool {
        self.text_inputs.dragging()
    }

    pub fn release_text_cursor(&mut self) -> bool {
        self.text_inputs.release()
    }

    /// Begin a caret, whole-word or whole-line selection, retaining its drag unit.
    pub fn begin_text_selection(
        &mut self,
        node: NodeId,
        position: TextPosition,
        extend: bool,
        granularity: SelectionGranularity,
    ) -> InteractionUpdate {
        if !self.input_available(node) || self.text_input_composing(node) {
            return InteractionUpdate::default();
        }
        let Some(result) = self
            .text_inputs
            .begin_selection(node, position, extend, granularity)
        else {
            return InteractionUpdate::default();
        };
        self.text_input_update(node, result)
    }

    /// Desired content-space column retained while navigating through shorter lines.
    #[must_use]
    pub fn text_input_goal_x(&self, node: NodeId) -> Option<f32> {
        self.text_inputs.get(node).and_then(TextInputState::goal_x)
    }

    pub fn move_text_vertically(
        &mut self,
        node: NodeId,
        position: TextPosition,
        goal_x: f32,
        extend: bool,
    ) -> InteractionUpdate {
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.move_vertically(position, goal_x, extend);
        self.text_input_update(node, result)
    }
}
