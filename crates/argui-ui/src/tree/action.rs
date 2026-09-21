use crate::{
    ActionBinding, ActionError, ActionId, ActionInvocation, ActionState, EventPhase,
    InteractionUpdate, NodeId, Shortcut, UiEvent, UiEventKind, UiTree,
};

impl UiTree {
    fn action_ancestry(&self, origin: Option<NodeId>) -> Result<Vec<usize>, ActionError> {
        let node = origin.or(self.focused_node()).unwrap_or(self.node_ids[0]);
        let Some(mut index) = self.index.position(node) else {
            return Err(ActionError::StaleOrigin);
        };
        let modal = self.active_modal_scope();
        if modal.is_some_and(|scope| !self.focus.contains(scope, node)) {
            return Err(ActionError::Unavailable);
        }
        let mut ancestry = Vec::new();
        loop {
            if self.element_at(index).is_some_and(|element| {
                self.resolved_layout_style(self.node_ids[index], element)
                    .display
                    == crate::Display::None
                    || element
                        .interaction
                        .as_ref()
                        .is_some_and(|interaction| !interaction.enabled)
            }) {
                return Err(ActionError::Unavailable);
            }
            ancestry.push(index);
            if Some(self.node_ids[index]) == modal {
                break;
            }
            let Some(parent) = self.index.parent(index) else {
                break;
            };
            index = parent;
        }
        Ok(ancestry)
    }
    fn action_binding(
        &self,
        invocation: &ActionInvocation,
    ) -> Result<Option<(usize, &ActionBinding)>, ActionError> {
        for index in self.action_ancestry(invocation.origin)? {
            if let Some(binding) = self
                .element_at(index)
                .and_then(|element| element.action_scope.as_ref())
                .and_then(|scope| {
                    scope
                        .bindings
                        .iter()
                        .find(|binding| binding.id == invocation.id)
                })
            {
                return Ok(Some((index, binding)));
            }
        }
        Ok(None)
    }
    /// Returns the current label and availability for an action invocation.
    /// Query again when presenting a menu rather than retaining stale state.
    ///
    /// * `invocation` — action identifier and optional origin node.
    ///
    /// # Errors
    ///
    /// Returns [`ActionError::StaleOrigin`] when the origin is no longer in the tree,
    /// or [`ActionError::Unavailable`] when the action cannot be resolved in scope.
    pub fn action_state(&self, invocation: ActionInvocation) -> Result<ActionState, ActionError> {
        if let Some((_, binding)) = self.action_binding(&invocation)? {
            return Ok(binding.state.clone());
        }
        let node = invocation.origin.or(self.focused_node());
        let caps = node.map_or_else(
            || self.document_selection_capabilities(),
            |node| self.selection_capabilities(node),
        );
        let cancel_preedit = node
            .and_then(|node| self.text_inputs.get(node))
            .is_some_and(|state| state.composing() && !state.read_only() && !state.protected());
        let command = invocation
            .id
            .selection_command()
            .ok_or(ActionError::Unavailable)?;
        let enabled = match command {
            crate::SelectionCommand::Undo => {
                cancel_preedit || node.is_some_and(|node| self.can_undo(node))
            }
            crate::SelectionCommand::Redo => {
                cancel_preedit || node.is_some_and(|node| self.can_redo(node))
            }
            crate::SelectionCommand::Copy => caps.copy,
            crate::SelectionCommand::Cut => caps.cut,
            crate::SelectionCommand::Paste => caps.paste,
            crate::SelectionCommand::SelectAll => caps.select_all,
        };
        Ok(ActionState::new(match command {
            crate::SelectionCommand::Undo => "Undo",
            crate::SelectionCommand::Redo => "Redo",
            crate::SelectionCommand::Copy => "Copy",
            crate::SelectionCommand::Cut => "Cut",
            crate::SelectionCommand::Paste => "Paste",
            crate::SelectionCommand::SelectAll => "Select all",
        })
        .enabled(enabled))
    }
    /// Invokes an enabled action and returns its event or selection-command update.
    ///
    /// * `invocation` — action identifier and optional origin node.
    pub fn invoke_action(&mut self, invocation: ActionInvocation) -> InteractionUpdate {
        if !self
            .action_state(invocation.clone())
            .is_ok_and(|state| state.enabled)
        {
            return InteractionUpdate::default();
        }
        if let Ok(Some((index, binding))) = self.action_binding(&invocation) {
            let node = self.node_ids[index];
            let mut base = UiEvent::new(
                node,
                self.key_for(node).map(str::to_owned),
                UiEventKind::Action(invocation.clone()),
            );
            base.set_focused_node(invocation.origin.or(self.focused_node()));
            return InteractionUpdate {
                events: vec![base.delivery(crate::event::EventDelivery {
                    current_target: node,
                    current_key: self.key_for(node).map(str::to_owned),
                    current_handler: Some(binding.listener.handler),
                    phase: EventPhase::Target,
                    passive: false,
                    once: None,
                    handler_value: binding.listener.value.as_ref().and_then(|source| {
                        source.resolve(
                            &base.kind,
                            self.interaction_bounds.get(&base.target).copied(),
                        )
                    }),
                })],
                ..Default::default()
            };
        }
        invocation
            .id
            .selection_command()
            .map_or_else(InteractionUpdate::default, |command| {
                self.selection_command(invocation.origin.or(self.focused_node()), command)
            })
    }
    pub(super) fn default_action(&self, base: &UiEvent) -> Option<UiEvent> {
        let invocation = match &base.kind {
            UiEventKind::Click(_) => {
                let mut index = self.index.position(base.target)?;
                loop {
                    if let Some(invocation) = self.element_at(index)?.action.clone() {
                        break ActionInvocation {
                            origin: invocation.origin.or(Some(self.node_ids[index])),
                            ..invocation
                        };
                    }
                    index = self.index.parent(index)?;
                }
            }
            UiEventKind::KeyInput(input) => {
                if self
                    .focused_node()
                    .is_some_and(|node| self.text_input_composing(node))
                {
                    return None;
                }
                let ancestry = self.action_ancestry(None).ok()?;
                let mut found = None;
                for index in ancestry {
                    if let Some(binding) =
                        self.element_at(index)?
                            .action_scope
                            .as_ref()
                            .and_then(|scope| {
                                scope.bindings.iter().find(|binding| {
                                    binding
                                        .state
                                        .shortcut
                                        .as_ref()
                                        .is_some_and(|shortcut| shortcut.matches(input))
                                })
                            })
                    {
                        found = Some(binding.id.clone());
                        break;
                    }
                }
                let id = found.or_else(|| {
                    [
                        (ActionId::COPY, Shortcut::primary("c")),
                        (ActionId::CUT, Shortcut::primary("x")),
                        (ActionId::PASTE, Shortcut::primary("v")),
                        (ActionId::SELECT_ALL, Shortcut::primary("a")),
                        (ActionId::UNDO, Shortcut::primary("z")),
                        (ActionId::REDO, Shortcut::primary("z").shift()),
                    ]
                    .into_iter()
                    .find(|(_, shortcut)| shortcut.matches(input))
                    .map(|(id, _)| id)
                    .or_else(|| {
                        (!cfg!(target_os = "macos") && Shortcut::primary("y").matches(input))
                            .then_some(ActionId::REDO)
                    })
                })?;
                ActionInvocation {
                    id,
                    origin: self.focused_node(),
                }
            }
            _ => return None,
        };
        let mut event = base.clone();
        event.kind = UiEventKind::Action(invocation);
        Some(event.into_default_action())
    }
}
