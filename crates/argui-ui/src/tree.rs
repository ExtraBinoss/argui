use argui_core::Point;
use argui_paint::{Border, ClipBehavior, Color, CornerRadii, Fill, PaintStyle, QuadStyle};
use argui_text::TextStyle;

use crate::interaction::{InteractionState, RawUpdate};
use crate::{
    Align, Direction, Edges, HitRegion, Interaction, InteractionUpdate, Justify, LayoutStyle,
    Length, NodeId, UiEvent, VisualState, Wrap, identity,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ElementKind {
    Container,
    Text { content: String, style: TextStyle },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeUpdate {
    #[default]
    None,
    Paint,
    Layout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub key: Option<String>,
    pub kind: ElementKind,
    pub style: LayoutStyle,
    pub paint: PaintStyle,
    pub interaction: Option<Interaction>,
    pub children: Vec<Self>,
}

impl Element {
    #[must_use]
    pub fn container(children: impl IntoIterator<Item = Self>) -> Self {
        Self {
            key: None,
            kind: ElementKind::Container,
            style: LayoutStyle::default(),
            paint: PaintStyle::default(),
            interaction: None,
            children: children.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn row(children: impl IntoIterator<Item = Self>) -> Self {
        Self::container(children).direction(Direction::Row)
    }

    #[must_use]
    pub fn column(children: impl IntoIterator<Item = Self>) -> Self {
        Self::container(children)
    }

    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self {
            key: None,
            kind: ElementKind::Text {
                content: value.into(),
                style: TextStyle::default(),
            },
            style: LayoutStyle::default(),
            paint: PaintStyle::default(),
            interaction: None,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn keyed(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    #[must_use]
    pub fn text_style(mut self, style: TextStyle) -> Self {
        if let ElementKind::Text {
            style: text_style, ..
        } = &mut self.kind
        {
            *text_style = style;
        }
        self
    }

    #[must_use]
    pub fn layout_style(mut self, style: LayoutStyle) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub const fn paint_style(mut self, style: PaintStyle) -> Self {
        self.paint = style;
        self
    }

    #[must_use]
    pub const fn fill(mut self, fill: Fill) -> Self {
        self.paint.quad.background = Some(fill);
        self
    }

    #[must_use]
    pub const fn width(mut self, width: Length) -> Self {
        self.style.width = width;
        self
    }

    #[must_use]
    pub const fn height(mut self, height: Length) -> Self {
        self.style.height = height;
        self
    }

    #[must_use]
    pub const fn direction(mut self, direction: Direction) -> Self {
        self.style.direction = direction;
        self
    }

    #[must_use]
    pub const fn wrap(mut self, wrap: Wrap) -> Self {
        self.style.wrap = wrap;
        self
    }

    #[must_use]
    pub const fn align(mut self, align: Align) -> Self {
        self.style.align = align;
        self
    }

    #[must_use]
    pub const fn justify(mut self, justify: Justify) -> Self {
        self.style.justify = justify;
        self
    }

    #[must_use]
    pub const fn padding(mut self, padding: Edges) -> Self {
        self.style.padding = padding;
        self
    }

    #[must_use]
    pub const fn gap(mut self, gap: f32) -> Self {
        self.style.gap = gap;
        self
    }

    #[must_use]
    pub const fn grow(mut self, grow: f32) -> Self {
        self.style.grow = grow;
        self
    }

    #[must_use]
    pub const fn shrink(mut self, shrink: f32) -> Self {
        self.style.shrink = shrink;
        self
    }

    #[must_use]
    pub const fn background(mut self, color: Color) -> Self {
        self = self.fill(Fill::Solid(color));
        self
    }

    #[must_use]
    pub const fn border(mut self, border: Border) -> Self {
        self.paint.quad.border = Some(border);
        self
    }

    #[must_use]
    pub const fn radius(mut self, radii: CornerRadii) -> Self {
        self.paint.quad.radii = radii;
        self
    }

    #[must_use]
    pub const fn paint_opacity(mut self, opacity: f32) -> Self {
        self.paint.quad.opacity = opacity;
        self
    }

    #[must_use]
    pub const fn clip(mut self, clip: ClipBehavior) -> Self {
        self.paint.clip = clip;
        self
    }

    #[must_use]
    pub const fn interaction(mut self, interaction: Interaction) -> Self {
        self.interaction = Some(interaction);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTree {
    root: Element,
    node_ids: Vec<NodeId>,
    next_node_id: u64,
    interaction: InteractionState,
    revision: u64,
    layout_dirty: bool,
}

impl UiTree {
    #[must_use]
    pub fn new(root: Element) -> Self {
        let mut next_node_id = 1;
        let node_ids = identity::initial_ids(&root, &mut next_node_id);
        Self {
            root,
            node_ids,
            next_node_id,
            interaction: InteractionState::default(),
            revision: 0,
            layout_dirty: true,
        }
    }

    #[must_use]
    pub const fn root(&self) -> &Element {
        &self.root
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub const fn layout_dirty(&self) -> bool {
        self.layout_dirty
    }

    pub fn replace(&mut self, root: Element) -> bool {
        self.update(root) != TreeUpdate::None
    }

    pub fn update(&mut self, root: Element) -> TreeUpdate {
        let update = classify_update(&self.root, &root);
        match update {
            TreeUpdate::None => return update,
            TreeUpdate::Paint => self.root = root,
            TreeUpdate::Layout => {
                self.node_ids = identity::reconcile_ids(
                    &self.root,
                    &self.node_ids,
                    &root,
                    &mut self.next_node_id,
                );
                self.root = root;
                self.interaction.retain(&self.node_ids);
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
    pub fn visual_state(&self, node: NodeId) -> VisualState {
        self.interaction.visual_state(node)
    }

    #[must_use]
    pub fn resolved_quad(&self, node: NodeId, element: &Element) -> QuadStyle {
        element
            .interaction
            .map_or(element.paint.quad, |interaction| {
                interaction.resolve(element.paint.quad, self.visual_state(node))
            })
    }

    pub fn pointer_moved(&mut self, point: Point, regions: &[HitRegion]) -> InteractionUpdate {
        let update = self.interaction.pointer_moved(point, regions);
        self.decorate(update)
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
        self.decorate(update)
    }

    fn decorate(&self, raw: RawUpdate) -> InteractionUpdate {
        let events = raw.events[..raw.count]
            .iter()
            .flatten()
            .map(|(target, kind)| UiEvent {
                target: *target,
                key: self.key_for(*target).map(ToOwned::to_owned),
                kind: *kind,
            })
            .collect();
        InteractionUpdate {
            events,
            paint_changed: raw.paint_changed,
        }
    }

    fn key_for(&self, node: NodeId) -> Option<&str> {
        let index = self
            .node_ids
            .iter()
            .position(|candidate| *candidate == node)?;
        nth_element(&self.root, index)?.key.as_deref()
    }
}

fn nth_element(root: &Element, target: usize) -> Option<&Element> {
    fn visit<'a>(element: &'a Element, target: usize, cursor: &mut usize) -> Option<&'a Element> {
        if *cursor == target {
            return Some(element);
        }
        *cursor += 1;
        element
            .children
            .iter()
            .find_map(|child| visit(child, target, cursor))
    }
    visit(root, target, &mut 0)
}

fn classify_update(old: &Element, new: &Element) -> TreeUpdate {
    if old == new {
        return TreeUpdate::None;
    }
    if old.key != new.key
        || old.kind != new.kind
        || old.style != new.style
        || old.paint.clip != new.paint.clip
        || interaction_geometry(old.interaction) != interaction_geometry(new.interaction)
        || old.children.len() != new.children.len()
    {
        return TreeUpdate::Layout;
    }
    if old
        .children
        .iter()
        .zip(&new.children)
        .any(|(old, new)| classify_update(old, new) == TreeUpdate::Layout)
    {
        TreeUpdate::Layout
    } else {
        TreeUpdate::Paint
    }
}

fn interaction_geometry(interaction: Option<Interaction>) -> Option<(bool, bool)> {
    interaction.map(|interaction| (interaction.enabled, interaction.focusable))
}
