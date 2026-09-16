use std::collections::HashMap;

use argui_core::{Affine2D, Point, PointerId, Rect, Size};
use argui_paint::{ClipChain, CornerRadii};

use crate::{CursorIcon, GestureSet, Sides, UiEventKind, VisualState, VisualStates};

mod pointer;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PointerEvents {
    #[default]
    Auto,
    None,
    BoxOnly,
    ContentsOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum HitShape {
    #[default]
    Bounds,
    RoundedRect(CornerRadii),
    Ellipse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitTestStyle {
    pub pointer_events: PointerEvents,
    pub shape: HitShape,
    pub slop: Sides<f32>,
}

impl Default for HitTestStyle {
    fn default() -> Self {
        Self {
            pointer_events: PointerEvents::Auto,
            shape: HitShape::Bounds,
            slop: Sides {
                left: 0.0,
                right: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
        }
    }
}

impl HitTestStyle {
    /// Sets how pointer events are assigned between this element and its children.
    ///
    /// * `pointer_events` — pointer assignment policy.
    #[must_use]
    pub const fn pointer_events(mut self, pointer_events: PointerEvents) -> Self {
        self.pointer_events = pointer_events;
        self
    }

    /// Sets the shape used to test pointer hits.
    #[must_use]
    pub const fn shape(mut self, shape: HitShape) -> Self {
        self.shape = shape;
        self
    }

    /// Expands the hit region by per-side logical pixel distances.
    ///
    /// * `slop` — additional hit distance on each side.
    #[must_use]
    pub const fn slop(mut self, slop: Sides<f32>) -> Self {
        self.slop = slop;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(u64);

impl NodeId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the opaque numeric value of this node identity.
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
    pub focus_policy: crate::FocusPolicy,
    pub cursor: CursorIcon,
    pub gestures: GestureSet,
    pub keyboard_activation: KeyboardActivation,
    pub window_drag: Option<WindowDragBehavior>,
}

impl Default for Interaction {
    fn default() -> Self {
        Self {
            enabled: true,
            focus_policy: crate::FocusPolicy::None,
            cursor: CursorIcon::Auto,
            gestures: GestureSet::EMPTY,
            keyboard_activation: KeyboardActivation::None,
            window_drag: None,
        }
    }
}

impl Interaction {
    /// Sets whether this element responds to interaction.
    ///
    /// * `enabled` — whether interaction is enabled.
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
            focus_policy: crate::FocusPolicy::None,
            cursor: CursorIcon::Auto,
            gestures: GestureSet::EMPTY,
            keyboard_activation: KeyboardActivation::None,
            window_drag: None,
        }
    }

    /// Sets the element's focus policy.
    #[must_use]
    pub const fn focus_policy(mut self, policy: crate::FocusPolicy) -> Self {
        self.focus_policy = policy;
        self
    }

    /// Sets the cursor requested while the pointer is over this element.
    #[must_use]
    pub const fn cursor(mut self, cursor: CursorIcon) -> Self {
        self.cursor = cursor;
        self
    }

    /// Sets the gestures recognized on this element.
    #[must_use]
    pub const fn gestures(mut self, gestures: GestureSet) -> Self {
        self.gestures = gestures;
        self
    }

    /// Sets which keyboard keys activate this element.
    ///
    /// * `activation` — keyboard activation policy.
    #[must_use]
    pub const fn keyboard_activation(mut self, activation: KeyboardActivation) -> Self {
        self.keyboard_activation = activation;
        self
    }

    /// Sets the window dragging behavior for this element.
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
    pub shape: HitShape,
    pub slop: Sides<f32>,
    pub enabled: bool,
    pub focus_policy: crate::FocusPolicy,
    pub cursor: CursorIcon,
    pub gestures: GestureSet,
    pub window_drag: Option<WindowDragBehavior>,
}

impl HitRegion {
    /// Tests whether a point lies in this hit region and within its clip chain.
    ///
    /// * `point` — point in the coordinate space of the hit region's transform.
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        self.transform.inverse().is_some_and(|inverse| {
            let point = inverse.transform_point(point);
            let bounds = expanded(self.bounds, self.slop);
            shape_contains(bounds, self.shape, point)
        }) && self.clips.contains(point)
    }
}

fn expanded(bounds: Rect, slop: Sides<f32>) -> Rect {
    let left = slop.left.max(0.0);
    let right = slop.right.max(0.0);
    let top = slop.top.max(0.0);
    let bottom = slop.bottom.max(0.0);
    Rect::new(
        Point::new(bounds.origin.x - left, bounds.origin.y - top),
        Size::new(
            (bounds.size.width + left + right).max(0.0),
            (bounds.size.height + top + bottom).max(0.0),
        ),
    )
}

fn shape_contains(bounds: Rect, shape: HitShape, point: Point) -> bool {
    if !bounds.contains(point) {
        return false;
    }
    match shape {
        HitShape::Bounds => true,
        HitShape::RoundedRect(radii) => rounded_rect_contains(bounds, radii, point),
        HitShape::Ellipse => ellipse_contains(bounds, point),
    }
}

fn rounded_rect_contains(bounds: Rect, radii: CornerRadii, point: Point) -> bool {
    let local = Point::new(point.x - bounds.origin.x, point.y - bounds.origin.y);
    let width = bounds.size.width.max(0.0);
    let height = bounds.size.height.max(0.0);
    let radius = if local.y < height * 0.5 {
        if local.x < width * 0.5 {
            radii.top_left
        } else {
            radii.top_right
        }
    } else if local.x < width * 0.5 {
        radii.bottom_left
    } else {
        radii.bottom_right
    }
    .clamp(0.0, width.min(height) * 0.5);
    let center = Point::new(
        local.x.clamp(radius, width - radius),
        local.y.clamp(radius, height - radius),
    );
    let delta = Point::new(local.x - center.x, local.y - center.y);
    delta.x * delta.x + delta.y * delta.y <= radius * radius
}

fn ellipse_contains(bounds: Rect, point: Point) -> bool {
    let radius_x = bounds.size.width * 0.5;
    let radius_y = bounds.size.height * 0.5;
    if radius_x <= 0.0 || radius_y <= 0.0 {
        return false;
    }
    let x = (point.x - bounds.origin.x - radius_x) / radius_x;
    let y = (point.y - bounds.origin.y - radius_y) / radius_y;
    x * x + y * y <= 1.0
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InteractionUpdate {
    pub events: Vec<crate::UiEvent>,
    pub paint_changed: bool,
    pub scroll_changed: bool,
    pub layout_changed: bool,
    pub text_input_changed: bool,
    pub clipboard: Option<crate::ClipboardRequest>,
    pub frame_requested: bool,
}

impl InteractionUpdate {
    /// Combines another update's events and change flags into this update.
    ///
    /// * `other` — update whose effects are merged into this value.
    pub fn merge(&mut self, other: Self) {
        self.events.extend(other.events);
        self.paint_changed |= other.paint_changed;
        self.scroll_changed |= other.scroll_changed;
        self.layout_changed |= other.layout_changed;
        self.text_input_changed |= other.text_input_changed;
        if other.clipboard.is_some() {
            self.clipboard = other.clipboard;
        }
        self.frame_requested |= other.frame_requested;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RawUpdate {
    pub events: Vec<(NodeId, crate::UiEventKind)>,
    pub paint_changed: bool,
}

impl RawUpdate {
    fn push(&mut self, target: NodeId, kind: crate::UiEventKind) {
        self.events.push((target, kind));
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct InteractionState {
    hovered: HashMap<PointerId, NodeId>,
    mouse_position: Option<Point>,
    pressed: HashMap<PointerId, pointer::PressRecord>,
    keyboard_pressed: Option<NodeId>,
    focused: Option<NodeId>,
    focus_visible: bool,
    captured: HashMap<PointerId, NodeId>,
    last_click: Option<pointer::ClickRecord>,
    pointer_settings: argui_core::PointerSettings,
}

impl InteractionState {
    pub fn set_pointer_settings(&mut self, settings: argui_core::PointerSettings) {
        self.pointer_settings = settings;
        self.last_click = None;
    }

    pub const fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    pub(crate) fn captured_node(&self, pointer: PointerId) -> Option<NodeId> {
        self.captured.get(&pointer).copied()
    }

    /// Returns the most recently observed mouse position.
    ///
    /// Returns `None` before the tree receives a mouse position or after the pointer leaves.
    pub(crate) const fn mouse_position(&self) -> Option<Point> {
        self.mouse_position
    }

    pub fn visual_states(&self, node: NodeId) -> VisualStates {
        let mut states = VisualStates::NONE;
        if self.focused == Some(node) {
            states.insert(VisualState::Focused);
            if self.focus_visible {
                states.insert(VisualState::FocusVisible);
            }
        }
        if self.hovered.values().any(|hovered| *hovered == node) {
            states.insert(VisualState::Hovered);
        }
        if self.pressed.iter().any(|(pointer, press)| {
            press.target == node
                && (!press.activation_cancelled || self.captured.get(pointer) == Some(&node))
        }) || self.keyboard_pressed == Some(node)
        {
            states.insert(VisualState::Pressed);
        }
        states
    }

    pub fn focus_pressed(
        &mut self,
        pointer: PointerId,
        regions: &[HitRegion],
        preserve_on_background: bool,
    ) -> RawUpdate {
        let target = self
            .pressed
            .get(&pointer)
            .map(|press| press.target)
            .filter(|target| {
                regions.iter().any(|region| {
                    region.node == *target && region.enabled && region.focus_policy.is_focusable()
                })
            });
        let Some(target) = target else {
            return if preserve_on_background {
                RawUpdate::default()
            } else {
                self.clear_focus()
            };
        };
        let mut update = RawUpdate::default();
        let focus_changed = self.focused != Some(target);
        self.release_keyboard(&mut update, None);
        if focus_changed {
            if let Some(previous) = self.focused.replace(target) {
                update.push(previous, UiEventKind::Blurred);
            }
            update.push(target, UiEventKind::Focused);
        }
        self.focus_visible = false;
        update
    }

    pub fn focus_next(&mut self, regions: &[HitRegion], backwards: bool) -> RawUpdate {
        let current = self
            .focused
            .and_then(|node| regions.iter().position(|region| region.node == node));
        let mut candidates = regions
            .iter()
            .enumerate()
            .filter(|(_, region)| region.enabled && region.focus_policy.is_tab_stop());
        let next = if backwards {
            candidates
                .clone()
                .rev()
                .find(|(i, _)| current.is_none_or(|current| *i < current))
                .or_else(|| candidates.next_back())
        } else {
            candidates
                .clone()
                .find(|(i, _)| current.is_none_or(|current| *i > current))
                .or_else(|| candidates.clone().next())
        };
        let Some((_, next)) = next else {
            return RawUpdate::default();
        };
        let next = next.node;
        let mut update = RawUpdate::default();
        if self.focused == Some(next) {
            update.paint_changed = !self.focus_visible;
            self.focus_visible = true;
            return update;
        }
        self.release_keyboard(&mut update, None);
        if let Some(previous) = self.focused.replace(next) {
            update.push(previous, UiEventKind::Blurred);
        }
        self.focused = Some(next);
        self.focus_visible = true;
        update.push(next, UiEventKind::Focused);
        update.paint_changed = true;
        update
    }

    pub fn focus_node_preserving_visibility(
        &mut self,
        node: NodeId,
        regions: &[HitRegion],
    ) -> RawUpdate {
        self.focus_node_with_visibility(node, regions, self.focus_visible)
    }

    pub fn focus_node_with_visibility(
        &mut self,
        node: NodeId,
        regions: &[HitRegion],
        focus_visible: bool,
    ) -> RawUpdate {
        if !regions
            .iter()
            .any(|region| region.node == node && region.focus_policy.is_focusable())
            || self.focused == Some(node)
        {
            return RawUpdate::default();
        }
        let mut update = RawUpdate::default();
        self.release_keyboard(&mut update, None);
        if let Some(previous) = self.focused.replace(node) {
            update.push(previous, UiEventKind::Blurred);
        }
        self.focus_visible = focus_visible;
        update.push(node, UiEventKind::Focused);
        update.paint_changed = true;
        update
    }

    pub fn clear_focus(&mut self) -> RawUpdate {
        let mut update = RawUpdate::default();
        self.release_keyboard(&mut update, None);
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
            update.paint_changed = true;
        }
        update
    }

    pub fn keyboard_released(&mut self, input: &argui_core::KeyInput, activate: bool) -> RawUpdate {
        let mut update = RawUpdate::default();
        self.release_keyboard(&mut update, activate.then(|| input.clone()));
        update
    }

    pub fn keyboard_clicked(&mut self, node: NodeId, input: &argui_core::KeyInput) -> RawUpdate {
        let mut update = RawUpdate::default();
        if self.focused == Some(node) {
            self.focus_visible = true;
        }
        update.push(
            node,
            UiEventKind::Click(crate::ClickEvent::keyboard(input.clone())),
        );
        update
    }

    pub fn retain(&mut self, ids: &[NodeId]) {
        let exists = |candidate: Option<NodeId>| candidate.is_some_and(|id| ids.contains(&id));
        self.hovered.retain(|_, node| ids.contains(node));
        self.pressed.retain(|_, press| ids.contains(&press.target));
        if !exists(self.keyboard_pressed) {
            self.keyboard_pressed = None;
        }
        if !exists(self.focused) {
            self.focused = None;
            self.focus_visible = false;
        }
        self.captured.retain(|_, node| ids.contains(node));
    }

    fn release_keyboard(
        &mut self,
        update: &mut RawUpdate,
        activation: Option<argui_core::KeyInput>,
    ) {
        if let Some(target) = self.keyboard_pressed.take() {
            if let Some(input) = activation
                && self.focused == Some(target)
            {
                update.push(
                    target,
                    UiEventKind::Click(crate::ClickEvent::keyboard(input)),
                );
            }
            update.paint_changed = true;
        }
    }
}
