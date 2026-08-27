//! Optional, renderer-backed effects that remain plain `argui-paint` filters.

use argui_paint::{CustomEffect, Filter, ShaderEffectId};
use argui_render::EffectShader;

pub const WORLEY_BORDER_FIRE_ID: ShaderEffectId = ShaderEffectId(0xA6_0001);
pub const LIQUID_GLASS_ID: ShaderEffectId = ShaderEffectId(0xA6_0002);
pub const ANIMATED_GRADIENT_ID: ShaderEffectId = ShaderEffectId(0xA6_0003);

pub const SHADERS: &[EffectShader] = &[
    EffectShader::new(
        WORLEY_BORDER_FIRE_ID,
        include_str!("shaders/effects/worley_border_fire.wgsl"),
    ),
    EffectShader::new(
        LIQUID_GLASS_ID,
        include_str!("shaders/effects/liquid_glass.wgsl"),
    ),
    EffectShader::new(
        ANIMATED_GRADIENT_ID,
        include_str!("shaders/effects/animated_gradient.wgsl"),
    ),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimatedGradient {
    pub phase: f32,
    pub frequency: f32,
    pub intensity: f32,
}

impl AnimatedGradient {
    #[must_use]
    pub const fn new(phase: f32) -> Self {
        Self {
            phase,
            frequency: 1.4,
            intensity: 1.0,
        }
    }

    #[must_use]
    pub const fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    #[must_use]
    pub const fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    #[must_use]
    pub fn filter(self) -> Filter {
        Filter::Custom(CustomEffect::new(
            ANIMATED_GRADIENT_ID,
            [self.phase, self.frequency, self.intensity],
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorleyBorderFire {
    pub phase: f32,
    pub intensity: f32,
    pub frequency: f32,
    pub expansion: f32,
}

impl WorleyBorderFire {
    #[must_use]
    pub const fn new(phase: f32) -> Self {
        Self {
            phase,
            intensity: 0.92,
            frequency: 13.0,
            expansion: 30.0,
        }
    }

    #[must_use]
    pub const fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    #[must_use]
    pub const fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    #[must_use]
    pub const fn expansion(mut self, pixels: f32) -> Self {
        self.expansion = pixels;
        self
    }

    #[must_use]
    pub fn filter(self) -> Filter {
        Filter::Custom(
            CustomEffect::new(
                WORLEY_BORDER_FIRE_ID,
                [self.phase, self.intensity, self.frequency, self.expansion],
            )
            .pixel_parameter(3)
            .expansion(self.expansion),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiquidGlass {
    pub refraction: f32,
    pub chromatic_aberration: f32,
    pub blur: f32,
    pub highlight: f32,
    pub edge_width: f32,
    pub saturation: f32,
}

impl Default for LiquidGlass {
    fn default() -> Self {
        Self {
            refraction: 8.0,
            chromatic_aberration: 1.25,
            blur: 3.0,
            highlight: 0.22,
            edge_width: 18.0,
            saturation: 1.0,
        }
    }
}

impl LiquidGlass {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            refraction: 8.0,
            chromatic_aberration: 1.25,
            blur: 3.0,
            highlight: 0.22,
            edge_width: 18.0,
            saturation: 1.0,
        }
    }

    #[must_use]
    pub const fn refraction(mut self, pixels: f32) -> Self {
        self.refraction = pixels;
        self
    }

    #[must_use]
    pub const fn chromatic_aberration(mut self, pixels: f32) -> Self {
        self.chromatic_aberration = pixels;
        self
    }

    #[must_use]
    pub const fn blur(mut self, pixels: f32) -> Self {
        self.blur = pixels;
        self
    }

    #[must_use]
    pub const fn highlight(mut self, intensity: f32) -> Self {
        self.highlight = intensity;
        self
    }

    #[must_use]
    pub const fn edge_width(mut self, pixels: f32) -> Self {
        self.edge_width = pixels;
        self
    }

    #[must_use]
    pub const fn saturation(mut self, saturation: f32) -> Self {
        self.saturation = saturation;
        self
    }

    #[must_use]
    pub fn filter(self) -> Filter {
        Filter::Custom(
            CustomEffect::new(
                LIQUID_GLASS_ID,
                [
                    self.refraction,
                    self.chromatic_aberration,
                    self.blur,
                    self.highlight,
                    self.edge_width,
                    self.saturation,
                ],
            )
            .pixel_parameter(0)
            .pixel_parameter(1)
            .pixel_parameter(2)
            .pixel_parameter(4),
        )
    }
}

#[cfg(test)]
mod tests {
    use argui_paint::{CustomEffect, Filter};

    use super::{AnimatedGradient, LiquidGlass, SHADERS, WorleyBorderFire};

    fn custom(filter: Filter) -> CustomEffect {
        match filter {
            Filter::Custom(effect) => effect,
            _ => panic!("preset must lower to a custom effect"),
        }
    }

    #[test]
    fn presets_are_registered_and_lower_to_custom_filters() {
        assert_eq!(SHADERS.len(), 3);
        let fire = custom(WorleyBorderFire::new(2.0).filter());
        assert_eq!(fire.parameters, [2.0, 0.92, 13.0, 30.0]);
        assert_eq!(fire.expansion, 30.0);

        let glass = custom(LiquidGlass::new().filter());
        assert_eq!(glass.parameters, [8.0, 1.25, 3.0, 0.22, 18.0, 1.0]);
        let saturated = custom(LiquidGlass::new().saturation(1.35).filter());
        assert_eq!(saturated.parameters[5], 1.35);
        assert_eq!(glass.pixel_parameters, 0b1_0111);

        let gradient = custom(AnimatedGradient::new(1.0).filter());
        assert_eq!(gradient.parameters, [1.0, 1.4, 1.0]);

        for shader in SHADERS {
            shader
                .validate()
                .expect("preset WGSL must match the public ABI");
        }

        let fire_source = SHADERS[0].wgsl;
        assert!(fire_source.contains("signed_distance < -1.0"));
        assert!(fire_source.contains("signed_distance > expansion"));
        assert!(SHADERS[1].wgsl.contains("distance > 1.0"));
    }
}
