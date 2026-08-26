use argui_core::{Point, Rect};
use argui_paint::QuadStyle;

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
pub enum VisualState {
    #[default]
    Rest,
    Hovered,
    Pressed,
    Focused,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InteractionStyles {
    pub hovered: Option<QuadStyle>,
    pub pressed: Option<QuadStyle>,
    pub focused: Option<QuadStyle>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interaction {
    pub enabled: bool,
    pub focusable: bool,
    pub styles: InteractionStyles,
}

impl Default for Interaction {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: false,
            styles: InteractionStyles::default(),
        }
    }
}

impl Interaction {
    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    #[must_use]
    pub const fn hovered(mut self, style: QuadStyle) -> Self {
        self.styles.hovered = Some(style);
        self
    }

    #[must_use]
    pub const fn pressed(mut self, style: QuadStyle) -> Self {
        self.styles.pressed = Some(style);
        self
    }

    #[must_use]
    pub const fn focused(mut self, style: QuadStyle) -> Self {
        self.styles.focused = Some(style);
        self
    }

    #[must_use]
    pub fn resolve(self, base: QuadStyle, state: VisualState) -> QuadStyle {
        match state {
            VisualState::Pressed => self.styles.pressed,
            VisualState::Hovered => self.styles.hovered,
            VisualState::Focused => self.styles.focused,
            VisualState::Rest => None,
        }
        .unwrap_or(base)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitRegion {
    pub node: NodeId,
    pub bounds: Rect,
    pub clip: Rect,
    pub focusable: bool,
}

impl HitRegion {
    #[must_use]
    pub fn contains(self, point: Point) -> bool {
        self.bounds.contains(point) && self.clip.contains(point)
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
    Scrolled { delta: Point, offset: Point },
    Focused,
    Blurred,
    TextChanged(String),
    Submitted(String),
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
    focused: Option<NodeId>,
    captured: Option<NodeId>,
}

impl InteractionState {
    pub const fn focused(&self) -> Option<NodeId> {
        self.focused
    }
    pub fn visual_state(&self, node: NodeId) -> VisualState {
        if self.pressed == Some(node) {
            VisualState::Pressed
        } else if self.hovered == Some(node) {
            VisualState::Hovered
        } else if self.focused == Some(node) {
            VisualState::Focused
        } else {
            VisualState::Rest
        }
    }

    pub fn pointer_moved(&mut self, point: Point, regions: &[HitRegion]) -> RawUpdate {
        let hit = hit_test(regions, point);
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
            && self.focused != Some(target)
        {
            if let Some(previous) = self.focused.replace(target) {
                update.push(previous, UiEventKind::Blurred);
            }
            update.push(target, UiEventKind::Focused);
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
            return update;
        }
        if let Some(previous) = self.focused.replace(next) {
            update.push(previous, UiEventKind::Blurred);
        }
        self.focused = Some(next);
        update.push(next, UiEventKind::Focused);
        update.paint_changed = true;
        update
    }

    pub fn window_blurred(&mut self) -> RawUpdate {
        let mut update = self.pointer_left();
        if let Some(target) = self.pressed.take() {
            update.push(target, UiEventKind::Released);
            update.paint_changed = true;
        }
        self.captured = None;
        if let Some(target) = self.focused.take() {
            update.push(target, UiEventKind::Blurred);
            update.paint_changed = true;
        }
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
        if !exists(self.focused) {
            self.focused = None;
        }
        if !exists(self.captured) {
            self.captured = None;
        }
    }
}

fn hit_test(regions: &[HitRegion], point: Point) -> Option<HitRegion> {
    regions
        .iter()
        .rev()
        .copied()
        .find(|region| region.contains(point))
}
