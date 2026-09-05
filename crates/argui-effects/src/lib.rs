//! Opt-in effect presets for Argui's generic effect registry.

use argui_render::{EffectDefinition, EffectRegistry, RendererError};

#[cfg(feature = "artistic")]
mod artistic;
#[cfg(feature = "blur")]
mod blur;
#[cfg(feature = "color")]
mod color;
#[cfg(feature = "refraction")]
mod refraction;
#[cfg(feature = "shadow")]
mod shadow;

#[cfg(feature = "artistic")]
pub use artistic::{
    ANIMATED_GRADIENT_ID, AnimatedGradient, LIQUID_GLASS_ID, LiquidGlass, WORLEY_BORDER_FIRE_ID,
    WorleyBorderFire,
};
#[cfg(feature = "blur")]
pub use blur::Blur;
#[cfg(feature = "color")]
pub use color::{Brightness, Contrast, HueRotate, Opacity, Saturation};
#[cfg(feature = "refraction")]
pub use refraction::Refraction;
#[cfg(feature = "shadow")]
pub use shadow::{DropShadow, Glow};

pub fn registry() -> Result<EffectRegistry, RendererError> {
    let definitions: Vec<EffectDefinition> = Vec::new();
    #[cfg(feature = "artistic")]
    let definitions = {
        let mut definitions = definitions;
        definitions.extend(artistic::definitions());
        definitions
    };
    EffectRegistry::new(definitions)
}
