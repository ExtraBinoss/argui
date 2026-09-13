use argui_paint::{Border, Fill};

use crate::{Element, PropertyBinding, PropertyKey};

impl Element {
    /// Replace the background in every visual state, removing its motion bindings.
    /// Other properties keep their conditional styles and transitions.
    pub fn override_background(&mut self, background: Option<Fill>) {
        self.paint.quad.background = background;
        self.remove_color_states(&[PropertyKey::Background, PropertyKey::BackgroundColor]);
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
        self.remove_color_states(&[
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

    fn remove_color_states(&mut self, properties: &[PropertyKey]) {
        self.conditional_styles.remove(properties);
        if let Some(transition) = &mut self.style_transition {
            transition.make_immediate(properties);
        }
    }
}
