//! Opt-in effect presets for Argui's generic effect registry.

use argui_render::{EffectDefinition, EffectRegistry, RendererError};

#[cfg(feature = "artistic")]
mod artistic;
#[cfg(feature = "liquid-glass")]
mod liquid_glass;
#[cfg(feature = "liquid-glass")]
pub use liquid_glass::{LIQUID_GLASS_ID, LiquidGlass};
#[cfg(feature = "blur")]
mod blur;
#[cfg(feature = "color")]
mod color;
#[cfg(feature = "refraction")]
mod refraction;
#[cfg(feature = "scroll")]
mod scroll;
#[cfg(feature = "shadow")]
mod shadow;
#[cfg(feature = "scroll")]
pub use scroll::{EDGE_FADE_ID, EDGE_SHADOW_ID, EdgeFade, EdgeShadow};

#[cfg(feature = "artistic")]
pub use artistic::{
    ANIMATED_GRADIENT_ID, AnimatedGradient, WORLEY_BORDER_FIRE_ID, WorleyBorderFire,
};
#[cfg(feature = "blur")]
pub use blur::Blur;
#[cfg(feature = "color")]
pub use color::{Brightness, Contrast, HueRotate, Opacity, Saturation};
#[cfg(feature = "refraction")]
pub use refraction::Refraction;
#[cfg(feature = "shadow")]
pub use shadow::{DropShadow, Glow};

/// Builds a registry containing the effect definitions enabled by crate features.
///
/// # Errors
/// Returns the renderer's registry-construction error if definitions are invalid.
pub fn registry() -> Result<EffectRegistry, RendererError> {
    let definitions: Vec<EffectDefinition> = Vec::new();
    #[cfg(feature = "artistic")]
    let definitions = {
        let mut definitions = definitions;
        definitions.extend(artistic::definitions());
        definitions
    };
    #[cfg(feature = "scroll")]
    let definitions = {
        let mut definitions = definitions;
        definitions.extend(scroll::definitions());
        definitions
    };
    #[cfg(feature = "liquid-glass")]
    let definitions = {
        let mut definitions = definitions;
        definitions.push(liquid_glass::definition());
        definitions
    };
    EffectRegistry::new(definitions)
}
