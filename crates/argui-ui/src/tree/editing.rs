use crate::{HistoryConfig, InteractionUpdate, NodeId, UiTree};

impl UiTree {
    pub(crate) fn input_available(&self, node: NodeId) -> bool {
        let Some(mut index) = self.node_ids.iter().position(|id| *id == node) else {
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
            let Some(parent) = self.events.parent(index) else {
                return true;
            };
            index = parent;
        }
    }
    /// Safe copy for telemetry; owner callbacks still receive the actual edit.
    #[must_use]
    pub fn inspect_event(&self, event: &crate::UiEvent) -> crate::UiEvent {
        let mut event = event.clone();
        if self.text_input_protected(event.target) {
            match &mut event.kind {
                crate::UiEventKind::TextChanged(value) | crate::UiEventKind::Submitted(value) => {
                    *value = "[protected]".into()
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
    #[must_use]
    pub fn text_input_protected(&self, node: NodeId) -> bool {
        self.text_inputs
            .get(node)
            .is_some_and(super::super::text_input::TextInputState::protected)
    }
    #[must_use]
    pub fn can_undo(&self, node: NodeId) -> bool {
        self.input_available(node)
            && self
                .text_inputs
                .get(node)
                .is_some_and(super::super::text_input::TextInputState::can_undo)
    }
    #[must_use]
    pub fn can_redo(&self, node: NodeId) -> bool {
        self.input_available(node)
            && self
                .text_inputs
                .get(node)
                .is_some_and(super::super::text_input::TextInputState::can_redo)
    }
    #[must_use]
    pub fn text_input_composing(&self, node: NodeId) -> bool {
        self.text_inputs
            .get(node)
            .is_some_and(super::super::text_input::TextInputState::composing)
    }
    pub fn configure_text_history(&mut self, node: NodeId, config: HistoryConfig) {
        if let Some(state) = self.text_inputs.get_mut(node) {
            state.configure_history(config);
        }
    }
    pub fn undo_text_input(&mut self, node: NodeId) -> InteractionUpdate {
        self.edit_history(node, false)
    }
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
