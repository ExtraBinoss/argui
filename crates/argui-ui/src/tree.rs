use argui_paint::{Border, ClipBehavior, Color, CornerRadii, Fill, PaintStyle};
use argui_text::TextStyle;

use crate::{Align, Direction, Edges, Justify, LayoutStyle, Length};

#[derive(Clone, Debug, PartialEq)]
pub enum ElementKind {
    Container,
    Text { content: String, style: TextStyle },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub key: Option<String>,
    pub kind: ElementKind,
    pub style: LayoutStyle,
    pub paint: PaintStyle,
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
        self.paint.background = Some(fill);
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
    pub const fn background(mut self, color: Color) -> Self {
        self = self.fill(Fill::Solid(color));
        self
    }

    #[must_use]
    pub const fn border(mut self, border: Border) -> Self {
        self.paint.border = Some(border);
        self
    }

    #[must_use]
    pub const fn radius(mut self, radii: CornerRadii) -> Self {
        self.paint.radii = radii;
        self
    }

    #[must_use]
    pub const fn paint_opacity(mut self, opacity: f32) -> Self {
        self.paint.opacity = opacity;
        self
    }

    #[must_use]
    pub const fn clip(mut self, clip: ClipBehavior) -> Self {
        self.paint.clip = clip;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTree {
    root: Element,
    revision: u64,
    layout_dirty: bool,
}

impl UiTree {
    #[must_use]
    pub const fn new(root: Element) -> Self {
        Self {
            root,
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
        if self.root == root {
            return false;
        }
        self.root = root;
        self.revision = self.revision.wrapping_add(1);
        self.layout_dirty = true;
        true
    }

    pub fn mark_layout_clean(&mut self) {
        self.layout_dirty = false;
    }
}
