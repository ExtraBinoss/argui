use argui_core::{
    ImeInput, KeyInput, Point, PointerEvent, PointerKind, PointerPhase, ScrollDelta, TextPosition,
};
use argui_paint::QuadStyle;

use crate::interaction::{InteractionState, RawUpdate};
use crate::scroll::ScrollState;
use crate::text_input::{TextInputState, TextInputStates};
use crate::transition::PaintTransitions;
use crate::traversal::{flattened, nth_element};
use crate::update::classify_update;
use crate::{
    Element, ElementKind, GestureArena, HitRegion, InteractionUpdate, NodeId, ScrollRegion,
    UiEvent, UiEventKind, VisualState, identity,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeUpdate {
    #[default]
    None,
    Semantics,
    Paint,
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
    interaction: InteractionState,
    gestures: GestureArena,
    scroll: ScrollState,
    text_inputs: TextInputStates,
    transitions: PaintTransitions,
    revision: u64,
    layout_dirty: bool,
    update_stats: TreeUpdateStats,
}

impl UiTree {
    #[must_use]
    pub fn new(root: Element) -> Self {
        let mut next_node_id = 1;
        let node_ids = identity::initial_ids(&root, &mut next_node_id);
        let mut tree = Self {
            root,
            node_ids,
            next_node_id,
            interaction: InteractionState::default(),
            gestures: GestureArena::default(),
            scroll: ScrollState::default(),
            text_inputs: TextInputStates::default(),
            transitions: PaintTransitions::default(),
            revision: 0,
            layout_dirty: true,
            update_stats: TreeUpdateStats::default(),
        };
        tree.sync_text_inputs();
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
        match update {
            TreeUpdate::None => return update,
            TreeUpdate::Semantics => {
                self.root = root;
            }
            TreeUpdate::Paint => {
                self.transitions
                    .sync(&self.root, &self.node_ids, &root, &self.node_ids);
                self.root = root;
            }
            TreeUpdate::Layout => {
                let node_ids = identity::reconcile_ids(
                    &self.root,
                    &self.node_ids,
                    &root,
                    &mut self.next_node_id,
                );
                self.transitions
                    .sync(&self.root, &self.node_ids, &root, &node_ids);
                self.node_ids = node_ids;
                self.root = root;
                self.interaction.retain(&self.node_ids);
                self.scroll.retain(&self.node_ids);
                self.sync_text_inputs();
                self.revision = self.revision.wrapping_add(1);
                self.layout_dirty = true;
            }
        }
        update
    }

    pub fn mark_layout_clean(&mut self) {
        self.layout_dirty = false;
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
    pub fn visual_state(&self, node: NodeId) -> VisualState {
        self.interaction.visual_state(node)
    }

    #[must_use]
    pub const fn focused_node(&self) -> Option<NodeId> {
        self.interaction.focused()
    }

    #[must_use]
    pub fn resolved_quad(&self, node: NodeId, element: &Element) -> QuadStyle {
        let base = self.transitions.resolve(node, element.paint.quad.clone());
        element
            .interaction
            .as_ref()
            .map_or(base.clone(), |interaction| {
                interaction.resolve(base, self.visual_state(node))
            })
    }

    #[must_use]
    pub fn resolved_transform(&self, node: NodeId, element: &Element) -> argui_core::Transform2D {
        self.transitions.resolve_transform(node, element.transform)
    }

    pub fn advance_animations(&mut self, now: argui_animation::Time) -> bool {
        self.transitions.advance(now)
    }

    pub fn set_reduced_motion(&mut self, reduced: bool) -> bool {
        self.transitions.set_reduced_motion(reduced)
    }

    #[must_use]
    pub fn wants_animation_frame(&self) -> bool {
        self.transitions.needs_frame()
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

    pub fn pointer_moved(&mut self, point: Point, regions: &[HitRegion]) -> InteractionUpdate {
        let update = self.interaction.pointer_moved(point, regions);
        self.decorate(update)
    }

    pub fn pointer_event(
        &mut self,
        event: PointerEvent,
        regions: &[HitRegion],
    ) -> InteractionUpdate {
        let hit = regions
            .iter()
            .rev()
            .find(|region| region.contains(event.position))
            .map(|region| (region.node, region.gestures));
        let gestures = self.gestures.update(event, hit);
        let mut update = if event.kind == PointerKind::Touch && !event.primary {
            InteractionUpdate::default()
        } else {
            match event.phase {
                PointerPhase::Entered | PointerPhase::Moved => {
                    self.pointer_moved(event.position, regions)
                }
                PointerPhase::Pressed => {
                    let mut moved = self.pointer_moved(event.position, regions);
                    moved.merge(self.primary_pressed(regions));
                    moved
                }
                PointerPhase::Released => {
                    let mut moved = self.pointer_moved(event.position, regions);
                    moved.merge(self.primary_released());
                    moved
                }
                PointerPhase::Left => self.pointer_left(),
                PointerPhase::Cancelled => {
                    let update = self.interaction.primary_cancelled();
                    self.decorate(update)
                }
            }
        };
        update
            .events
            .extend(gestures.into_iter().map(|gesture| UiEvent {
                target: gesture.target,
                key: self.key_for(gesture.target).map(ToOwned::to_owned),
                kind: UiEventKind::Gesture(gesture),
            }));
        update
    }

    pub fn pointer_left(&mut self) -> InteractionUpdate {
        let update = self.interaction.pointer_left();
        self.decorate(update)
    }

    pub fn primary_pressed(&mut self, regions: &[HitRegion]) -> InteractionUpdate {
        let update = self.interaction.primary_pressed(regions);
        self.decorate(update)
    }

    pub fn primary_released(&mut self) -> InteractionUpdate {
        let update = self.interaction.primary_released();
        self.decorate(update)
    }

    pub fn window_blurred(&mut self) -> InteractionUpdate {
        let update = self.interaction.window_blurred();
        let mut update = self.decorate(update);
        update.events.extend(
            self.gestures
                .cancel_all()
                .into_iter()
                .map(|gesture| UiEvent {
                    target: gesture.target,
                    key: self.key_for(gesture.target).map(ToOwned::to_owned),
                    kind: UiEventKind::Gesture(gesture),
                }),
        );
        update
    }

    pub fn focus_next(&mut self, regions: &[HitRegion], backwards: bool) -> InteractionUpdate {
        let update = self.interaction.focus_next(regions, backwards);
        self.decorate(update)
    }

    pub fn focus_node(&mut self, node: NodeId, regions: &[HitRegion]) -> InteractionUpdate {
        let update = self.interaction.focus_node(node, regions);
        self.decorate(update)
    }

    pub fn key_input(&mut self, input: &KeyInput) -> InteractionUpdate {
        let Some(node) = self.interaction.focused() else {
            return InteractionUpdate::default();
        };
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.key(input);
        self.text_input_update(node, result)
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

    pub fn paste_text(&mut self, text: &str) -> InteractionUpdate {
        let Some(node) = self.interaction.focused() else {
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

    #[must_use]
    pub fn scroll_offset(&self, node: NodeId) -> Point {
        self.scroll.offset(node)
    }

    pub fn set_scroll_offset(&mut self, node: NodeId, offset: Point) -> bool {
        self.scroll.set_offset(node, offset)
    }

    pub fn scroll(
        &mut self,
        point: Point,
        delta: ScrollDelta,
        regions: &[ScrollRegion],
    ) -> InteractionUpdate {
        let Some(outcome) = self.scroll.scroll(point, delta, regions) else {
            return InteractionUpdate::default();
        };
        match outcome {
            crate::scroll::ScrollOutcome::Changed(change) => self.scroll_update(change),
            crate::scroll::ScrollOutcome::Consumed => InteractionUpdate::default(),
        }
    }

    pub fn scrollbar_pressed(
        &mut self,
        point: Point,
        regions: &[ScrollRegion],
    ) -> Option<InteractionUpdate> {
        self.scroll.scrollbar_pressed(point, regions).map(|change| {
            change.map_or_else(InteractionUpdate::default, |change| {
                self.scroll_update(change)
            })
        })
    }

    pub fn scrollbar_dragged(
        &mut self,
        point: Point,
        regions: &[ScrollRegion],
    ) -> Option<InteractionUpdate> {
        if !self.scroll.dragging() {
            return None;
        }
        Some(
            self.scroll
                .scrollbar_dragged(point, regions)
                .map_or_else(InteractionUpdate::default, |change| {
                    self.scroll_update(change)
                }),
        )
    }

    pub fn scrollbar_released(&mut self) -> bool {
        self.scroll.scrollbar_released()
    }

    #[must_use]
    pub fn scrollbar_dragging(&self) -> bool {
        self.scroll.dragging()
    }

    fn scroll_update(&self, change: crate::scroll::ScrollChange) -> InteractionUpdate {
        InteractionUpdate {
            events: vec![UiEvent {
                target: change.node,
                key: self.key_for(change.node).map(ToOwned::to_owned),
                kind: UiEventKind::Scrolled {
                    delta: change.delta,
                    offset: change.offset,
                },
            }],
            paint_changed: true,
            scroll_changed: true,
            ..InteractionUpdate::default()
        }
    }

    fn decorate(&self, raw: RawUpdate) -> InteractionUpdate {
        let text_input_changed = raw.events[..raw.count]
            .iter()
            .flatten()
            .any(|(target, kind)| {
                matches!(kind, UiEventKind::Focused | UiEventKind::Blurred)
                    && self.text_inputs.get(*target).is_some()
            });
        let events = raw.events[..raw.count]
            .iter()
            .flatten()
            .map(|(target, kind)| UiEvent {
                target: *target,
                key: self.key_for(*target).map(ToOwned::to_owned),
                kind: kind.clone(),
            })
            .collect();
        InteractionUpdate {
            events,
            paint_changed: raw.paint_changed,
            scroll_changed: false,
            text_input_changed,
            ..InteractionUpdate::default()
        }
    }

    fn text_input_update(
        &self,
        node: NodeId,
        result: crate::text_input::EditResult,
    ) -> InteractionUpdate {
        let value = self
            .text_inputs
            .get(node)
            .map_or_else(String::new, |state| state.value().to_owned());
        let mut events = Vec::with_capacity(2);
        if result.changed {
            events.push(UiEvent {
                target: node,
                key: self.key_for(node).map(ToOwned::to_owned),
                kind: UiEventKind::TextChanged(value.clone()),
            });
        }
        if result.submitted {
            events.push(UiEvent {
                target: node,
                key: self.key_for(node).map(ToOwned::to_owned),
                kind: UiEventKind::Submitted(value),
            });
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
                ElementKind::TextInput { initial_value, .. } => Some((node, initial_value.clone())),
                _ => None,
            });
        self.text_inputs.sync(inputs);
    }
}
