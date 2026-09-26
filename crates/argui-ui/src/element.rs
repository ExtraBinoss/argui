use crate::{
    ContainerScopeId, EffectScope, HitTestStyle, Interaction, LayoutStyle, MotionProperty,
    ScopedEffect, ScrollConfig, StateName, StateScopeId, StyleCondition, StylePatch,
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
mod layout;
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
