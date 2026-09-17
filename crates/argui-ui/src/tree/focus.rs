use std::collections::HashSet;

use argui_core::{Key, KeyInput, KeyState};

use crate::{
    Element, FocusRequest, FocusScope, FocusTarget, HitRegion, InitialFocus, InteractionUpdate,
    KeyboardActivation, NodeId, UiEvent, UiEventKind,
};

use super::UiTree;

#[derive(Clone, Debug)]
struct ScopeEntry {
    node: NodeId,
    members: Vec<NodeId>,
    visible: Vec<NodeId>,
    policy: FocusScope,
}

#[derive(Clone, Copy, Debug)]
struct ScopeFrame {
    node: NodeId,
    restore: Option<NodeId>,
    restore_visible: bool,
}

#[derive(Clone, Debug)]
enum FocusIntent {
    First {
        scope: NodeId,
        visible: Option<bool>,
    },
    Target {
        target: FocusTarget,
        within: Option<NodeId>,
        visible: Option<bool>,
    },
    Clear,
}

#[derive(Clone, Debug, Default)]
pub(super) struct FocusRegistry {
    scopes: Vec<ScopeEntry>,
    stack: Vec<ScopeFrame>,
    pending: Vec<FocusIntent>,
    events: Vec<UiEvent>,
    suspended: Option<NodeId>,
}

impl FocusRegistry {
    pub(super) fn new(root: &Element, ids: &[NodeId]) -> Self {
        let scopes = collect_scopes(root, ids);
        let mut registry = Self {
            scopes,
            ..Self::default()
        };
        for entry in &registry.scopes {
            registry.stack.push(ScopeFrame {
                node: entry.node,
                restore: None,
                restore_visible: false,
            });
            if let Some(initial) = &entry.policy.initial {
                registry.pending.push(initial_intent(entry.node, initial));
            }
        }
        registry
    }

    pub(super) fn sync(
        &mut self,
        root: &Element,
        ids: &[NodeId],
        focused_before: Option<NodeId>,
        focus_visible_before: bool,
        removed_focus: Option<UiEvent>,
    ) {
        let next = collect_scopes(root, ids);
        let present = next.iter().map(|entry| entry.node).collect::<HashSet<_>>();
        for frame in self
            .stack
            .iter()
            .rev()
            .filter(|frame| !present.contains(&frame.node))
        {
            if self
                .scopes
                .iter()
                .find(|entry| entry.node == frame.node)
                .is_some_and(|entry| entry.policy.restore)
                && focused_before.is_none_or(|focused| self.contains(frame.node, focused))
                && let Some(target) = frame.restore
            {
                self.pending.push(FocusIntent::Target {
                    target: target.into(),
                    within: None,
                    visible: Some(frame.restore_visible),
                });
            }
        }
        self.stack.retain(|frame| present.contains(&frame.node));
        let mounted = self
            .stack
            .iter()
            .map(|frame| frame.node)
            .collect::<HashSet<_>>();
        for entry in next.iter().filter(|entry| !mounted.contains(&entry.node)) {
            self.stack.push(ScopeFrame {
                node: entry.node,
                restore: focused_before,
                restore_visible: focus_visible_before,
            });
            if let Some(initial) = &entry.policy.initial {
                self.pending.push(initial_intent(entry.node, initial));
            }
        }
        self.scopes = next;
        if let Some(event) = removed_focus {
            self.events.push(event);
            if let Some(scope) = self.active_trap() {
                self.pending.insert(
                    0,
                    FocusIntent::First {
                        scope,
                        visible: Some(focus_visible_before),
                    },
                );
            }
        }
    }

    fn active_trap(&self) -> Option<NodeId> {
        self.stack.iter().rev().find_map(|frame| {
            self.entry(frame.node)
                .filter(|entry| entry.policy.traps())
                .map(|entry| entry.node)
        })
    }

