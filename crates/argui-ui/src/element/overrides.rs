use argui_paint::{Border, CornerRadii, Fill};

use crate::{Element, PropertyBinding, PropertyKey};

impl Element {
    pub(super) fn override_radii(&mut self, radii: CornerRadii) {
        self.paint.quad.radii = radii;
        self.remove_overridden_states(&[PropertyKey::CornerRadii]);
        self.bindings
            .retain(|binding| !matches!(binding, PropertyBinding::CornerRadii(_)));
    }

    /// Replace paint opacity in every visual state, removing its motion binding.
    pub fn override_paint_opacity(&mut self, opacity: f32) {
        self.paint.quad.opacity = opacity.clamp(0.0, 1.0);
        self.remove_overridden_states(&[PropertyKey::Opacity]);
        self.bindings
            .retain(|binding| !matches!(binding, PropertyBinding::Opacity(_)));
    }

    /// Replace the background in every visual state, removing its motion bindings.
    /// Other properties keep their conditional styles and transitions.
    pub fn override_background(&mut self, background: Option<Fill>) {
        self.paint.quad.background = background;
        self.remove_overridden_states(&[PropertyKey::Background, PropertyKey::BackgroundColor]);
        self.bindings.retain(|binding| {
            !matches!(
                binding,
                PropertyBinding::BackgroundColor(_)
                    | PropertyBinding::GradientPoint(..)
                    | PropertyBinding::GradientStopOffset(..)
                    | PropertyBinding::GradientStopColor(..)
            )
        });
    }

    /// Replace the border in every visual state, removing its motion bindings.
    pub fn override_border(&mut self, border: Option<Border>) {
        self.paint.quad.border = border;
        self.remove_overridden_states(&[
            PropertyKey::Border,
            PropertyKey::BorderColor,
            PropertyKey::BorderWidths,
        ]);
        self.bindings.retain(|binding| {
            !matches!(
                binding,
                PropertyBinding::BorderColor(_) | PropertyBinding::BorderWidths(_)
            )
        });
    }

    fn remove_overridden_states(&mut self, properties: &[PropertyKey]) {
        self.conditional_styles.remove(properties);
        if let Some(transition) = &mut self.style_transition {
            transition.make_immediate(properties);
        }
    }
}
