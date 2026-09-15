use std::ops::Range;

use argui_core::{CaretAffinity, TextPosition};
use argui_paint::Color;

use crate::{Element, InteractionUpdate, NodeId, UiTree};

mod policy;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UserSelect {
    #[default]
    Auto,
    Text,
    None,
    All,
    Contain,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextSelectionStyle {
    pub background: Color,
    pub handle: Color,
}

impl Default for TextSelectionStyle {
    fn default() -> Self {
        Self {
            background: Color::srgba(0.20, 0.48, 0.96, 0.38),
            handle: Color::srgb(0.20, 0.48, 0.96),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentTextPoint {
    pub node: NodeId,
    pub position: TextPosition,
}

impl DocumentTextPoint {
    /// Creates a document position within a retained node.
    ///
    /// * `node` — node containing the text position.
    /// * `position` — byte index and affinity within that node's text.
    #[must_use]
    pub const fn new(node: NodeId, position: TextPosition) -> Self {
        Self { node, position }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentTextSelection {
    pub anchor: DocumentTextPoint,
    pub focus: DocumentTextPoint,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SelectionGranularity {
    #[default]
    Character,
    Word,
    Line,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionCommand {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectionCapabilities {
    pub editable: bool,
    pub cut: bool,
    pub copy: bool,
    pub paste: bool,
    pub select_all: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DocumentSelectionState {
    pub(crate) selection: Option<DocumentTextSelection>,
    drag: Option<DocumentSelectionDrag>,
    pub(crate) touch_handles: bool,
    pub(crate) revision: u64,
    scope: Option<NodeId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DocumentSelectionDrag {
    granularity: SelectionGranularity,
    origin: DocumentTextSelection,
}

impl Element {
    /// Sets how users may select this element's text.
    ///
    /// * `value` — selection policy for this element and its content.
    #[must_use]
    pub fn user_select(mut self, value: UserSelect) -> Self {
        self.user_select = value;
        self
    }

    /// Sets the colors used for document text selection and touch handles.
    ///
    /// * `value` — selection highlight and handle colors.
    #[must_use]
    pub fn selection_style(mut self, value: TextSelectionStyle) -> Self {
        self.selection_style = Some(value);
        self
    }
}

impl UiTree {
    /// Returns the current document-wide text selection, if any.
    #[must_use]
    pub const fn document_selection(&self) -> Option<DocumentTextSelection> {
        self.document_selection.selection
    }

    /// Returns the revision number of the current document selection state.
    #[must_use]
    pub const fn document_selection_revision(&self) -> u64 {
        self.document_selection.revision
    }

    /// Returns whether a document selection drag is in progress.
    #[must_use]
    pub const fn document_selection_dragging(&self) -> bool {
        self.document_selection.drag.is_some()
    }

    /// Returns whether touch selection handles are currently shown.
    #[must_use]
    pub const fn document_selection_handles_visible(&self) -> bool {
        self.document_selection.touch_handles
    }

    /// Returns whether the document selection contains a non-empty range.
    #[must_use]
    pub fn has_document_selection(&self) -> bool {
        match self.document_selection.selection {
            Some(selection) => selection.anchor != selection.focus,
            None => false,
        }
    }

    /// Begins or extends a document selection at `point`.
    ///
    /// * `point` — text position where selection starts.
    /// * `extend` — whether to preserve the existing anchor.
    /// * `granularity` — character, word, or line selection unit.
    ///
    /// Returns the interaction changes caused by starting the selection.
    pub fn begin_document_selection(
        &mut self,
        point: DocumentTextPoint,
        extend: bool,
        granularity: SelectionGranularity,
    ) -> InteractionUpdate {
        self.document_selection.touch_handles = false;
        self.begin_selection(point, extend, granularity)
    }

    /// Begins a touch selection and shows touch handles at `point`.
    ///
    /// * `point` — text position where selection starts.
    /// * `granularity` — character, word, or line selection unit.
    ///
    /// Returns the interaction changes caused by starting the selection.
    pub fn begin_touch_document_selection(
        &mut self,
        point: DocumentTextPoint,
        granularity: SelectionGranularity,
    ) -> InteractionUpdate {
        self.document_selection.touch_handles = true;
        self.begin_selection(point, false, granularity)
    }

    fn begin_selection(
        &mut self,
        point: DocumentTextPoint,
        extend: bool,
        granularity: SelectionGranularity,
    ) -> InteractionUpdate {
        let Some((point, opposite, scope)) = self.expanded_point(point, granularity) else {
            return self.clear_document_selection();
        };
        let anchor = if extend {
            self.document_selection
                .selection
                .map_or(point, |selection| selection.anchor)
        } else {
            point
        };
        self.document_selection.scope = if extend {
            self.document_selection.scope
        } else {
            scope
        };
        let selection = DocumentTextSelection {
            anchor,
            focus: opposite,
        };
        self.set_document_selection(
            selection,
            Some(DocumentSelectionDrag {
                granularity,
                origin: selection,
            }),
        )
    }

    /// Extends the active document selection to `point`.
    ///
    /// Returns the interaction changes caused by moving the selection.
    pub fn drag_document_selection(&mut self, point: DocumentTextPoint) -> InteractionUpdate {
        let Some(drag) = self.document_selection.drag else {
            return InteractionUpdate::default();
        };
        let point = self.clamp_to_selection_scope(point);
        let selection = if drag.granularity == SelectionGranularity::Character {
            DocumentTextSelection {
                anchor: drag.origin.anchor,
                focus: point,
            }
        } else {
            let Some((start, end, _)) = self.expanded_point(point, drag.granularity) else {
                return InteractionUpdate::default();
            };
            let anchor_before_focus =
                self.document_order(drag.origin.anchor) <= self.document_order(drag.origin.focus);
            let (origin_start, origin_end) = if anchor_before_focus {
                (drag.origin.anchor, drag.origin.focus)
            } else {
                (drag.origin.focus, drag.origin.anchor)
            };
            if self.document_order(start) < self.document_order(origin_start) {
                DocumentTextSelection {
                    anchor: origin_end,
                    focus: start,
                }
            } else {
                DocumentTextSelection {
                    anchor: origin_start,
                    focus: if self.document_order(end) > self.document_order(origin_end) {
                        end
                    } else {
                        origin_end
                    },
                }
            }
        };
        self.set_document_selection(selection, Some(drag))
    }

    /// Ends an active document selection drag.
    ///
    /// Returns the interaction changes caused by releasing the selection.
    pub fn release_document_selection(&mut self) -> InteractionUpdate {
        if self.document_selection.drag.take().is_none() {
            return InteractionUpdate::default();
        }
        self.document_selection_update(false)
    }

    /// Clears the document selection and any active drag or touch handles.
    ///
    /// Returns the interaction changes caused by clearing the selection.
    pub fn clear_document_selection(&mut self) -> InteractionUpdate {
        if self.document_selection.selection.take().is_none() {
            return InteractionUpdate::default();
        }
        self.document_selection.drag = None;
        self.document_selection.touch_handles = false;
        self.document_selection.revision = self.document_selection.revision.wrapping_add(1);
        self.document_selection.scope = None;
        self.document_selection_update(true)
    }

    /// Selects all selectable document text in the current containment scope.
    ///
    /// Returns the interaction changes caused by selecting the text.
    pub fn select_all_document_text(&mut self) -> InteractionUpdate {
        self.document_selection.touch_handles = false;
        let mut entries = self.selectable_text_entries();
        if let Some(scope) = self.document_selection.scope {
            entries.retain(|entry| entry.contain_root == Some(scope));
        }
        let (Some(first), Some(last)) = (entries.first(), entries.last()) else {
            return self.clear_document_selection();
        };
        self.set_document_selection(
            DocumentTextSelection {
                anchor: DocumentTextPoint::new(
                    first.node,
                    TextPosition::new(0, CaretAffinity::Before),
                ),
                focus: DocumentTextPoint::new(
                    last.node,
                    TextPosition::new(last.text.len(), CaretAffinity::After),
                ),
            },
            None,
        )
    }

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

    /// Applies a selection command to a specific input or the document selection.
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
    pub fn document_selection_contains(&self, point: DocumentTextPoint) -> bool {
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

    fn set_document_selection(
        &mut self,
        selection: DocumentTextSelection,
        drag: Option<DocumentSelectionDrag>,
    ) -> InteractionUpdate {
        if self.document_selection.selection == Some(selection)
            && self.document_selection.drag == drag
        {
            return InteractionUpdate::default();
        }
        self.document_selection.selection = Some(selection);
        self.document_selection.drag = drag;
        self.document_selection.revision = self.document_selection.revision.wrapping_add(1);
        self.document_selection_update(true)
    }

    fn document_selection_update(&mut self, paint_changed: bool) -> InteractionUpdate {
        InteractionUpdate {
            events: self
                .node_ids()
                .first()
                .copied()
                .map_or_else(Vec::new, |target| {
                    self.event_deliveries(
                        target,
                        crate::UiEventKind::DocumentSelectionChanged {
                            text: self.selected_document_text(),
                            bounds: None,
                            touch: self.document_selection.touch_handles,
                            dragging: self.document_selection.drag.is_some(),
                        },
                    )
                }),
            paint_changed,
            ..InteractionUpdate::default()
        }
    }

    fn document_order(&self, point: DocumentTextPoint) -> Option<(usize, usize)> {
        Some((self.node_index(point.node)?, point.position.index))
    }

    fn node_index(&self, node: NodeId) -> Option<usize> {
        self.node_ids()
            .iter()
            .position(|candidate| *candidate == node)
    }
}