    pub(super) fn active_modal(&self) -> Option<NodeId> {
        self.stack.iter().rev().find_map(|frame| {
            self.entry(frame.node)
                .filter(|entry| entry.policy.is_modal())
                .map(|entry| entry.node)
        })
    }

    pub(super) fn contains(&self, scope: NodeId, node: NodeId) -> bool {
        self.entry(scope)
            .is_some_and(|entry| entry.members.contains(&node))
    }

    pub(super) fn modal_visible(&self, node: NodeId) -> bool {
        self.active_modal()
            .and_then(|modal| self.entry(modal))
            .is_none_or(|entry| entry.visible.contains(&node))
    }

    fn entry(&self, node: NodeId) -> Option<&ScopeEntry> {
        self.scopes.iter().find(|entry| entry.node == node)
    }
}

impl UiTree {
    pub(crate) fn active_modal_scope(&self) -> Option<NodeId> {
        self.focus.active_modal()
    }

    pub(crate) fn semantic_focus_visible(&self, node: NodeId) -> bool {
        self.focus.modal_visible(node)
    }

    /// Processes a primary-button press and applies pointer focus defaults.
    ///
    /// * `regions` — current hit regions used to resolve focus behavior.
    pub fn primary_pressed(&mut self, regions: &[HitRegion]) -> InteractionUpdate {
        let position = self.interaction.mouse_position().unwrap_or_default();
        let event = argui_core::PointerEvent {
            phase: argui_core::PointerPhase::Pressed,
            ..argui_core::PointerEvent::mouse(argui_core::PointerPhase::Pressed, position)
        };
        let mut update = self.primary_pressed_for(event, regions);
        update.merge(self.focus_pointer_default(event.id, regions));
        update
    }

    pub(crate) fn primary_pressed_for(
        &mut self,
        event: argui_core::PointerEvent,
        _regions: &[HitRegion],
    ) -> InteractionUpdate {
        let raw = self.interaction.primary_pressed(event);
        self.decorate(raw)
    }

    /// Applies the default focus action for a pointer press.
    ///
    /// * `pointer` — identifier of the pointer that pressed.
    /// * `regions` — current hit regions and focus policies.
    pub fn focus_pointer_default(
        &mut self,
        pointer: argui_core::PointerId,
        regions: &[HitRegion],
    ) -> InteractionUpdate {
        let active = self.focus.active_trap();
        let scoped = regions
            .iter()
            .cloned()
            .map(|mut region| {
                if active.is_some_and(|scope| !self.focus.contains(scope, region.node)) {
                    region.focus_policy = crate::FocusPolicy::None;
                }
                region
            })
            .collect::<Vec<_>>();
        let raw = self
            .interaction
            .focus_pressed(pointer, &scoped, active.is_some());
        self.decorate(raw)
    }

    /// Delivers a key event to the focused node or the root fallback.
    ///
    /// * `input` — key and modifier state to deliver.
    /// * `regions` — hit regions accepted by the keyboard input path; not inspected here.
    pub fn keyboard_event(&mut self, input: &KeyInput, regions: &[HitRegion]) -> InteractionUpdate {
        let _ = regions;
        let target = self
            .interaction
            .focused()
            .or_else(|| self.node_ids.first().copied());
        if self.interaction.focused().is_none()
            && target.is_some_and(|node| {
                self.default_action(&UiEvent::new(
                    node,
                    None,
                    UiEventKind::KeyInput(input.clone()),
                ))
                .is_none()
            })
        {
            return InteractionUpdate::default();
        }
        InteractionUpdate {
            events: target.map_or_else(Vec::new, |node| {
                self.event_deliveries(node, UiEventKind::KeyInput(input.clone()))
            }),
            ..InteractionUpdate::default()
        }
    }

