use crate::{
    AlignContent, AlignItems, AlignSelf, ContainerScopeId, Dimension, Dimensions, Display,
    EffectScope, FlexDirection, FlexWrap, HitTestStyle, Interaction, JustifyContent, JustifyItems,
    JustifySelf, LayoutStyle, LengthPercentage, LengthPercentageAuto, MotionProperty, Position,
    ScopedEffect, ScrollConfig, Sides, StateName, StateScopeId, StyleCondition, StylePatch,
    StyleTransition,
};
use argui_core::{Transform2D, TransformOrigin};
use argui_paint::{
    Border, Color, CornerRadii, Fill, ImageFit, ImageId, ImageSampling, LayerStyle, PaintStyle,
    VectorId,
};
use argui_text::TextStyle;
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

mod boundary;
mod constructors;
mod direction;
mod kind;
mod overrides;
mod portal;
mod safe_area;
mod virtual_list;
pub use kind::{ElementKind, ElementNode, TextEditorSpec};

#[derive(Clone)]
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
    /// Returns true when both values share the same retained subtree.
    ///
    /// * `other` — element to compare with this value.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Creates an image element for the supplied asset.
    ///
    /// * `image` — image asset identity.
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

    /// Creates a vector element for the supplied asset.
    ///
    /// * `vector` — vector asset identity.
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
    /// Sets how a vector asset is fitted into this element's bounds.
    /// * `fit` — fitting mode for the vector asset.
    #[must_use]
    pub fn vector_fit(mut self, fit: ImageFit) -> Self {
        if let ElementKind::Vector { fit: value, .. } = &mut self.kind {
            *value = fit;
        }
        self
    }
    /// Sets the tint used when painting a vector asset.
    /// * `color` — tint color applied to the vector asset.
    #[must_use]
    pub fn vector_color(mut self, color: Color) -> Self {
        if let ElementKind::Vector { color: value, .. } = &mut self.kind {
            *value = color;
        }
        self
    }

    /// Sets how an image asset is fitted into this element's bounds.
    /// * `fit` — fitting mode for the image asset.
    #[must_use]
    pub fn image_fit(mut self, fit: ImageFit) -> Self {
        if let ElementKind::Image { fit: value, .. } = &mut self.kind {
            *value = fit;
        }
        self
    }

    /// Sets the sampling mode used when painting an image asset.
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

    /// Sets the element's 2D transform.
    #[must_use]
    pub fn transform(mut self, transform: Transform2D) -> Self {
        self.transform = transform;
        self
    }

    /// Sets the origin about which the element's transform is applied.
    #[must_use]
    pub fn transform_origin(mut self, origin: TransformOrigin) -> Self {
        self.transform_origin = origin;
        self
    }

    /// Assigns a stable key used to preserve identity across tree updates.
    #[must_use]
    pub fn keyed(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Assigns an owner-local identity used by retained reconciliation.
    ///
    /// Application code should normally use [`Self::keyed`]. Hosts may use this
    /// channel to preserve native state when a node moves between parents.
    ///
    /// * `identity` — stable owner and node identity.
    #[doc(hidden)]
    #[must_use]
    pub fn retained_identity(mut self, identity: crate::RetainedIdentity) -> Self {
        self.retained_identity = Some(identity);
        self
    }

    /// Returns the retained producer identity, if this element has one.
    ///
    /// Hosts use this identity to associate layout measurements with the
    /// correct node without changing public element keys.
    #[doc(hidden)]
    #[must_use]
    pub fn source_identity(&self) -> Option<&crate::RetainedIdentity> {
        self.retained_identity.as_ref()
    }

    /// Sets the descriptive tooltip text associated with this element.
    /// * `description` — tooltip text associated with the element.
    #[must_use]
    pub fn tooltip(mut self, description: impl Into<String>) -> Self {
        self.tooltip = Some(description.into());
        self
    }

    /// Sets whether this element is exposed to UI inspection.
    /// * `inspectable` — whether the element appears in inspection data.
    #[must_use]
    pub fn inspectable(mut self, inspectable: bool) -> Self {
        self.inspectable = inspectable;
        self
    }

    /// Sets the text style when this element contains text.
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

    /// Replaces the element's complete layout style.
    #[must_use]
    pub fn layout_style(mut self, style: LayoutStyle) -> Self {
        self.style = style;
        self
    }

    /// Replaces the element's complete paint style.
    #[must_use]
    pub fn paint_style(mut self, style: PaintStyle) -> Self {
        self.paint = style;
        self
    }

    /// Sets the element's background fill.
    #[must_use]
    pub fn fill(mut self, fill: Fill) -> Self {
        self.paint.quad.background = Some(fill);
        self
    }

    /// Sets the preferred width.
    #[must_use]
    pub fn width(mut self, width: Dimension) -> Self {
        self.style.size.width = width;
        self
    }

    /// Sets the preferred height.
    #[must_use]
    pub fn height(mut self, height: Dimension) -> Self {
        self.style.size.height = height;
        self
    }

    /// Sets the minimum width.
    #[must_use]
    pub fn min_width(mut self, width: LengthPercentageAuto) -> Self {
        self.style.min_size.width = width;
        self
    }

    /// Sets the minimum height.
    #[must_use]
    pub fn min_height(mut self, height: LengthPercentageAuto) -> Self {
        self.style.min_size.height = height;
        self
    }

    /// Sets the maximum height.
    #[must_use]
    pub fn max_height(mut self, height: LengthPercentageAuto) -> Self {
        self.style.max_size.height = height;
        self
    }

    /// Sets the maximum width.
    #[must_use]
    pub fn max_width(mut self, width: LengthPercentageAuto) -> Self {
        self.style.max_size.width = width;
        self
    }

    /// Sets the layout display mode.
    #[must_use]
    pub fn display(mut self, display: Display) -> Self {
        self.style.display = display;
        self
    }

    /// Sets the main-axis direction for flex layout.
    #[must_use]
    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.style.flex_direction = direction;
        self
    }

    /// Sets whether flex items wrap onto additional lines.
    #[must_use]
    pub fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.style.flex_wrap = wrap;
        self
    }

    /// Sets cross-axis alignment for this container's items.
    #[must_use]
    pub fn align_items(mut self, alignment: AlignItems) -> Self {
        self.style.align_items = Some(alignment);
        self
    }

    /// Sets this flex item's cross-axis alignment in its parent.
    #[must_use]
    pub fn align_self(mut self, alignment: AlignSelf) -> Self {
        self.style.align_self = Some(alignment);
        self
    }

    /// Sets alignment of grid items in their cells.
    #[must_use]
    pub fn justify_items(mut self, alignment: JustifyItems) -> Self {
        self.style.justify_items = Some(alignment);
        self
    }

    /// Sets alignment of this grid item in its cell.
    #[must_use]
    pub fn justify_self(mut self, alignment: JustifySelf) -> Self {
        self.style.justify_self = Some(alignment);
        self
    }

    /// Sets how flex or grid content is distributed across the container.
    /// * `alignment` — distribution mode for the container's content.
    #[must_use]
    pub fn align_content(mut self, alignment: AlignContent) -> Self {
        self.style.align_content = Some(alignment);
        self
    }

    /// Sets how items are distributed along the main axis.
    /// * `alignment` — distribution mode along the main axis.
    #[must_use]
    pub fn justify_content(mut self, alignment: JustifyContent) -> Self {
        self.style.justify_content = Some(alignment);
        self
    }

    /// Sets the element's positioning mode.
    /// * `position` — positioning mode for the element.
    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.style.position = position;
        self
    }

    /// Positions the element absolutely with the supplied per-side inset.
    #[must_use]
    pub fn absolute(mut self, inset: Sides<LengthPercentageAuto>) -> Self {
        self.style.position = Position::Absolute;
        self.style.inset = inset;
        self
    }

    /// Sets the focus scope established by this element.
    #[must_use]
    pub fn focus_scope(mut self, scope: crate::FocusScope) -> Self {
        self.focus_scope = Some(scope);
        self
    }

    /// Sets the element's margins.
    /// * `margin` — per-side outer spacing.
    #[must_use]
    pub fn margin(mut self, margin: Sides<LengthPercentageAuto>) -> Self {
        self.style.margin = margin;
        self
    }

    /// Sets the element's padding.
    #[must_use]
    pub fn padding(mut self, padding: Sides<LengthPercentage>) -> Self {
        self.style.padding = padding;
        self
    }

    /// Sets equal row and column gaps in logical pixels.
    /// * `gap` — spacing between rows and columns.
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        let gap = LengthPercentage::length(gap);
        self.style.gap = Dimensions {
            width: gap,
            height: gap,
        };
        self
    }

    /// Sets the gap between flex rows or grid rows.
    #[must_use]
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.style.gap.height = LengthPercentage::length(gap);
        self
    }

    /// Sets the gap between flex columns or grid columns.
    #[must_use]
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.style.gap.width = LengthPercentage::length(gap);
        self
    }

    /// Sets the flex growth factor.
    /// * `grow` — flex growth factor.
    #[must_use]
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    /// Sets the flex shrink factor.
    #[must_use]
    pub fn shrink(mut self, shrink: f32) -> Self {
        self.style.flex_shrink = shrink;
        self
    }

    /// Sets a solid background color.
    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self = self.fill(Fill::Solid(color));
        self
    }

    /// Sets the element's border.
    #[must_use]
    pub fn border(mut self, border: Border) -> Self {
        self.paint.quad.border = Some(border);
        self
    }

    /// Sets corner radii and removes conflicting animated or conditional radii.
    #[must_use]
    pub fn radius(mut self, radii: CornerRadii) -> Self {
        self.override_radii(radii);
        self
    }

    /// Sets paint opacity for the element's quad.
    #[must_use]
    pub fn paint_opacity(mut self, opacity: f32) -> Self {
        self.paint.quad.opacity = opacity;
        self
    }

    /// Sets pointer, focus, keyboard, and gesture interaction behavior.
    #[must_use]
    pub fn interaction(mut self, interaction: Interaction) -> Self {
        self.interaction = Some(interaction);
        self
    }

    /// Sets the pointer hit-testing style.
    /// * `hit_test` — hit-testing behavior for the element.
    #[must_use]
    pub fn hit_test(mut self, hit_test: HitTestStyle) -> Self {
        self.hit_test = hit_test;
        self
    }

    /// Applies a conditional style when the supplied condition matches.
    ///
    /// * `condition` — state or container condition that activates the style.
    /// * `style` — properties to apply while the condition is active.
    #[must_use]
    pub fn when(mut self, condition: impl Into<StyleCondition>, style: StylePatch) -> Self {
        self.conditional_styles.set(condition.into(), style);
        self
    }

    /// Sets transitions for changes to conditional styles.
    /// * `transition` — transition definitions to apply.
    #[must_use]
    pub fn transition(mut self, transition: StyleTransition) -> Self {
        self.style_transition = Some(transition);
        self
    }

    /// Sets the identity scope used for state selectors.
    #[must_use]
    pub fn state_scope(mut self, scope: StateScopeId) -> Self {
        self.state_scope = Some(scope);
        self
    }

    /// Sets the container-query scope established by this element.
    #[must_use]
    pub fn container_scope(mut self, scope: ContainerScopeId) -> Self {
        self.container_scope = Some(scope);
        self
    }

    /// Adds or removes an explicitly active visual state.
    ///
    /// * `state` — state name to update.
    /// * `active` — whether the state should be active.
    #[must_use]
    pub fn active_state(mut self, state: StateName, active: bool) -> Self {
        self.active_states.retain(|candidate| *candidate != state);
        if active {
            self.active_states.push(state);
        }
        self
    }

    /// Binds an animated property to this element.
    ///
    /// * `property` — property selector to animate.
    /// * `binding` — motion and composition that produce its values.
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

    /// Sets the element's compositing layer style.
    #[must_use]
    pub fn layer(mut self, style: LayerStyle) -> Self {
        self.layer = Some(Box::new(style));
        self
    }

    /// Adds a layer effect to the selected element region.
    ///
    /// * `scope` — region of the element affected by the effect.
    /// * `layer` — effect layer configuration.
    #[must_use]
    pub fn effect(mut self, scope: EffectScope, layer: LayerStyle) -> Self {
        self.effects.push(ScopedEffect::new(scope, layer));
        self
    }

    /// Adds an effect to the element background.
    /// * `layer` — effect layer configuration.
    #[must_use]
    pub fn background_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::Background, layer)
    }

    /// Adds an effect to the whole element.
    /// * `layer` — effect layer configuration.
    #[must_use]
    pub fn whole_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::WholeElement, layer)
    }

    /// Adds an effect to the element border.
    /// * `layer` — effect layer configuration.
    #[must_use]
    pub fn border_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::Border, layer)
    }

    /// Adds an effect to the element content.
    /// * `layer` — effect layer configuration.
    #[must_use]
    pub fn content_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::Content, layer)
    }

    /// Adds an effect to text painted by the element.
    /// * `layer` — effect layer configuration.
    #[must_use]
    pub fn text_effect(self, layer: LayerStyle) -> Self {
        self.effect(EffectScope::Text, layer)
    }

    /// Sets scrolling behavior and synchronizes the layout scrollbar width.
    /// * `config` — scrolling behavior and scrollbar configuration.
    #[must_use]
    pub fn scroll_config(mut self, config: ScrollConfig) -> Self {
        if let Some(scrollbar) = &config.scrollbar {
            self.style.scrollbar_width = scrollbar.gutter_width();
        }
        self.scroll = Some(Box::new(config));
        self
    }

    /// Requests a scroll offset when the declared value changes and returns the element.
    ///
    /// * `offset` — horizontal and vertical content offset in logical pixels.
    ///
    /// User scrolling remains active between changes to this value.
    #[must_use]
    pub fn scroll_offset(mut self, offset: argui_core::Point) -> Self {
        self.declared_scroll_offset = Some(offset);
        self
    }
    /// Sets the element's stacking order within its window layer.
    /// * `z_index` — stacking order within the window layer.
    #[must_use]
    pub fn z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}
