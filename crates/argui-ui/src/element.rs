use argui_paint::{Border, ClipBehavior, Color, CornerRadii, Fill, PaintStyle};
use argui_text::TextStyle;

use crate::{
    Align, Direction, Edges, Inset, Interaction, Justify, LayoutStyle, Length, Position,
    ScrollConfig, Wrap,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ElementKind {
    Container,
    Text {
        content: String,
        style: TextStyle,
    },
    TextInput {
        initial_value: String,
        placeholder: String,
        text: TextStyle,
        placeholder_text: TextStyle,
        selection: Color,
        caret: Color,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub key: Option<String>,
    pub kind: ElementKind,
    pub style: LayoutStyle,
    pub paint: PaintStyle,
    pub interaction: Option<Interaction>,
    pub scroll: Option<ScrollConfig>,
    pub z_index: i32,
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
            scroll: None,
            z_index: 0,
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
            scroll: None,
            z_index: 0,
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
    pub const fn position(mut self, position: Position) -> Self {
        self.style.position = position;
        self
    }

    #[must_use]
    pub const fn absolute(mut self, inset: Inset) -> Self {
        self.style.position = Position::Absolute;
        self.style.inset = inset;
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

    #[must_use]
    pub const fn scrollable(mut self, config: ScrollConfig) -> Self {
        self.scroll = Some(config);
        self.paint.clip = ClipBehavior::Bounds;
        self
    }

    #[must_use]
    pub const fn z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}
