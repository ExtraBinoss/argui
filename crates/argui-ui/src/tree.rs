use argui_core::{ImeInput, TextPosition};

use crate::interaction::{InteractionState, RawUpdate};
use crate::scroll::ScrollState;
use crate::text_input::{TextInputState, TextInputStates};
use crate::traversal::{flattened, nth_element};
use crate::update::{classify_update, strongest_update};
use crate::{
    Element, ElementKind, GestureArena, InteractionUpdate, NodeId, TextSelectionRequest, UiEvent,
    UiEventKind, identity,
};

mod animation;
mod event;
mod focus;
mod pointer;
mod resolve;
mod responsive;
mod scroll;
mod transition;
use animation::AnimationRegistry;
use event::EventRegistry;
use focus::FocusRegistry;
use transition::TransitionRegistry;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeUpdate {
    #[default]
    None,
    Semantics,
    Paint,
    Scroll,
    Layout,
}

/// Deterministic work counters for the latest retained-tree update.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeUpdateStats {
    pub visited: usize,
    pub shared_subtrees: usize,
}

#[derive(Clone, Debug)]
pub struct UiTree {
    root: Element,
    node_ids: Vec<NodeId>,
    next_node_id: u64,
    pub(crate) interaction: InteractionState,
    focus: FocusRegistry,
    gestures: GestureArena,
    scroll: ScrollState,
    pub(crate) text_inputs: TextInputStates,
    pub(crate) document_selection: crate::text_selection::DocumentSelectionState,
    caret: crate::caret::CaretAnimator,
    revision: u64,
    layout_dirty: bool,
    update_stats: TreeUpdateStats,
    animations: AnimationRegistry,
    transitions: TransitionRegistry,
    container_sizes: Vec<(NodeId, argui_core::Size)>,
    container_indices: Vec<(NodeId, usize)>,
    has_container_queries: bool,
    reduced_motion: bool,
    events: EventRegistry,
    pending_gestures: Vec<crate::GestureEvent>,
}

impl UiTree {
    #[must_use]
    pub fn new(root: Element) -> Self {
        let mut next_node_id = 1;
        let node_ids = identity::initial_ids(&root, &mut next_node_id);
        let animations = AnimationRegistry::new(&root);
        let focus = FocusRegistry::new(&root, &node_ids);
        let events = EventRegistry::new(&root);
        let mut tree = Self {
            root,
            node_ids,
            next_node_id,
            interaction: InteractionState::default(),
            focus,
            gestures: GestureArena::default(),
            scroll: ScrollState::default(),
            text_inputs: TextInputStates::default(),
            document_selection: crate::text_selection::DocumentSelectionState::default(),
            caret: crate::caret::CaretAnimator::default(),
            revision: 0,
            layout_dirty: true,
            update_stats: TreeUpdateStats::default(),
            animations,
            transitions: TransitionRegistry::default(),
            container_sizes: Vec::new(),
            container_indices: Vec::new(),
            has_container_queries: false,
            reduced_motion: false,
            events,
            pending_gestures: Vec::new(),
        };
        tree.sync_text_inputs();
        tree.sync_responsive_registry();
        tree.sync_transitions();
        tree
    }
    #[must_use]
    pub const fn root(&self) -> &Element {
        &self.root
    }