    /// Applies default keyboard focus and activation behavior.
    ///
    /// * `input` — key event whose default behavior is processed.
    /// * `regions` — current hit regions used for Tab navigation.
    pub fn keyboard_default(
        &mut self,
        input: &KeyInput,
        regions: &[HitRegion],
    ) -> InteractionUpdate {
        let Some(node) = self.interaction.focused() else {
            return if input.state == KeyState::Pressed && input.key == Key::Tab && !input.repeat {
                self.focus_next_scoped(regions, input.modifiers.shift)
            } else {
                InteractionUpdate::default()
            };
        };
        let mut update = InteractionUpdate::default();
        if input.state == KeyState::Pressed && input.key == Key::Tab && !input.repeat {
            update.merge(self.focus_next_scoped(regions, input.modifiers.shift));
            return update;
        }
        let activation = self
            .element_for(node)
            .and_then(|element| element.interaction.as_ref())
            .map_or(KeyboardActivation::None, |interaction| {
                interaction.keyboard_activation
            });
        let enter = input.key == Key::Enter
            && matches!(
                activation,
                KeyboardActivation::Enter | KeyboardActivation::EnterOrSpace
            );
        let space = matches!(&input.key, Key::Character(value) if value == " ")
            && activation == KeyboardActivation::EnterOrSpace;
        let raw = match (input.state, input.repeat, enter, space) {
            (KeyState::Pressed, false, true, _) => self.interaction.keyboard_clicked(node, input),
            (KeyState::Pressed, false, _, true) => self.interaction.keyboard_pressed(node),
            (KeyState::Released, _, _, true) => self.interaction.keyboard_released(input, true),
            _ => Default::default(),
        };
        update.merge(self.decorate(raw));
        update
    }

    /// Processes a key event, its defaults, and focused text editing.
    ///
    /// * `input` — key and modifier state to process.
    /// * `regions` — current hit regions for keyboard defaults.
    pub fn key_input(&mut self, input: &KeyInput, regions: &[HitRegion]) -> InteractionUpdate {
        let mut update = self.keyboard_event(input, regions);
        update.merge(self.keyboard_default(input, regions));
        if !(input.state == KeyState::Pressed && input.key == Key::Tab && !input.repeat) {
            update.merge(self.edit_text_input(input));
        }
        update
    }

    /// Applies a key event to the focused text input, if available.
    ///
    /// * `input` — key and modifier state to edit with.
    pub fn edit_text_input(&mut self, input: &KeyInput) -> InteractionUpdate {
        let Some(node) = self.interaction.focused() else {
            return InteractionUpdate::default();
        };
        if !self.input_available(node) {
            return InteractionUpdate::default();
        }
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.key(input);
        self.text_input_update(node, result)
    }

    /// Resolves queued and explicit focus requests against current hit regions.
    ///
    /// * `regions` — current focusable hit regions.
    /// * `request` — optional explicit focus or clear request.
    pub fn sync_focus(
        &mut self,
        regions: &[HitRegion],
        request: Option<FocusRequest>,
    ) -> InteractionUpdate {
        let mut update = InteractionUpdate {
            events: std::mem::take(&mut self.focus.events),
            ..InteractionUpdate::default()
        };
        update.paint_changed = !update.events.is_empty();
        let mut intents = std::mem::take(&mut self.focus.pending);
        if let Some(request) = request {
            intents.push(match request {
                FocusRequest::Focus(target) => FocusIntent::Target {
                    target,
                    within: None,
                    visible: Some(true),
                },
                FocusRequest::Clear => FocusIntent::Clear,
            });
        }
        for intent in intents {
            // A web window can receive focus before its first nonzero layout.
            if regions.is_empty() && !matches!(intent, FocusIntent::Clear) {
                self.focus.pending.push(intent);
                continue;
            }
            let active = self.focus.active_trap();
            let (target, visible) = match intent {
                FocusIntent::First { scope, visible } => {
                    (self.first_focusable(regions, scope), visible)
                }
                FocusIntent::Target {
                    target,
                    within,
                    visible,
                } => (
                    self.resolve_focus_target(regions, &target, within.or(active)),
                    visible,
                ),
                FocusIntent::Clear if active.is_none() => {
                    self.focus.pending.clear();
                    let raw = self.interaction.clear_focus();
                    update.merge(self.decorate(raw));
                    continue;
                }
                FocusIntent::Clear => continue,
            };
            if let Some(target) = target {
                let raw = match visible {
                    Some(visible) => self
                        .interaction
                        .focus_node_with_visibility(target, regions, visible),
                    None => self
                        .interaction
                        .focus_node_preserving_visibility(target, regions),
                };
                update.merge(self.decorate(raw));
            }
        }
        update
    }

