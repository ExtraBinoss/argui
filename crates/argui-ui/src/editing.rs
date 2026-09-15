use crate::{ActionInvocation, FocusTarget, InteractionUpdate, NodeId, SelectionCommand, UiTree};

/// Deferred UI commands, processed on the UI thread after the event handler returns.
#[derive(Clone, Debug, PartialEq)]
pub enum UiCommand {
    Action(ActionInvocation),
    ReplaceText {
        target: FocusTarget,
        value: String,
    },
    Selection {
        target: Option<NodeId>,
        command: SelectionCommand,
    },
}

impl UiTree {
    /// Applies a deferred UI command and returns resulting interaction updates.
    ///
    /// * `command` — action, selection command, or text replacement to process.
    pub fn apply_command(&mut self, command: UiCommand) -> InteractionUpdate {
        match command {
            UiCommand::Action(invocation) => self.invoke_action(invocation),
            UiCommand::Selection { target, command } => self.invoke_action(ActionInvocation {
                id: command.into(),
                origin: target.or(self.focused_node()),
            }),
            UiCommand::ReplaceText { target, value } => {
                let node = match target {
                    FocusTarget::Node(node) => Some(node),
                    FocusTarget::Key(key) => {
                        let mut nodes = self.node_ids().iter().copied().enumerate().filter_map(
                            |(index, node)| {
                                self.element_at(index)
                                    .is_some_and(|element| element.key.as_deref() == Some(&key))
                                    .then_some(node)
                            },
                        );
                        let first = nodes.next();
                        if nodes.next().is_some() { None } else { first }
                    }
                };
                node.map_or_else(InteractionUpdate::default, |node| {
                    self.replace_text_input(node, &value)
                })
            }
        }
    }
}
