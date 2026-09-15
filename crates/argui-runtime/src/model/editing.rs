use super::Context;
use argui_ui::{ActionInvocation, FocusTarget, NodeId, SelectionCommand, UiCommand};

impl<T: super::Render> Context<T> {
    /// Queues a low-level UI editing command for the current window.
    ///
    /// `command` is the operation to pass to the UI tree.
    pub fn ui_command(&mut self, command: UiCommand) {
        self.effects.ui_commands.push(command);
    }
    /// Queues invocation of a declared UI action.
    ///
    /// `invocation` identifies the action and the state to invoke it with.
    pub fn invoke_action(&mut self, invocation: ActionInvocation) {
        self.ui_command(UiCommand::Action(invocation));
    }
    /// One undoable replacement. Updating the authored value instead is an external reset.
    ///
    /// `target` identifies the text node; `value` is the replacement text.
    pub fn edit_text(&mut self, target: impl Into<FocusTarget>, value: impl Into<String>) {
        self.ui_command(UiCommand::ReplaceText {
            target: target.into(),
            value: value.into(),
        });
    }
    /// Queues a selection command for whichever text input currently owns selection.
    ///
    /// `command` describes the selection operation.
    pub fn selection_command(&mut self, command: SelectionCommand) {
        self.ui_command(UiCommand::Selection {
            target: None,
            command,
        });
    }
    /// Queues a selection command for a specific text node.
    ///
    /// `target` is the node identifier; `command` describes the selection operation.
    pub fn selection_command_for(&mut self, target: NodeId, command: SelectionCommand) {
        self.ui_command(UiCommand::Selection {
            target: Some(target),
            command,
        });
    }
}
