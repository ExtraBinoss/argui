use crate::text_input::TextInputState;
use crate::{InteractionUpdate, NodeId, SelectionGranularity, TextSelectionRequest, UiTree};
use argui_core::{ImeInput, TextPosition};

impl UiTree {
    /// Returns the authored text value for a text input, if `node` is one.
    ///
    /// * `node` — retained text-input node.
    #[must_use]
    pub fn text_input_value(&self, node: NodeId) -> Option<&str> {
        self.text_inputs.get(node).map(TextInputState::value)
    }

    /// Returns the text as currently displayed, including privacy masking.
    ///
    /// * `node` — retained text-input node.
    #[must_use]
    pub fn text_input_display(&self, node: NodeId) -> Option<String> {
        self.text_inputs.get(node).map(TextInputState::display)
    }

    /// Returns the displayed cursor byte index for a text input.
    ///
    /// * `node` — retained text-input node.
    #[must_use]
    pub fn text_input_cursor(&self, node: NodeId) -> Option<usize> {
        self.text_inputs
            .get(node)
            .map(TextInputState::display_cursor)
    }

    /// Returns the displayed cursor position and affinity for a text input.
    ///
    /// * `node` — retained text-input node.
    #[must_use]
    pub fn text_input_position(&self, node: NodeId) -> Option<TextPosition> {
        self.text_inputs
            .get(node)
            .map(TextInputState::display_position)
    }

    /// Returns the selected byte range for a text input, if nonempty.
    ///
    /// * `node` — retained text-input node.
    #[must_use]
    pub fn text_input_selection(&self, node: NodeId) -> Option<(usize, usize)> {
        self.text_inputs
            .get(node)
            .and_then(TextInputState::selection)
    }

    /// Returns selected displayed positions, including caret affinities.
    ///
    /// * `node` — retained text-input node.
    #[must_use]
    pub fn text_input_selection_positions(
        &self,
        node: NodeId,
    ) -> Option<(TextPosition, TextPosition)> {
        self.text_inputs
            .get(node)
            .and_then(TextInputState::selection_positions)
    }

    /// Returns whether the cursor should be brought into view after editing.
    ///
    /// * `node` — retained text-input node.
    #[must_use]
    pub fn text_input_should_reveal_cursor(&self, node: NodeId) -> bool {
        self.text_inputs.should_reveal_cursor(node)
    }

    /// Applies a text selection request to its resolved input target.
    ///
    /// * `request` — target node or key and requested selection.
    ///
    /// Returns the resulting interaction update, or an empty update if the target
    /// is not a text input in the current tree.
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
    /// Clears pending cursor-reveal requests after layout has handled them.
    pub fn mark_text_input_layout_clean(&mut self) {
        self.text_inputs.clear_cursor_reveals();
    }

    /// Samples the caret frame for a text input using its current animation time.
    ///
    /// * `node` — retained text-input node.
    /// * `style` — caret animation style to sample.
    #[must_use]
    pub fn resolved_caret_frame(
        &self,
        node: NodeId,
        style: &crate::CaretStyle,
    ) -> crate::CaretFrame {
        style.sample(self.caret.elapsed(node))
    }

    /// Applies an input-method event to the focused text input.
    ///
    /// * `input` — preedit, commit, enable, or disable event.
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

    /// Pastes text into the requested or focused text input.
    ///
    /// * `target` — optional explicit input node; otherwise the focused node is used.
    /// * `text` — text to insert.
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
    ///
    /// * `node` — text-input node to edit.
    /// * `value` — replacement text.
    ///
    /// Returns the resulting interaction update.
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

    /// Places the cursor at a byte index, optionally extending the selection.
    ///
    /// * `node` — text-input node to edit.
    /// * `index` — requested byte index in the displayed text.
    /// * `extend` — whether to preserve the selection anchor.
    ///
    /// Returns the resulting interaction update.
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

    /// Places the cursor at a displayed text position and affinity.
    ///
    /// * `node` — text-input node to edit.
    /// * `position` — requested position in the displayed text.
    /// * `extend` — whether to preserve the selection anchor.
    ///
    /// Returns the resulting interaction update.
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

    /// Extends an active cursor drag to a displayed byte index.
    ///
    /// * `node` — text-input node being dragged.
    /// * `index` — requested byte index in the displayed text.
    ///
    /// Returns the resulting interaction update.
    pub fn drag_text_cursor(&mut self, node: NodeId, index: usize) -> InteractionUpdate {
        let Some(result) = self.text_inputs.drag(node, index) else {
            return InteractionUpdate::default();
        };
        self.text_input_update(node, result)
    }

    /// Extends an active cursor drag to a displayed text position.
    ///
    /// * `node` — text-input node being dragged.
    /// * `position` — requested displayed position and affinity.
    ///
    /// Returns the resulting interaction update.
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

    /// Moves the cursor to a byte index, optionally extending selection.
    ///
    /// * `node` — text-input node to edit.
    /// * `index` — requested byte index in the displayed text.
    /// * `extend` — whether to preserve the selection anchor.
    ///
    /// Returns the resulting interaction update.
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

    /// Moves the cursor to a displayed position, optionally extending selection.
    ///
    /// * `node` — text-input node to edit.
    /// * `position` — requested displayed position and affinity.
    /// * `extend` — whether to preserve the selection anchor.
    ///
    /// Returns the resulting interaction update.
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

    /// Returns whether a text cursor drag is active.
    #[must_use]
    pub const fn text_cursor_dragging(&self) -> bool {
        self.text_inputs.dragging()
    }

    /// Ends the active text cursor drag.
    ///
    /// Returns whether a drag was active.
    pub fn release_text_cursor(&mut self) -> bool {
        self.text_inputs.release()
    }

    /// Begin a caret, whole-word or whole-line selection, retaining its drag unit.
    ///
    /// * `node` — text-input node to select within.
    /// * `position` — selection start position.
    /// * `extend` — whether to preserve the existing selection anchor.
    /// * `granularity` — character, word, or line selection unit.
    ///
    /// Returns the resulting interaction update.
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
    ///
    /// * `node` — text-input node to query.
    ///
    /// Returns the retained horizontal goal, if one has been established.
    #[must_use]
    pub fn text_input_goal_x(&self, node: NodeId) -> Option<f32> {
        self.text_inputs.get(node).and_then(TextInputState::goal_x)
    }

    /// Moves a text cursor vertically while retaining a horizontal goal column.
    ///
    /// * `node` — text-input node to edit.
    /// * `position` — destination position supplied by text layout.
    /// * `goal_x` — desired horizontal column in content coordinates.
    /// * `extend` — whether to preserve the selection anchor.
    ///
    /// Returns the resulting interaction update.
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
