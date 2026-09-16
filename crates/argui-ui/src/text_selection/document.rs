use std::ops::Range;

use super::{SelectionCapabilities, SelectionCommand};
use crate::{InteractionUpdate, NodeId, UiTree};

impl UiTree {
    /// Reports supported text-selection commands for a node or document.
    ///
    /// * `target` — node whose text-input capabilities are queried.
    #[must_use]
    pub fn selection_capabilities(&self, target: NodeId) -> SelectionCapabilities {
        if let Some(input) = self.text_inputs.get(target) {
            let selected = input.has_selection() && !input.protected();
            return SelectionCapabilities {
                editable: true,
                cut: selected && !input.read_only(),
                copy: selected,
                paste: !input.read_only(),
                select_all: !input.value().is_empty(),
            };
        }
        self.document_selection_capabilities()
    }

    /// Reports capabilities for the current document-wide selection.
    #[must_use]
    pub fn document_selection_capabilities(&self) -> SelectionCapabilities {
        SelectionCapabilities {
            copy: self.has_document_selection(),
            select_all: !self.selectable_text_entries().is_empty(),
            ..SelectionCapabilities::default()
        }
    }

    /// Applies a selection command to the currently focused target.
    ///
    /// * `command` — copy, paste, cut, undo, redo, or select-all operation.
    ///
    /// Returns the interaction changes caused by the command.
    pub fn focused_selection_command(&mut self, command: SelectionCommand) -> InteractionUpdate {
        let target = self.interaction.focused();
        self.selection_command(target, command)
    }

    /// Applies a selection command to a specific input or document selection.
    ///
    /// * `target` — optional text-input node; `None` targets document text.
    /// * `command` — copy, paste, cut, undo, redo, or select-all operation.
    ///
    /// Returns the interaction changes caused by the command.
    pub fn selection_command(
        &mut self,
        target: Option<NodeId>,
        command: SelectionCommand,
    ) -> InteractionUpdate {
        if target
            .is_some_and(|node| self.text_inputs.get(node).is_some() && !self.input_available(node))
        {
            return InteractionUpdate::default();
        }
        if let Some(node) = target
            && let Some(state) = self.text_inputs.get_mut(node)
        {
            let mut result = state.selection_command(command);
            if matches!(result.clipboard, Some(crate::ClipboardRequest::Read { .. })) {
                result.clipboard = Some(crate::ClipboardRequest::Read { target: Some(node) });
            }
            return self.text_input_update(node, result);
        }
        match command {
            SelectionCommand::Copy => {
                self.selected_document_text()
                    .map_or_else(InteractionUpdate::default, |text| InteractionUpdate {
                        clipboard: Some(crate::ClipboardRequest::Write(text)),
                        ..InteractionUpdate::default()
                    })
            }
            SelectionCommand::SelectAll => self.select_all_document_text(),
            SelectionCommand::Cut
            | SelectionCommand::Paste
            | SelectionCommand::Undo
            | SelectionCommand::Redo => InteractionUpdate::default(),
        }
    }

    /// Returns the selected document text, joined by newlines, when non-empty.
    #[must_use]
    pub fn selected_document_text(&self) -> Option<String> {
        self.document_selection.selection?;
        let mut selected = Vec::new();
        for entry in self.selectable_text_entries() {
            let Some(range) = self.document_selection_range(entry.node, entry.text.len()) else {
                continue;
            };
            if range.start < range.end {
                selected.push(entry.text[range].to_owned());
            }
        }
        (!selected.is_empty()).then(|| selected.join("\n"))
    }

    /// Returns the selected byte range within `node`, bounded by `text_len`.
    ///
    /// * `node` — node whose text range is requested.
    /// * `text_len` — byte length of that node's text.
    #[must_use]
    pub fn document_selection_range(&self, node: NodeId, text_len: usize) -> Option<Range<usize>> {
        let selection = self.document_selection.selection?;
        let anchor = self.document_order(selection.anchor)?;
        let focus = self.document_order(selection.focus)?;
        let (start, end) = if anchor <= focus {
            (selection.anchor, selection.focus)
        } else {
            (selection.focus, selection.anchor)
        };
        let index = self.node_index(node)?;
        let start_index = self.node_index(start.node)?;
        let end_index = self.node_index(end.node)?;
        if index < start_index || index > end_index {
            return None;
        }
        let range = if start.node == end.node {
            start.position.index.min(text_len)..end.position.index.min(text_len)
        } else if node == start.node {
            start.position.index.min(text_len)..text_len
        } else if node == end.node {
            0..end.position.index.min(text_len)
        } else {
            0..text_len
        };
        Some(range)
    }

    /// Returns whether `point` lies within the document selection, inclusively.
    ///
    /// * `point` — document position to test.
    #[must_use]
    pub fn document_selection_contains(&self, point: super::DocumentTextPoint) -> bool {
        let Some(selection) = self.document_selection.selection else {
            return false;
        };
        let Some(anchor) = self.document_order(selection.anchor) else {
            return false;
        };
        let Some(focus) = self.document_order(selection.focus) else {
            return false;
        };
        let Some(candidate) = self.document_order(point) else {
            return false;
        };
        let (start, end) = if anchor <= focus {
            (anchor, focus)
        } else {
            (focus, anchor)
        };
        start <= candidate && candidate <= end
    }

    /// Returns whether the selection intersects a node-index interval.
    ///
    /// * `first` — first node index in the interval.
    /// * `count` — number of node indices in the interval.
    #[must_use]
    pub fn document_selection_intersects(&self, first: usize, count: usize) -> bool {
        let Some(selection) = self.document_selection.selection else {
            return false;
        };
        let Some(anchor) = self.node_index(selection.anchor.node) else {
            return false;
        };
        let Some(focus) = self.node_index(selection.focus.node) else {
            return false;
        };
        let start = anchor.min(focus);
        let end = anchor.max(focus);
        start < first.saturating_add(count) && end >= first
    }
}
