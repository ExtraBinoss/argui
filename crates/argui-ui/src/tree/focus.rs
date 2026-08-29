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
}

#[derive(Clone, Debug)]
enum FocusIntent {
    First(NodeId),
    Target {
        target: FocusTarget,
        within: Option<NodeId>,
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
                && let Some(target) = frame.restore
            {
                self.pending.push(FocusIntent::Target {
                    target: target.into(),
                    within: None,
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
            });
            if let Some(initial) = &entry.policy.initial {
                self.pending.push(initial_intent(entry.node, initial));
            }
        }
        self.scopes = next;
        if let Some(event) = removed_focus {
            self.events.push(event);
            if let Some(scope) = self.active_trap() {
                self.pending.insert(0, FocusIntent::First(scope));
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

    pub fn primary_pressed(&mut self, regions: &[HitRegion]) -> InteractionUpdate {
        let active = self.focus.active_trap();
        let scoped = regions
            .iter()
            .cloned()
            .map(|mut region| {
                region.focusable &=
                    active.is_none_or(|scope| self.focus.contains(scope, region.node));
                region
            })
            .collect::<Vec<_>>();
        let raw = self.interaction.primary_pressed(&scoped);
        self.decorate(raw)
    }

    pub fn keyboard_event(&mut self, input: &KeyInput, regions: &[HitRegion]) -> InteractionUpdate {
        let Some(node) = self.interaction.focused() else {
            return if input.state == KeyState::Pressed && input.key == Key::Tab && !input.repeat {
                self.focus_next_scoped(regions, input.modifiers.shift)
            } else {
                InteractionUpdate::default()
            };
        };
        let mut update = InteractionUpdate {
            events: vec![UiEvent {
                target: node,
                key: self.key_for(node).map(ToOwned::to_owned),
                kind: UiEventKind::KeyInput(input.clone()),
            }],
            ..InteractionUpdate::default()
        };
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
            (KeyState::Pressed, false, true, _) => self.interaction.keyboard_clicked(node),
            (KeyState::Pressed, false, _, true) => self.interaction.keyboard_pressed(node),
            (KeyState::Released, _, _, true) => self.interaction.keyboard_released(true),
            _ => Default::default(),
        };
        update.merge(self.decorate(raw));
        update
    }

    pub fn key_input(&mut self, input: &KeyInput, regions: &[HitRegion]) -> InteractionUpdate {
        let mut update = self.keyboard_event(input, regions);
        if !(input.state == KeyState::Pressed && input.key == Key::Tab && !input.repeat) {
            update.merge(self.edit_text_input(input));
        }
        update
    }

    pub fn edit_text_input(&mut self, input: &KeyInput) -> InteractionUpdate {
        let Some(node) = self.interaction.focused() else {
            return InteractionUpdate::default();
        };
        let Some(state) = self.text_inputs.get_mut(node) else {
            return InteractionUpdate::default();
        };
        let result = state.key(input);
        self.text_input_update(node, result)
    }

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
                },
                FocusRequest::Clear => FocusIntent::Clear,
            });
        }
        for intent in intents {
            let active = self.focus.active_trap();
            let target = match intent {
                FocusIntent::First(scope) => self.first_focusable(regions, scope),
                FocusIntent::Target { target, within } => {
                    self.resolve_focus_target(regions, &target, within.or(active))
                }
                FocusIntent::Clear if active.is_none() => {
                    let raw = self.interaction.clear_focus();
                    update.merge(self.decorate(raw));
                    continue;
                }
                FocusIntent::Clear => continue,
            };
            if let Some(target) = target {
                let raw = self.interaction.focus_node(target, regions);
                update.merge(self.decorate(raw));
            }
        }
        update
    }

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
        regions
            .iter()
            .find(|region| region.focusable && self.focus.contains(scope, region.node))
            .map(|region| region.node)
    }

    fn resolve_focus_target(
        &self,
        regions: &[HitRegion],
        target: &FocusTarget,
        within: Option<NodeId>,
    ) -> Option<NodeId> {
        let mut matches = regions.iter().filter(|region| {
            region.focusable
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
        InitialFocus::First => FocusIntent::First(scope),
        InitialFocus::Target(target) => FocusIntent::Target {
            target: target.clone(),
            within: Some(scope),
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
