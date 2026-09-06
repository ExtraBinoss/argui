use crate::{
    AlignContent, AlignItems, AlignSelf, ContainerScopeId, Dimension, Dimensions, Display,
    EffectScope, FlexDirection, FlexWrap, HitTestStyle, Interaction, JustifyContent, JustifyItems,
    JustifySelf, LayoutStyle, LengthPercentage, LengthPercentageAuto, MotionProperty, Position,
    ScopedEffect, ScrollConfig, Sides, StateName, StateScopeId, StyleCondition, StylePatch,
    StyleTransition, WritingDirection,
};
use argui_accessibility::Semantics;
use argui_core::{Transform2D, TransformOrigin};
use argui_paint::{
    Border, Color, CornerRadii, Fill, ImageFit, ImageId, ImageSampling, LayerStyle, PaintStyle,
    VectorId,
};
use argui_text::{TextContent, TextStyle};
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

mod kind;
mod portal;
pub use kind::{ElementKind, ElementNode, TextEditorSpec};

#[derive(Clone, Debug)]
pub struct Element(Rc<ElementNode>);

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0) || self.0 == other.0
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
        Rc::make_mut(&mut self.0)
    }
}

impl Element {
    #[must_use]
    pub fn container(children: impl IntoIterator<Item = Self>) -> Self {
        Self(Rc::new(ElementNode {
            inspectable: true,
            native_content: None,
            key: None,
            kind: ElementKind::Container,
            style: LayoutStyle::default(),
            paint: PaintStyle::default(),
            transform: Transform2D::IDENTITY,
            transform_origin: TransformOrigin::CENTER,
            interaction: None,
            hit_test: HitTestStyle::default(),
            conditional_styles: crate::state::ConditionalStyles::default(),
            style_transition: None,
            state_scope: None,
            container_scope: None,
            active_states: Vec::new(),
            semantics: None,
            semantic_hidden: false,
            bindings: Vec::new(),
            layer: None,
            effects: Vec::new(),
            scroll: None,
            virtual_item: None,
            portal: None,
            focus_scope: None,
            event_listeners: Vec::new(),
            user_select: crate::UserSelect::Auto,
            selection_style: None,
            z_index: 0,
            children: children.into_iter().collect(),
        }))
    }

    #[must_use]
    pub fn row(children: impl IntoIterator<Item = Self>) -> Self {
        let mut element = Self::container(children);
        element.style.display = Display::Flex;
        element.style.flex_direction = FlexDirection::Row;
        element
    }

    #[must_use]
    pub fn column(children: impl IntoIterator<Item = Self>) -> Self {
        let mut element = Self::container(children);
        element.style.display = Display::Flex;
        element.style.flex_direction = FlexDirection::Column;
        element
    }

    #[must_use]
    pub fn grid(children: impl IntoIterator<Item = Self>) -> Self {
        let mut element = Self::container(children);
        element.style.display = Display::Grid;
        element.style.align_items = Some(AlignItems::STRETCH);
        element.style.justify_items = Some(JustifyItems::STRETCH);
        element
    }

    #[must_use]
    pub fn text(value: impl Into<TextContent>) -> Self {
        Self(Rc::new(ElementNode {
            inspectable: true,
            native_content: None,
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
            hit_test: HitTestStyle::default(),
            conditional_styles: crate::state::ConditionalStyles::default(),
            style_transition: None,
            state_scope: None,
            container_scope: None,
            active_states: Vec::new(),
            semantics: None,
            semantic_hidden: false,
            bindings: Vec::new(),
            layer: None,
            effects: Vec::new(),
            scroll: None,
            virtual_item: None,
            portal: None,
            focus_scope: None,
            event_listeners: Vec::new(),
            user_select: crate::UserSelect::Auto,
            selection_style: None,
            z_index: 0,
            children: Vec::new(),
        }))
    }

    #[must_use]
    pub fn text_editor(spec: TextEditorSpec) -> Self {
        let mut element = Self::container([]);
        element.kind = ElementKind::TextEditor {
            value: spec.value,
            placeholder: spec.placeholder,
            multiline: spec.multiline,
            read_only: spec.read_only,
            filter: spec.filter,
            text: spec.text,
            placeholder_text: spec.placeholder_text,
            selection: spec.selection,
            caret: spec.caret,
        };
        element
    }