    /// Restores suspended focus when the host window regains focus.
    ///
    /// * `regions` — current focusable hit regions.
    pub fn window_focused(&mut self, regions: &[HitRegion]) -> InteractionUpdate {
        let request = self
            .focus
            .suspended
            .take()
            .map(|node| FocusRequest::Focus(node.into()));
        self.sync_focus(regions, request)
    }

    pub(super) fn suspend_focus(&mut self) {
        self.focus.suspended = self.interaction.focused();
    }

    pub(super) fn focus_next_scoped(
        &mut self,
        regions: &[HitRegion],
        backwards: bool,
    ) -> InteractionUpdate {
        let active = self.focus.active_trap();
        let scoped = regions
            .iter()
            .filter(|region| active.is_none_or(|scope| self.focus.contains(scope, region.node)))
            .cloned()
            .collect::<Vec<_>>();
        let raw = self.interaction.focus_next(&scoped, backwards);
        self.decorate(raw)
    }

    fn first_focusable(&self, regions: &[HitRegion], scope: NodeId) -> Option<NodeId> {
        let mut candidates = regions.iter().filter(|region| {
            region.enabled
                && region.focus_policy.is_focusable()
                && self.focus.contains(scope, region.node)
        });
        candidates
            .clone()
            .find(|region| region.focus_policy.is_tab_stop())
            .or_else(|| candidates.next())
            .map(|region| region.node)
    }

    fn resolve_focus_target(
        &self,
        regions: &[HitRegion],
        target: &FocusTarget,
        within: Option<NodeId>,
    ) -> Option<NodeId> {
        let mut matches = regions.iter().filter(|region| {
            region.focus_policy.is_focusable()
                && within.is_none_or(|scope| self.focus.contains(scope, region.node))
                && match target {
                    FocusTarget::Node(node) => region.node == *node,
                    FocusTarget::Key(key) => self.key_for(region.node) == Some(key.as_str()),
                }
        });
        let node = matches.next()?.node;
        matches.next().is_none().then_some(node)
    }
}

fn initial_intent(scope: NodeId, initial: &InitialFocus) -> FocusIntent {
    match initial {
        InitialFocus::First => FocusIntent::First {
            scope,
            visible: None,
        },
        InitialFocus::Target(target) => FocusIntent::Target {
            target: target.clone(),
            within: Some(scope),
            visible: None,
        },
    }
}

fn collect_scopes(root: &Element, ids: &[NodeId]) -> Vec<ScopeEntry> {
    fn visit(
        element: &Element,
        ids: &[NodeId],
        cursor: &mut usize,
        output: &mut Vec<ScopeEntry>,
        ancestors: &mut Vec<NodeId>,
    ) -> Vec<NodeId> {
        let node = ids[*cursor];
        *cursor += 1;
        let slot = element.focus_scope.as_ref().map(|policy| {
            let slot = output.len();
            output.push(ScopeEntry {
                node,
                members: Vec::new(),
                visible: Vec::new(),
                policy: policy.clone(),
            });
            slot
        });
        let mut members = vec![node];
        ancestors.push(node);
        for child in &element.children {
            members.extend(visit(child, ids, cursor, output, ancestors));
        }
        ancestors.pop();
        if let Some(slot) = slot {
            output[slot].members.clone_from(&members);
            output[slot].visible.extend_from_slice(ancestors);
            output[slot].visible.extend_from_slice(&members);
        }
        members
    }

    let mut output = Vec::new();
    visit(root, ids, &mut 0, &mut output, &mut Vec::new());
    output
}
