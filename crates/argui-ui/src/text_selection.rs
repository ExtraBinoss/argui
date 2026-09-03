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
    pub(crate) dragging: bool,
    pub(crate) touch_handles: bool,
    pub(crate) revision: u64,
    scope: Option<NodeId>,
}

impl Element {
    #[must_use]
    pub fn user_select(mut self, value: UserSelect) -> Self {
        self.user_select = value;
        self
    }

    #[must_use]
    pub fn selection_style(mut self, value: TextSelectionStyle) -> Self {
        self.selection_style = Some(value);
        self
    }
}

impl UiTree {
    #[must_use]
    pub const fn document_selection(&self) -> Option<DocumentTextSelection> {
        self.document_selection.selection
    }

    #[must_use]
    pub const fn document_selection_revision(&self) -> u64 {
        self.document_selection.revision
    }

    #[must_use]
    pub const fn document_selection_dragging(&self) -> bool {
        self.document_selection.dragging
    }

    #[must_use]
    pub const fn document_selection_handles_visible(&self) -> bool {
        self.document_selection.touch_handles
    }

    #[must_use]
    pub fn has_document_selection(&self) -> bool {
        match self.document_selection.selection {
            Some(selection) => selection.anchor != selection.focus,
            None => false,
        }
    }

    pub fn begin_document_selection(
        &mut self,
        point: DocumentTextPoint,
        extend: bool,
        granularity: SelectionGranularity,
    ) -> InteractionUpdate {
        self.document_selection.touch_handles = false;
        self.begin_selection(point, extend, granularity)
    }

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
        self.set_document_selection(
            DocumentTextSelection {
                anchor,
                focus: opposite,
            },
            true,
        )
    }

    pub fn drag_document_selection(&mut self, point: DocumentTextPoint) -> InteractionUpdate {
        let Some(selection) = self.document_selection.selection else {
            return InteractionUpdate::default();
        };
        let point = self.clamp_to_selection_scope(point);
        self.set_document_selection(
            DocumentTextSelection {
                focus: point,
                ..selection
            },
            true,
        )
    }

    pub fn release_document_selection(&mut self) -> InteractionUpdate {
        if !self.document_selection.dragging {
            return InteractionUpdate::default();
        }
        self.document_selection.dragging = false;
        self.document_selection_update(false)
    }

    pub fn clear_document_selection(&mut self) -> InteractionUpdate {
        if self.document_selection.selection.take().is_none() {
            return InteractionUpdate::default();
        }
        self.document_selection.dragging = false;
        self.document_selection.touch_handles = false;
        self.document_selection.revision = self.document_selection.revision.wrapping_add(1);
        self.document_selection.scope = None;
        self.document_selection_update(true)
    }

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
            false,
        )
    }

    #[must_use]
    pub fn selection_capabilities(&self, target: NodeId) -> SelectionCapabilities {
        if let Some(input) = self.text_inputs.get(target) {
            let selected = input.has_selection();
            return SelectionCapabilities {
                editable: true,
                cut: selected && !input.read_only(),
                copy: selected,
                paste: !input.read_only(),
                select_all: !input.value().is_empty(),
            };
        }
        SelectionCapabilities {
            copy: self.has_document_selection(),
            select_all: !self.selectable_text_entries().is_empty(),
            ..SelectionCapabilities::default()
        }
    }

    pub fn focused_selection_command(&mut self, command: SelectionCommand) -> InteractionUpdate {
        let target = self.interaction.focused();
        self.selection_command(target, command)
    }

    pub fn selection_command(
        &mut self,
        target: Option<NodeId>,
        command: SelectionCommand,
    ) -> InteractionUpdate {
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
            SelectionCommand::Cut | SelectionCommand::Paste => InteractionUpdate::default(),
        }
    }

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
        dragging: bool,
    ) -> InteractionUpdate {
        if self.document_selection.selection == Some(selection)
            && self.document_selection.dragging == dragging
        {
            return InteractionUpdate::default();
        }
        self.document_selection.selection = Some(selection);
        self.document_selection.dragging = dragging;
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
                            dragging: self.document_selection.dragging,
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