    /// Returns true when both values share the same retained subtree.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    #[must_use]
    pub fn virtual_item(&self) -> Option<&crate::VirtualItem> {
        self.virtual_item.as_ref()
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
            fit: ImageFit::Contain,
            color: Color::WHITE,
        };
        element
    }
    #[must_use]
    pub fn vector_fit(mut self, fit: ImageFit) -> Self {
        if let ElementKind::Vector { fit: value, .. } = &mut self.kind {
            *value = fit;
        }
        self
    }
    #[must_use]
    pub fn vector_color(mut self, color: Color) -> Self {
        if let ElementKind::Vector { color: value, .. } = &mut self.kind {
            *value = color;
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
    pub fn width(mut self, width: Dimension) -> Self {
        self.style.size.width = width;
        self
    }

    #[must_use]
    pub fn height(mut self, height: Dimension) -> Self {
        self.style.size.height = height;
        self
    }

    #[must_use]
    pub fn min_width(mut self, width: LengthPercentageAuto) -> Self {
        self.style.min_size.width = width;
        self
    }

    #[must_use]
    pub fn min_height(mut self, height: LengthPercentageAuto) -> Self {
        self.style.min_size.height = height;
        self
    }

    #[must_use]
    pub fn max_height(mut self, height: LengthPercentageAuto) -> Self {
        self.style.max_size.height = height;
        self
    }

    #[must_use]
    pub fn max_width(mut self, width: LengthPercentageAuto) -> Self {
        self.style.max_size.width = width;
        self
    }

    #[must_use]
    pub fn display(mut self, display: Display) -> Self {
        self.style.display = display;
        self
    }

    #[must_use]
    pub fn writing_direction(mut self, direction: WritingDirection) -> Self {
        self.style.writing_direction = direction;
        self
    }

    #[must_use]
    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.style.flex_direction = direction;
        self
    }

    #[must_use]
    pub fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.style.flex_wrap = wrap;
        self
    }

    #[must_use]
    pub fn align_items(mut self, alignment: AlignItems) -> Self {
        self.style.align_items = Some(alignment);
        self
    }

    #[must_use]
    pub fn align_self(mut self, alignment: AlignSelf) -> Self {
        self.style.align_self = Some(alignment);
        self
    }

    #[must_use]
    pub fn justify_items(mut self, alignment: JustifyItems) -> Self {
        self.style.justify_items = Some(alignment);
        self
    }

    #[must_use]
    pub fn justify_self(mut self, alignment: JustifySelf) -> Self {
        self.style.justify_self = Some(alignment);
        self
    }

    #[must_use]
    pub fn align_content(mut self, alignment: AlignContent) -> Self {
        self.style.align_content = Some(alignment);
        self
    }

    #[must_use]
    pub fn justify_content(mut self, alignment: JustifyContent) -> Self {
        self.style.justify_content = Some(alignment);
        self
    }

    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.style.position = position;
        self
    }

    #[must_use]
    pub fn absolute(mut self, inset: Sides<LengthPercentageAuto>) -> Self {
        self.style.position = Position::Absolute;
        self.style.inset = inset;
        self
    }

    #[must_use]
    pub fn focus_scope(mut self, scope: crate::FocusScope) -> Self {
        self.focus_scope = Some(scope);
        self
    }

    #[must_use]
    pub fn margin(mut self, margin: Sides<LengthPercentageAuto>) -> Self {
        self.style.margin = margin;
        self
    }

    #[must_use]
    pub fn padding(mut self, padding: Sides<LengthPercentage>) -> Self {
        self.style.padding = padding;
        self
    }

    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        let gap = LengthPercentage::length(gap);
        self.style.gap = Dimensions {
            width: gap,
            height: gap,
        };
        self
    }

    #[must_use]
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.style.gap.height = LengthPercentage::length(gap);
        self
    }

    #[must_use]
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.style.gap.width = LengthPercentage::length(gap);
        self
    }

    #[must_use]
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    #[must_use]
    pub fn shrink(mut self, shrink: f32) -> Self {
        self.style.flex_shrink = shrink;
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
    pub fn interaction(mut self, interaction: Interaction) -> Self {
        self.interaction = Some(interaction);
        self
    }

    #[must_use]
    pub fn hit_test(mut self, hit_test: HitTestStyle) -> Self {
        self.hit_test = hit_test;
        self
    }

    #[must_use]
    pub fn when(mut self, condition: impl Into<StyleCondition>, style: StylePatch) -> Self {
        self.conditional_styles.set(condition.into(), style);
        self
    }

    #[must_use]
    pub fn transition(mut self, transition: StyleTransition) -> Self {
        self.style_transition = Some(transition);
        self
    }

    #[must_use]
    pub fn state_scope(mut self, scope: StateScopeId) -> Self {
        self.state_scope = Some(scope);
        self
    }

    #[must_use]
    pub fn container_scope(mut self, scope: ContainerScopeId) -> Self {
        self.container_scope = Some(scope);
        self
    }

    #[must_use]
    pub fn active_state(mut self, state: StateName, active: bool) -> Self {
        self.active_states.retain(|candidate| *candidate != state);
        if active {
            self.active_states.push(state);
        }
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
    pub fn scroll_config(mut self, config: ScrollConfig) -> Self {
        if let Some(scrollbar) = &config.scrollbar {
            self.style.scrollbar_width = scrollbar.gutter_width();
        }
        self.scroll = Some(config);
        self
    }

    #[must_use]
    pub fn z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}