    #[must_use]
    pub fn node_ids(&self) -> &[NodeId] {
        &self.node_ids
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub const fn layout_dirty(&self) -> bool {
        self.layout_dirty
    }

    #[must_use]
    pub const fn update_stats(&self) -> TreeUpdateStats {
        self.update_stats
    }

    pub fn replace(&mut self, root: Element) -> bool {
        self.update(root) != TreeUpdate::None
    }

    pub fn update(&mut self, root: Element) -> TreeUpdate {
        let mut stats = TreeUpdateStats::default();
        let update = classify_update(&self.root, &root, &mut stats);
        self.update_stats = stats;
        let focused_before = self.interaction.focused();
        let focus_visible_before = focused_before.is_some_and(|node| {
            self.interaction
                .visual_states(node)
                .contains(crate::VisualState::FocusVisible)
        });
        let focused_key = focused_before.and_then(|node| self.key_for(node).map(ToOwned::to_owned));
        match update {
            TreeUpdate::None => return update,
            TreeUpdate::Semantics => {
                self.root = root;
                self.sync_animation_registry();
                self.focus.sync(
                    &self.root,
                    &self.node_ids,
                    focused_before,
                    focus_visible_before,
                    None,
                );
            }
            TreeUpdate::Paint | TreeUpdate::Scroll => {
                self.root = root;
                self.sync_animation_registry();
            }
            TreeUpdate::Layout => {
                let node_ids = identity::reconcile_ids(
                    &self.root,
                    &self.node_ids,
                    &root,
                    &mut self.next_node_id,
                );
                self.node_ids = node_ids;
                self.root = root;
                self.sync_animation_registry();
                self.interaction.retain(&self.node_ids);
                self.pending_gestures
                    .retain(|gesture| self.node_ids.contains(&gesture.target));
                self.scroll.retain(&self.node_ids);
                self.sync_text_inputs();
                if self.document_selection().is_some_and(|selection| {
                    !self.node_ids.contains(&selection.anchor.node)
                        || !self.node_ids.contains(&selection.focus.node)
                }) {
                    self.document_selection =
                        crate::text_selection::DocumentSelectionState::default();
                }
                self.revision = self.revision.wrapping_add(1);
                self.layout_dirty = true;
                let removed_focus = focused_before
                    .filter(|node| !self.node_ids.contains(node))
                    .map(|target| UiEvent::new(target, focused_key, UiEventKind::Blurred));
                self.focus.sync(
                    &self.root,
                    &self.node_ids,
                    focused_before,
                    focus_visible_before,
                    removed_focus,
                );
            }
        }
        self.events.sync(&self.root, &self.node_ids);
        if update == TreeUpdate::Layout {
            self.sync_responsive_registry();
        }
        let transition_update = self.sync_transitions();
        let update = strongest_update(update, transition_update);
        self.layout_dirty |= update == TreeUpdate::Layout;
        update
    }

    pub fn mark_layout_clean(&mut self) {
        self.layout_dirty = false;
    }

    pub fn set_pointer_settings(&mut self, settings: argui_core::PointerSettings) {
        self.interaction.set_pointer_settings(settings);
    }

    #[must_use]
    pub fn node_id_at(&self, index: usize) -> Option<NodeId> {
        self.node_ids.get(index).copied()
    }

    #[must_use]
    pub fn key(&self, node: NodeId) -> Option<&str> {
        self.key_for(node)
    }

    #[must_use]
    pub fn resolve_node(&self, target: &crate::FocusTarget) -> Option<NodeId> {
        match target {
            crate::FocusTarget::Node(node) => self.node_ids.contains(node).then_some(*node),
            crate::FocusTarget::Key(key) => self
                .node_ids
                .iter()
                .copied()
                .find(|node| self.key_for(*node) == Some(key.as_str())),
        }
    }

    #[must_use]
    pub fn parent_of(&self, node: NodeId) -> Option<NodeId> {
        let index = self
            .node_ids
            .iter()
            .position(|candidate| *candidate == node)?;
        self.events
            .parent(index)
            .and_then(|parent| self.node_ids.get(parent).copied())
    }

    #[must_use]
    pub fn visual_states(&self, node: NodeId) -> crate::VisualStates {
        self.interaction.visual_states(node)
    }

    #[must_use]
    pub const fn focused_node(&self) -> Option<NodeId> {
        self.interaction.focused()
    }

    #[must_use]
    pub fn element_at(&self, index: usize) -> Option<&Element> {
        nth_element(&self.root, index)
    }

    #[must_use]
    pub fn text_input_value(&self, node: NodeId) -> Option<&str> {
        self.text_inputs.get(node).map(TextInputState::value)
    }

    #[must_use]
    pub fn text_input_display(&self, node: NodeId) -> Option<String> {
        self.text_inputs.get(node).map(TextInputState::display)
    }

    #[must_use]
    pub fn text_input_cursor(&self, node: NodeId) -> Option<usize> {
        self.text_inputs
            .get(node)
            .map(TextInputState::display_cursor)
    }

    #[must_use]
    pub fn text_input_position(&self, node: NodeId) -> Option<TextPosition> {
        self.text_inputs
            .get(node)
            .map(TextInputState::display_position)
    }

    #[must_use]
    pub fn text_input_selection(&self, node: NodeId) -> Option<(usize, usize)> {
        self.text_inputs
            .get(node)
            .and_then(TextInputState::selection)
    }

    #[must_use]
    pub fn text_input_selection_positions(
        &self,
        node: NodeId,
    ) -> Option<(TextPosition, TextPosition)> {
        self.text_inputs
            .get(node)
            .and_then(TextInputState::selection_positions)
    }

    #[must_use]
    pub fn text_input_should_reveal_cursor(&self, node: NodeId) -> bool {
        self.text_inputs.should_reveal_cursor(node)
    }

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
    pub fn mark_text_input_layout_clean(&mut self) {
        self.text_inputs.clear_cursor_reveals();
    }

    #[must_use]
    pub fn resolved_caret_frame(
        &self,
        node: NodeId,
        style: &crate::CaretStyle,
    ) -> crate::CaretFrame {
        style.sample(self.caret.elapsed(node))
    }

    pub fn ime_input(&mut self, input: ImeInput) -> InteractionUpdate {
        let Some(node) = self.interaction.focused() else {
            return InteractionUpdate::default();
        };
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.ime(input);
        self.text_input_update(node, result)
    }

    pub fn paste_text(&mut self, target: Option<NodeId>, text: &str) -> InteractionUpdate {
        let Some(node) = target.or_else(|| self.interaction.focused()) else {
            return InteractionUpdate::default();
        };
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.paste(text);
        self.text_input_update(node, result)
    }

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

    pub fn drag_text_cursor(&mut self, node: NodeId, index: usize) -> InteractionUpdate {
        let Some(result) = self.text_inputs.drag(node, index) else {
            return InteractionUpdate::default();
        };
        self.text_input_update(node, result)
    }

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

    #[must_use]
    pub const fn text_cursor_dragging(&self) -> bool {
        self.text_inputs.dragging()
    }

    pub fn release_text_cursor(&mut self) -> bool {
        self.text_inputs.release()
    }

    fn decorate(&mut self, raw: RawUpdate) -> InteractionUpdate {
        let text_input_changed = raw.events.iter().any(|(target, kind)| {
            matches!(kind, UiEventKind::Focused | UiEventKind::Blurred)
                && self.text_inputs.get(*target).is_some()
        });
        let events = raw
            .events
            .into_iter()
            .flat_map(|(target, kind)| self.event_deliveries(target, kind))
            .collect();
        let transition_update = self.sync_transitions();
        if text_input_changed {
            self.caret.reset();
        }
        InteractionUpdate {
            events,
            paint_changed: raw.paint_changed || transition_update == TreeUpdate::Paint,
            scroll_changed: transition_update == TreeUpdate::Scroll,
            layout_changed: transition_update == TreeUpdate::Layout,
            text_input_changed,
            ..InteractionUpdate::default()
        }
    }

    pub(crate) fn text_input_update(
        &mut self,
        node: NodeId,
        result: crate::text_input::EditResult,
    ) -> InteractionUpdate {
        let value = self
            .text_inputs
            .get(node)
            .map_or_else(String::new, |state| state.value().to_owned());
        let mut events = Vec::with_capacity(2);
        if result.changed {
            events.extend(self.event_deliveries(node, UiEventKind::TextChanged(value.clone())));
        }
        if result.submitted {
            events.extend(self.event_deliveries(node, UiEventKind::Submitted(value)));
        }
        if result.layout {
            self.text_inputs.request_cursor_reveal(node);
            self.caret.reset();
        }
        InteractionUpdate {
            events,
            paint_changed: result.layout,
            layout_changed: result.reshape,
            text_input_changed: result.layout,
            clipboard: result.clipboard,
            ..InteractionUpdate::default()
        }
    }

    fn key_for(&self, node: NodeId) -> Option<&str> {
        let index = self
            .node_ids
            .iter()
            .position(|candidate| *candidate == node)?;
        nth_element(&self.root, index)?.key.as_deref()
    }

    fn sync_text_inputs(&mut self) {
        let inputs = flattened(&self.root)
            .into_iter()
            .zip(self.node_ids.iter().copied())
            .filter_map(|(element, node)| match &element.kind {
                ElementKind::TextEditor {
                    value,
                    multiline,
                    read_only,
                    filter,
                    ..
                } => Some((node, value.clone(), *multiline, *read_only, *filter)),
                _ => None,
            });
        self.text_inputs.sync(inputs);
    }

    fn sync_animation_registry(&mut self) {
        self.animations = AnimationRegistry::new(&self.root);
    }

    fn element_for(&self, node: NodeId) -> Option<&Element> {
        let index = self
            .node_ids
            .iter()
            .position(|candidate| *candidate == node)?;
        nth_element(&self.root, index)
    }

    fn focused_animated_caret(&self) -> Option<NodeId> {
        let node = self.interaction.focused()?;
        let element = self.element_for(node)?;
        match &element.kind {
            ElementKind::TextEditor { caret, .. }
                if caret.is_animated() && !self.reduced_motion =>
            {
                Some(node)
            }
            _ => None,
        }
    }
}
