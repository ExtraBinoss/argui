use argui_core::{Affine2D, KeyInput, Point, Rect};
use argui_paint::ClipChain;

use crate::{CursorIcon, GestureSet, VisualState, VisualStates};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(u64);

impl NodeId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum KeyboardActivation {
    #[default]
    None,
    Enter,
    EnterOrSpace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowDragBehavior {
    Move,
    MoveAndToggleMaximize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Interaction {
    pub enabled: bool,
    pub focusable: bool,
    pub cursor: CursorIcon,
    pub gestures: GestureSet,
    pub keyboard_activation: KeyboardActivation,
    pub window_drag: Option<WindowDragBehavior>,
}

impl Default for Interaction {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: false,
            cursor: CursorIcon::Auto,
            gestures: GestureSet::NONE,
            keyboard_activation: KeyboardActivation::None,
            window_drag: None,
        }
    }
}

impl Interaction {
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Creates a non-focusable hit target useful for popovers and other
    /// surfaces that must occlude pointer input behind them.
    #[must_use]
    pub const fn blocker() -> Self {
        Self {
            enabled: true,
            focusable: false,
            cursor: CursorIcon::Auto,
            gestures: GestureSet::NONE,
            keyboard_activation: KeyboardActivation::None,
            window_drag: None,
        }
    }

    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    #[must_use]
    pub const fn cursor(mut self, cursor: CursorIcon) -> Self {
        self.cursor = cursor;
        self
    }

    #[must_use]
    pub const fn gestures(mut self, gestures: GestureSet) -> Self {
        self.gestures = gestures;
        self
    }

    #[must_use]
    pub const fn keyboard_activation(mut self, activation: KeyboardActivation) -> Self {
        self.keyboard_activation = activation;
        self
    }

