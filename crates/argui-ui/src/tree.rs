use crate::interaction::{InteractionState, RawUpdate};
use crate::scroll::ScrollState;
use crate::text_input::TextInputStates;
use crate::traversal::flattened;
use crate::update::{classify_update, strongest_update};
use crate::{
    Element, ElementKind, GestureArena, InteractionUpdate, NodeId, UiEvent, UiEventKind, identity,
};

mod action;
mod animation;
mod editing;
mod event;
mod focus;
mod index;
mod pointer;
mod portal;
mod resolve;
mod responsive;
mod scroll;
mod text_input;
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
    Composite,
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
    pub(crate) desktop_backdrop_state: crate::DesktopBackdropState,
    root: Element,
    node_ids: Vec<NodeId>,
    pub(crate) index: index::TreeIndex,
    next_node_id: u64,
    pub(crate) interaction: InteractionState,
    focus: FocusRegistry,
    gestures: GestureArena,
    scroll: ScrollState,
    declared_scroll_offsets: std::collections::HashMap<NodeId, argui_core::Point>,
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
    native_portals: std::collections::HashMap<NodeId, argui_core::Rect>,
    interaction_bounds: std::collections::HashMap<NodeId, argui_core::Rect>,
    pressed_positions:
        std::collections::HashMap<NodeId, (argui_core::PointerId, argui_core::Point)>,
}

impl UiTree {
    /// Creates a retained UI tree from its root element.
    ///
    /// * `root` — element hierarchy to retain.
    #[must_use]
    pub fn new(root: Element) -> Self {
        let mut next_node_id = 1;
        let node_ids = identity::initial_ids(&root, &mut next_node_id);
        let animations = AnimationRegistry::new(&root);
        let focus = FocusRegistry::new(&root, &node_ids);
        let events = EventRegistry::default();
        let mut tree = Self {
            desktop_backdrop_state: crate::DesktopBackdropState::default(),
            index: index::TreeIndex::new(&root, &node_ids),
            root,
            node_ids,
            next_node_id,
            interaction: InteractionState::default(),
            focus,
            gestures: GestureArena::default(),
            scroll: ScrollState::default(),
            declared_scroll_offsets: std::collections::HashMap::new(),
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
            native_portals: Default::default(),
            interaction_bounds: Default::default(),
            pressed_positions: Default::default(),
        };
        tree.sync_text_inputs();
        tree.sync_responsive_registry();
        tree.sync_transitions();
        tree.sync_declared_scroll_offsets();
        tree
    }
    /// Returns the root element of the retained tree.
    #[must_use]
    pub const fn root(&self) -> &Element {
        &self.root
    }

    /// Returns stable node identifiers in tree preorder.
    #[must_use]
    pub fn node_ids(&self) -> &[NodeId] {
        &self.node_ids
    }

    /// Allocated capacity of dense traversal columns and stable IDs, sampled in
    /// O(1). Excludes hash tables, shared element descriptions and nested payloads.
    #[must_use]
    pub fn index_storage_bytes(&self) -> usize {
        self.node_ids.capacity() * size_of::<NodeId>() + self.index.column_bytes()
    }

    /// Returns the current retained-tree revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns whether a layout update has been requested.
    #[must_use]
    pub const fn layout_dirty(&self) -> bool {
        self.layout_dirty
    }

    /// Preorder positions of explicit layout boundaries and portal roots.
    /// The layout engine uses this sparse list without scanning every element.
    #[must_use]
    pub fn layout_root_indices(&self) -> &[usize] {
        self.index.layout_roots()
    }

    /// Returns counters describing the most recent tree update.
    #[must_use]
    pub const fn update_stats(&self) -> TreeUpdateStats {
        self.update_stats
    }

    /// Replaces the tree root and reports whether retained state changed.
    ///
    /// * `root` — new element hierarchy.
    pub fn replace(&mut self, root: Element) -> bool {
        self.update(root) != TreeUpdate::None
    }

    /// Reconciles a new element hierarchy with this tree's retained state.
    ///
    /// * `root` — new element hierarchy.
    ///
    /// Returns the strongest update required by the reconciliation.
    pub fn update(&mut self, mut root: Element) -> TreeUpdate {
        let mut stats = TreeUpdateStats::default();
        let update = classify_update(&self.root, &mut root, &mut stats);
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
                self.sync_animation_registry(false);
                self.focus.sync(
                    &self.root,
                    &self.node_ids,
                    focused_before,
                    focus_visible_before,
                    None,
                );
            }
            TreeUpdate::Composite | TreeUpdate::Paint | TreeUpdate::Scroll => {
                self.root = root;
                self.sync_animation_registry(false);
            }
            TreeUpdate::Layout => {
                let node_ids = identity::reconcile_ids(
                    &self.root,
                    &self.node_ids,
                    self.index.subtree_ends(),
                    &root,
                    &mut self.next_node_id,
                );
                let structure_changed = self.node_ids != node_ids;
                self.node_ids = node_ids;
                self.root = root;
                self.sync_animation_registry(structure_changed);
                self.interaction.retain(&self.node_ids);
                self.interaction_bounds
                    .retain(|node, _| self.node_ids.contains(node));
                self.pressed_positions
                    .retain(|node, _| self.node_ids.contains(node));
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
        self.events.sync(&self.index);
        if update == TreeUpdate::Layout {
            self.sync_responsive_registry();
        }
        let transition_update = self.sync_transitions();
        let scroll_update = if self.sync_declared_scroll_offsets() {
            TreeUpdate::Scroll
        } else {
            TreeUpdate::None
        };
        let update = strongest_update(update, strongest_update(transition_update, scroll_update));
        self.layout_dirty |= update == TreeUpdate::Layout;
        update
    }

