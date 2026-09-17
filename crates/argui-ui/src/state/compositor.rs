use argui_core::Transform2D;

use crate::{Element, PropertyBinding, PropertyKey};

impl Element {
    /// Returns whether state or conditional styles have an active animation.
    #[must_use]
    pub fn has_state_animation(&self) -> bool {
        self.style_transition.is_some() || !self.conditional_styles.is_empty()
    }

    #[doc(hidden)]
    /// Returns whether this element needs a stable retained compositor boundary.
    #[must_use]
    pub fn needs_compositor_layer(&self) -> bool {
        self.transform != Transform2D::IDENTITY
            || self
                .layer
                .as_ref()
                .is_some_and(|layer| layer.opacity != 1.0)
            || self.bindings.iter().any(|binding| match binding {
                PropertyBinding::Transform(_) => true,
                PropertyBinding::LayerOpacity(_) => self.layer.is_some(),
                _ => false,
            })
            || self.conditional_styles.contains(PropertyKey::Transform)
            || (self.layer.is_some() && self.conditional_styles.contains(PropertyKey::LayerOpacity))
    }
}
