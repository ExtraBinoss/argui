use argui_accessibility::Semantics;
use argui_core::{Transform2D, TransformOrigin};
use argui_paint::{
    Border, ClipBehavior, Color, CornerRadii, Fill, ImageFit, ImageId, ImageSampling, LayerStyle,
    PaintStyle, VectorId,
};
use argui_text::TextStyle;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{
    Align, Direction, Edges, EffectScope, Inset, Interaction, Justify, LayoutStyle, Length,
    MotionProperty, Position, PropertyBinding, ScopedEffect, ScrollConfig, Wrap,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ElementKind {
    Container,
    Text {
        content: String,
        style: TextStyle,
    },
    TextEditor {
        value: String,
        placeholder: String,
        multiline: bool,
        text: TextStyle,
        placeholder_text: TextStyle,
        selection: Color,
        caret: Color,
    },
    Image {
        image: ImageId,
        fit: ImageFit,
        sampling: ImageSampling,
    },
    Vector {
        vector: VectorId,
        progress: f32,
    },
}

#[derive(Clone, Debug)]
pub struct Element(Arc<ElementNode>);

#[derive(Clone, Debug, PartialEq)]
pub struct ElementNode {
    pub inspectable: bool,
    pub key: Option<String>,
    pub kind: ElementKind,
    pub style: LayoutStyle,
    pub paint: PaintStyle,
    pub transform: Transform2D,
    pub transform_origin: TransformOrigin,
    pub interaction: Option<Interaction>,
    pub semantics: Option<Semantics>,
    pub semantic_hidden: bool,
    pub bindings: Vec<PropertyBinding>,
    pub layer: Option<LayerStyle>,
    pub effects: Vec<ScopedEffect>,
    pub scroll: Option<ScrollConfig>,
    pub overlay: Option<crate::OverlayAnchor>,
    pub focus_scope: Option<crate::FocusScope>,
    pub z_index: i32,
    pub children: Vec<Element>,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

impl Deref for Element {
    type Target = ElementNode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Element {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}

impl Element {
    #[must_use]
    pub fn container(children: impl IntoIterator<Item = Self>) -> Self {
        Self(Arc::new(ElementNode {
            inspectable: true,
            key: None,
            kind: ElementKind::Container,
            style: LayoutStyle::default(),
            paint: PaintStyle::default(),
            transform: Transform2D::IDENTITY,
            transform_origin: TransformOrigin::CENTER,
            interaction: None,
            semantics: None,
            semantic_hidden: false,
            bindings: Vec::new(),
            layer: None,
            effects: Vec::new(),
            scroll: None,
            overlay: None,
            focus_scope: None,
            z_index: 0,
            children: children.into_iter().collect(),
        }))
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
        Self(Arc::new(ElementNode {
            inspectable: true,
            key: None,
            kind: ElementKind::Text {
                content: value.into(),
                style: TextStyle::default(),
            },
            style: LayoutStyle::default(),
            paint: PaintStyle::default(),
            transform: Transform2D::IDENTITY,
            transform_origin: TransformOrigin::CENTER,
            interaction: None,
            semantics: None,
            semantic_hidden: false,
            bindings: Vec::new(),
            layer: None,
            effects: Vec::new(),
            scroll: None,
            overlay: None,
            focus_scope: None,
            z_index: 0,
            children: Vec::new(),
        }))
    }

    /// Returns true when both values share the same retained subtree.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    #[must_use]
    pub fn image(image: ImageId) -> Self {
        let mut element = Self::container([]);
        element.kind = ElementKind::Image {
            image,
            fit: ImageFit::default(),
            sampling: ImageSampling::default(),
        };
        element
    }

    #[must_use]
    pub fn vector(vector: VectorId) -> Self {
        let mut element = Self::container([]);
        element.kind = ElementKind::Vector {
            vector,
            progress: 0.0,
        };
        element
    }

    #[must_use]
    pub fn vector_progress(mut self, progress: f32) -> Self {
        if let ElementKind::Vector {
            progress: value, ..
        } = &mut self.kind
        {
            *value = progress;
        }
        self
    }

    #[must_use]
    pub fn image_fit(mut self, fit: ImageFit) -> Self {
        if let ElementKind::Image { fit: value, .. } = &mut self.kind {
            *value = fit;
        }
        self
    }

    #[must_use]
    pub fn image_sampling(mut self, sampling: ImageSampling) -> Self {
        if let ElementKind::Image {
            sampling: value, ..
        } = &mut self.kind
        {
            *value = sampling;
        }
        self
    }

    #[must_use]
    pub fn transform(mut self, transform: Transform2D) -> Self {
        self.transform = transform;
        self
    }

    #[must_use]
    pub fn transform_origin(mut self, origin: TransformOrigin) -> Self {
        self.transform_origin = origin;
        self
    }

    #[must_use]
    pub fn keyed(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    #[must_use]
    pub fn inspectable(mut self, inspectable: bool) -> Self {
        self.inspectable = inspectable;
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
    pub fn paint_style(mut self, style: PaintStyle) -> Self {
        self.paint = style;
        self
    }

    #[must_use]
    pub fn fill(mut self, fill: Fill) -> Self {
        self.paint.quad.background = Some(fill);
        self
    }

    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.style.width = width;
        self
    }

    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.style.height = height;
        self
    }

    #[must_use]
    pub fn min_width(mut self, width: Length) -> Self {
        self.style.min_width = width;
        self
    }

    #[must_use]
    pub fn min_height(mut self, height: Length) -> Self {
        self.style.min_height = height;
        self
    }

    #[must_use]
    pub fn max_width(mut self, width: Length) -> Self {
        self.style.max_width = width;
        self
    }

    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.style.direction = direction;
        self
    }

    #[must_use]
    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.style.wrap = wrap;
        self
    }

    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        self.style.align = align;
        self
    }

    #[must_use]
    pub fn justify(mut self, justify: Justify) -> Self {
        self.style.justify = justify;
        self
    }

    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.style.position = position;
        self
    }

    #[must_use]
    pub fn absolute(mut self, inset: Inset) -> Self {
        self.style.position = Position::Absolute;
        self.style.inset = inset;
        self
    }

    #[must_use]
    pub fn anchored_to(
        mut self,
        key: impl Into<String>,
        placement: crate::OverlayPlacement,
    ) -> Self {
        self.style.position = Position::Absolute;
        self.style.inset = Inset::default();
        self.overlay = Some(crate::OverlayAnchor::new(key, placement));
        self
    }

    #[must_use]
    pub fn focus_scope(mut self, scope: crate::FocusScope) -> Self {
        self.focus_scope = Some(scope);
        self
    }

    #[must_use]
    pub fn padding(mut self, padding: Edges) -> Self {
        self.style.padding = padding;
        self
    }

    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = gap;
        self
    }

    #[must_use]
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.grow = grow;
        self
    }

    #[must_use]
    pub fn shrink(mut self, shrink: f32) -> Self {
        self.style.shrink = shrink;
        self
    }

    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self = self.fill(Fill::Solid(color));
        self
    }

    #[must_use]
    pub fn border(mut self, border: Border) -> Self {
        self.paint.quad.border = Some(border);
        self
    }

    #[must_use]
    pub fn radius(mut self, radii: CornerRadii) -> Self {
        self.paint.quad.radii = radii;
        self
    }

    #[must_use]
    pub fn paint_opacity(mut self, opacity: f32) -> Self {
        self.paint.quad.opacity = opacity;
        self
    }

    #[must_use]
    pub fn clip(mut self, clip: ClipBehavior) -> Self {
        self.paint.clip = clip;
        self
    }

    #[must_use]
    pub fn interaction(mut self, interaction: Interaction) -> Self {
        self.interaction = Some(interaction);
        self
    }

    #[must_use]
    pub fn semantics(mut self, semantics: Semantics) -> Self {
        self.semantics = Some(semantics);
        self
    }

    #[must_use]
    pub fn semantic_hidden(mut self, hidden: bool) -> Self {
        self.semantic_hidden = hidden;
        self
    }

    #[must_use]
    pub fn bind<P>(
        mut self,
        property: P,
        binding: impl Into<argui_animation::MotionBinding<P::Value>>,
    ) -> Self
    where
        P: MotionProperty,
    {
        self.bindings.push(property.into_binding(binding.into()));
        crate::binding::sort_bindings(&mut self.bindings);
        self
    }

    #[must_use]
    pub fn layer(mut self, style: LayerStyle) -> Self {
        self.layer = Some(style);
        self
    }

    #[must_use]
    pub fn effect(mut self, scope: EffectScope, layer: LayerStyle) -> Self {
        self.effects.push(ScopedEffect::new(scope, layer));
        self
    }

    #[must_use]
    pub fn background_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::Background, layer)
    }

    #[must_use]
    pub fn whole_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::WholeElement, layer)
    }

    #[must_use]
    pub fn border_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::Border, layer)
    }

    #[must_use]
    pub fn content_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::Content, layer)
    }

    #[must_use]
    pub fn text_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::Text, layer)
    }

    #[must_use]
    pub fn scrollable(mut self, config: ScrollConfig) -> Self {
        self.scroll = Some(config);
        self.paint.clip = ClipBehavior::Bounds;
        self
    }

    #[must_use]
    pub fn z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}
