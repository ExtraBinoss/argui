use crate::{HistoryConfig, InteractionUpdate, NodeId, UiTree};

impl UiTree {
    pub(crate) fn input_available(&self, node: NodeId) -> bool {
        let Some(mut index) = self.index.position(node) else {
            return false;
        };
        loop {
            let Some(element) = self.element_at(index) else {
                return false;
            };
            if self
                .resolved_layout_style(self.node_ids[index], element)
                .display
                == crate::Display::None
                || element
                    .interaction
                    .as_ref()
                    .is_some_and(|interaction| !interaction.enabled)
            {
                return false;
            }
            let Some(parent) = self.index.parent(index) else {
                return true;
            };
            index = parent;
        }
    }
    /// Safe copy for telemetry; owner callbacks still receive the actual edit.
    ///
    /// * `event` — event to copy and redact if its target is protected.
    #[must_use]
    pub fn inspect_event(&self, event: &crate::UiEvent) -> crate::UiEvent {
        let mut event = event.clone();
        if self.text_input_protected(event.target) {
            match &mut event.kind {
                crate::UiEventKind::TextChanged(value) | crate::UiEventKind::Submitted(value) => {
                    *value = "[protected]".into()
                }
                crate::UiEventKind::TextEdited(edit) => {
                    edit.range = 0..0;
                    edit.replacement = "[protected]".into();
                }
                crate::UiEventKind::KeyInput(input) => {
                    input.text = None;
                    if matches!(input.key, argui_core::Key::Character(_)) {
                        input.key = argui_core::Key::Other;
                    }
                }
                crate::UiEventKind::SemanticAction { value, .. } => *value = None,
                _ => {}
            }
        }
        event
    }
    /// Returns whether a text input is in a protected privacy mode.
    ///
    /// * `node` — text-input node to query.
    #[must_use]
    pub fn text_input_protected(&self, node: NodeId) -> bool {
        self.text_inputs
            .get(node)
            .is_some_and(super::super::text_input::TextInputState::protected)
    }
    /// Returns whether undo is currently available for a text input.
    ///
    /// * `node` — text-input node to query.
    #[must_use]
    pub fn can_undo(&self, node: NodeId) -> bool {
        self.input_available(node)
            && self
                .text_inputs
                .get(node)
                .is_some_and(super::super::text_input::TextInputState::can_undo)
    }
    /// Returns whether redo is currently available for a text input.
    ///
    /// * `node` — text-input node to query.
    #[must_use]
    pub fn can_redo(&self, node: NodeId) -> bool {
        self.input_available(node)
            && self
                .text_inputs
                .get(node)
                .is_some_and(super::super::text_input::TextInputState::can_redo)
    }
    /// Returns whether a text input has an active IME composition.
    ///
    /// * `node` — text-input node to query.
    #[must_use]
    pub fn text_input_composing(&self, node: NodeId) -> bool {
        self.text_inputs
            .get(node)
            .is_some_and(super::super::text_input::TextInputState::composing)
    }
    /// Sets history limits for an existing text input.
    ///
    /// * `node` — text-input node to configure.
    /// * `config` — transaction and byte limits to apply.
    pub fn configure_text_history(&mut self, node: NodeId, config: HistoryConfig) {
        if let Some(state) = self.text_inputs.get_mut(node) {
            state.configure_history(config);
        }
    }
    /// Undoes the latest retained edit for a text input.
    ///
    /// * `node` — text-input node to edit.
    ///
    /// Returns the interaction update, which is empty if undo is unavailable.
    pub fn undo_text_input(&mut self, node: NodeId) -> InteractionUpdate {
        self.edit_history(node, false)
    }
    /// Reapplies the next retained edit for a text input.
    ///
    /// * `node` — text-input node to edit.
    ///
    /// Returns the interaction update, which is empty if redo is unavailable.
    pub fn redo_text_input(&mut self, node: NodeId) -> InteractionUpdate {
        self.edit_history(node, true)
    }
    fn edit_history(&mut self, node: NodeId, redo: bool) -> InteractionUpdate {
        if !self.input_available(node) {
            return InteractionUpdate::default();
        }
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.history_step(redo);
        self.text_input_update(node, result)
    }
}