    #[must_use]
    pub const fn window_drag(mut self, behavior: WindowDragBehavior) -> Self {
        self.window_drag = Some(behavior);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HitRegion {
    pub node: NodeId,
    pub bounds: Rect,
    pub transform: Affine2D,
    pub clips: ClipChain,
    pub enabled: bool,
    pub focusable: bool,
    pub cursor: CursorIcon,
    pub gestures: GestureSet,
    pub window_drag: Option<WindowDragBehavior>,
}

impl HitRegion {
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        self.transform
            .inverse()
            .is_some_and(|inverse| self.bounds.contains(inverse.transform_point(point)))
            && self.clips.contains(point)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiEventKind {
    PointerEntered,
    PointerLeft,
    PointerMoved(Point),
    Pressed,
    Released,
    Clicked,
    KeyInput(KeyInput),
    Scrolled {
        delta: Point,
        offset: Point,
    },
    Focused,
    Blurred,
    TextChanged(String),
    Submitted(String),
    Gesture(crate::GestureEvent),
    SemanticAction {
        action: crate::SemanticAction,
        value: Option<crate::SemanticValue>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiEvent {
    pub target: NodeId,
    pub key: Option<String>,
    pub kind: UiEventKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InteractionUpdate {
    pub events: Vec<UiEvent>,
    pub paint_changed: bool,
    pub scroll_changed: bool,
    pub layout_changed: bool,
    pub text_input_changed: bool,
    pub clipboard: Option<crate::ClipboardRequest>,
}

impl InteractionUpdate {
    pub fn merge(&mut self, other: Self) {
        self.events.extend(other.events);
        self.paint_changed |= other.paint_changed;
        self.scroll_changed |= other.scroll_changed;
        self.layout_changed |= other.layout_changed;
        self.text_input_changed |= other.text_input_changed;
        if other.clipboard.is_some() {
            self.clipboard = other.clipboard;
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RawUpdate {
    pub events: [Option<(NodeId, UiEventKind)>; 5],
    pub count: usize,
    pub paint_changed: bool,
}

impl RawUpdate {
    fn push(&mut self, target: NodeId, kind: UiEventKind) {
        self.events[self.count] = Some((target, kind));
        self.count += 1;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct InteractionState {
    hovered: Option<NodeId>,
    pressed: Option<NodeId>,
    keyboard_pressed: Option<NodeId>,
    focused: Option<NodeId>,
    focus_visible: bool,
    captured: Option<NodeId>,
}

impl InteractionState {
    pub const fn focused(&self) -> Option<NodeId> {
        self.focused
    }
    pub fn visual_states(&self, node: NodeId) -> VisualStates {
        let mut states = VisualStates::NONE;
        if self.focused == Some(node) {
            states.insert(VisualState::Focused);
            if self.focus_visible {
                states.insert(VisualState::FocusVisible);
            }
        }
        if self.hovered == Some(node) {
            states.insert(VisualState::Hovered);
        }
        if self.pressed == Some(node) || self.keyboard_pressed == Some(node) {
            states.insert(VisualState::Pressed);
        }
        states
    }

    pub fn pointer_moved(&mut self, point: Point, regions: &[HitRegion]) -> RawUpdate {
        let hit = hit_test(regions, point).filter(|region| region.enabled);
        let mut update = RawUpdate::default();
        if hit.map(|region| region.node) != self.hovered {
            if let Some(previous) = self.hovered {
                update.push(previous, UiEventKind::PointerLeft);
            }
            self.hovered = hit.map(|region| region.node);
            if let Some(node) = self.hovered {
                update.push(node, UiEventKind::PointerEntered);
            }
            update.paint_changed = true;
        }
        if let Some(target) = self.captured.or(self.hovered) {
            update.push(target, UiEventKind::PointerMoved(point));
        }
        update
    }

    pub fn pointer_left(&mut self) -> RawUpdate {
        let mut update = RawUpdate::default();
        if let Some(node) = self.hovered.take() {
            update.push(node, UiEventKind::PointerLeft);
            update.paint_changed = true;
        }
        update
    }

    pub fn primary_pressed(&mut self, regions: &[HitRegion]) -> RawUpdate {
        let mut update = RawUpdate::default();
        let Some(target) = self.hovered else {
            return update;
        };
        self.pressed = Some(target);
        self.captured = Some(target);
        update.push(target, UiEventKind::Pressed);
        if regions
            .iter()
            .find(|region| region.node == target)
            .is_some_and(|region| region.focusable)
        {
            let focus_changed = self.focused != Some(target);
            self.release_keyboard(&mut update, false);
            if focus_changed {
                if let Some(previous) = self.focused.replace(target) {
                    update.push(previous, UiEventKind::Blurred);
                }
                update.push(target, UiEventKind::Focused);
            }
            self.focus_visible = false;
        }
        update.paint_changed = true;
        update
    }

    pub fn primary_released(&mut self) -> RawUpdate {
        let mut update = RawUpdate::default();
        if let Some(target) = self.captured.take() {
            update.push(target, UiEventKind::Released);
            if self.hovered == Some(target) {
                update.push(target, UiEventKind::Clicked);
            }
        }
        update.paint_changed = self.pressed.take().is_some();
        update
    }

    pub fn primary_cancelled(&mut self) -> RawUpdate {
        let mut update = RawUpdate::default();
        if let Some(target) = self.captured.take() {
            update.push(target, UiEventKind::Released);
        }
        update.paint_changed = self.pressed.take().is_some();
        update
    }

    pub fn focus_next(&mut self, regions: &[HitRegion], backwards: bool) -> RawUpdate {
        let focusable = regions
            .iter()
            .filter(|region| region.focusable)
            .map(|region| region.node)
            .collect::<Vec<_>>();
        if focusable.is_empty() {
            return RawUpdate::default();
        }
        let current = self
            .focused
            .and_then(|node| focusable.iter().position(|candidate| *candidate == node));
        let index = match (current, backwards) {
            (Some(0), true) | (None, true) => focusable.len() - 1,
            (Some(index), true) => index - 1,
            (Some(index), false) => (index + 1) % focusable.len(),
            (None, false) => 0,
        };
        let next = focusable[index];
        let mut update = RawUpdate::default();
        if self.focused == Some(next) {
            update.paint_changed = !self.focus_visible;
            self.focus_visible = true;
            return update;
        }
        self.release_keyboard(&mut update, false);
        if let Some(previous) = self.focused.replace(next) {
            update.push(previous, UiEventKind::Blurred);
        }
        self.focused = Some(next);
        self.focus_visible = true;
        update.push(next, UiEventKind::Focused);
        update.paint_changed = true;
        update
    }

    pub fn focus_node(&mut self, node: NodeId, regions: &[HitRegion]) -> RawUpdate {
        if !regions
            .iter()
            .any(|region| region.node == node && region.focusable)
            || self.focused == Some(node)
        {
            return RawUpdate::default();
        }
        let mut update = RawUpdate::default();
        self.release_keyboard(&mut update, false);
        if let Some(previous) = self.focused.replace(node) {
            update.push(previous, UiEventKind::Blurred);
        }
        self.focus_visible = true;
        update.push(node, UiEventKind::Focused);
        update.paint_changed = true;
        update
    }

    pub fn clear_focus(&mut self) -> RawUpdate {
        let mut update = RawUpdate::default();
        self.release_keyboard(&mut update, false);
        if let Some(target) = self.focused.take() {
            update.push(target, UiEventKind::Blurred);
            update.paint_changed = true;
        }
        self.focus_visible = false;
        update
    }

    pub fn keyboard_pressed(&mut self, node: NodeId) -> RawUpdate {
        let mut update = RawUpdate::default();
        if self.focused == Some(node) && !self.focus_visible {
            self.focus_visible = true;
            update.paint_changed = true;
        }
        if self.keyboard_pressed.replace(node) != Some(node) {
            update.push(node, UiEventKind::Pressed);
            update.paint_changed = true;
        }
        update
    }

    pub fn keyboard_released(&mut self, activate: bool) -> RawUpdate {
        let mut update = RawUpdate::default();
        self.release_keyboard(&mut update, activate);
        update
    }

    pub fn keyboard_clicked(&mut self, node: NodeId) -> RawUpdate {
        let mut update = RawUpdate::default();
        if self.focused == Some(node) {
            self.focus_visible = true;
        }
        update.push(node, UiEventKind::Pressed);
        update.push(node, UiEventKind::Clicked);
        update.push(node, UiEventKind::Released);
        update
    }

    pub fn window_blurred(&mut self) -> RawUpdate {
        let mut update = self.pointer_left();
        if let Some(target) = self.pressed.take() {
            update.push(target, UiEventKind::Released);
            update.paint_changed = true;
        }
        self.release_keyboard(&mut update, false);
        self.captured = None;
        if let Some(target) = self.focused.take() {
            update.push(target, UiEventKind::Blurred);
            update.paint_changed = true;
        }
        self.focus_visible = false;
        update
    }

    pub fn retain(&mut self, ids: &[NodeId]) {
        let exists = |candidate: Option<NodeId>| candidate.is_some_and(|id| ids.contains(&id));
        if !exists(self.hovered) {
            self.hovered = None;
        }
        if !exists(self.pressed) {
            self.pressed = None;
        }
        if !exists(self.keyboard_pressed) {
            self.keyboard_pressed = None;
        }
        if !exists(self.focused) {
            self.focused = None;
            self.focus_visible = false;
        }
        if !exists(self.captured) {
            self.captured = None;
        }
    }

    fn release_keyboard(&mut self, update: &mut RawUpdate, activate: bool) {
        if let Some(target) = self.keyboard_pressed.take() {
            update.push(target, UiEventKind::Released);
            if activate && self.focused == Some(target) {
                update.push(target, UiEventKind::Clicked);
            }
            update.paint_changed = true;
        }
    }
}

fn hit_test(regions: &[HitRegion], point: Point) -> Option<&HitRegion> {
    regions.iter().rev().find(|region| region.contains(point))
}
