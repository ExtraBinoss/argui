use super::Context;
use argui_ui::{ActionInvocation, FocusTarget, NodeId, SelectionCommand, UiCommand};

impl<T: super::Render> Context<T> {
    pub fn ui_command(&mut self, command: UiCommand) {
        self.effects.ui_commands.push(command);
    }
    pub fn invoke_action(&mut self, invocation: ActionInvocation) {
        self.ui_command(UiCommand::Action(invocation));
    }
    /// One undoable replacement. Updating the authored value instead is an external reset.
    pub fn edit_text(&mut self, target: impl Into<FocusTarget>, value: impl Into<String>) {
        self.ui_command(UiCommand::ReplaceText {
            target: target.into(),
            value: value.into(),
        });
    }
    pub fn selection_command(&mut self, command: SelectionCommand) {
        self.ui_command(UiCommand::Selection {
            target: None,
            command,
        });
    }
    pub fn selection_command_for(&mut self, target: NodeId, command: SelectionCommand) {
        self.ui_command(UiCommand::Selection {
            target: Some(target),
            command,
        });
    }
}