    /// Marks layout as clean after the host has processed the pending layout.
    pub fn mark_layout_clean(&mut self) {
        self.layout_dirty = false;
    }

    /// Updates pointer-device settings used by interaction policy.
    ///
    /// * `settings` — current host pointer settings.
    pub fn set_pointer_settings(&mut self, settings: argui_core::PointerSettings) {
        self.interaction.set_pointer_settings(settings);
    }

    /// Returns the node identifier at a preorder index, if present.
    ///
    /// * `index` — zero-based preorder position.
    #[must_use]
    pub fn node_id_at(&self, index: usize) -> Option<NodeId> {
        self.node_ids.get(index).copied()
    }

    /// Returns a node's explicit key, if it has one.
    ///
    /// * `node` — retained node identifier.
    #[must_use]
    pub fn key(&self, node: NodeId) -> Option<&str> {
        self.key_for(node)
    }

    /// Resolves a node, key, or unique retained identity against the current tree.
    ///
    /// * `target` — node identifier, explicit key, or retained identity.
    #[must_use]
    pub fn resolve_node(&self, target: &crate::FocusTarget) -> Option<NodeId> {
        match target {
            crate::FocusTarget::Node(node) => self.node_ids.contains(node).then_some(*node),
            crate::FocusTarget::Key(key) => self
                .node_ids
                .iter()
                .copied()
                .find(|node| self.key_for(*node) == Some(key.as_str())),
            crate::FocusTarget::Identity(identity) => self.index.identity(identity),
        }
    }

    /// Returns the parent node, or `None` for an unknown node or the root.
    ///
    /// * `node` — retained node identifier.
    #[must_use]
    pub fn parent_of(&self, node: NodeId) -> Option<NodeId> {
        let index = self.index.position(node)?;
        self.index
            .parent(index)
            .and_then(|parent| self.node_ids.get(parent).copied())
    }

    /// Returns the active visual states for a node.
    ///
    /// * `node` — retained node identifier.
    #[must_use]
    pub fn visual_states(&self, node: NodeId) -> crate::VisualStates {
        self.interaction.visual_states(node)
    }

    /// Returns the currently focused node, if any.
    #[must_use]
    pub const fn focused_node(&self) -> Option<NodeId> {
        self.interaction.focused()
    }

    /// Returns the element at a preorder index, if present.
    ///
    /// * `index` — zero-based preorder position.
    #[must_use]
    pub fn element_at(&self, index: usize) -> Option<&Element> {
        self.index.at(index)
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
            composite_changed: transition_update == TreeUpdate::Composite,
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
        let crate::text_input::EditResult {
            edit,
            submitted,
            layout,
            reshape,
            clipboard,
        } = result;
        let mut events = Vec::with_capacity(2);
        if let Some(edit) = edit {
            events.extend(self.event_deliveries(node, UiEventKind::TextEdited(edit.edit)));
            if self.has_event_listener(node, crate::EventType::Input) {
                let value = self
                    .text_inputs
                    .get(node)
                    .map_or_else(String::new, |state| state.value().to_owned());
                events.extend(self.event_deliveries(node, UiEventKind::TextChanged(value)));
            }
        }
        if submitted {
            let value = self
                .text_inputs
                .get(node)
                .map_or_else(String::new, |state| state.value().to_owned());
            events.extend(self.event_deliveries(node, UiEventKind::Submitted(value)));
        }
        if layout {
            self.text_inputs.request_cursor_reveal(node);
            self.caret.reset();
        }
        InteractionUpdate {
            events,
            paint_changed: layout,
            layout_changed: reshape,
            text_input_changed: layout,
            clipboard,
            ..InteractionUpdate::default()
        }
    }

    fn key_for(&self, node: NodeId) -> Option<&str> {
        self.index.element(node)?.key.as_deref()
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
                } => Some(crate::text_input::RetainedInput {
                    node,
                    value: value.clone(),
                    multiline: *multiline,
                    read_only: *read_only,
                    filter: *filter,
                    privacy: element.text_privacy,
                    history: element.text_history,
                }),
                _ => None,
            });
        self.text_inputs.sync(inputs);
    }

    fn sync_animation_registry(&mut self, structure_changed: bool) {
        let bindings_changed = if structure_changed {
            self.index = index::TreeIndex::new(&self.root, &self.node_ids);
            true
        } else {
            self.index.sync(&self.root)
        };
        if bindings_changed {
            self.animations = AnimationRegistry::new(&self.root);
        }
    }

    /// Returns the retained element for a node identifier, if it exists.
    ///
    /// * `node` — retained node identifier.
    #[must_use]
    pub fn element_for(&self, node: NodeId) -> Option<&Element> {
        self.index.element(node)
    }

    fn focused_animated_caret(&self) -> Option<(NodeId, &crate::CaretAnimation)> {
        let node = self.interaction.focused()?;
        let element = self.element_for(node)?;
        match &element.kind {
            ElementKind::TextEditor { caret, .. } if !self.reduced_motion => {
                caret.animation.as_ref().map(|animation| (node, animation))
            }
            _ => None,
        }
    }
}
