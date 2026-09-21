use argui_paint::{EffectId, EffectInstance, EffectValue, Filter};
use argui_render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition};

/// Registry identifier for the Worley border-fire effect.
pub const WORLEY_BORDER_FIRE_ID: EffectId = EffectId::new("argui.artistic.worley-border-fire");
/// Registry identifier for the animated-gradient effect.
pub const ANIMATED_GRADIENT_ID: EffectId = EffectId::new("argui.artistic.animated-gradient");

pub(crate) fn definitions() -> [EffectDefinition; 2] {
    [
        EffectDefinition::new(
            WORLEY_BORDER_FIRE_ID,
            [
                EffectParameter::new("phase", EffectParameterType::F32),
                EffectParameter::new("intensity", EffectParameterType::F32),
                EffectParameter::new("frequency", EffectParameterType::F32),
                EffectParameter::new("expansion", EffectParameterType::LogicalPixels),
            ],
            [EffectPassDefinition::fragment(
                "fire",
                include_str!("shaders/effects/worley_border_fire.wgsl"),
            )],
        ),
        EffectDefinition::new(
            ANIMATED_GRADIENT_ID,
            [
                EffectParameter::new("phase", EffectParameterType::F32),
                EffectParameter::new("frequency", EffectParameterType::F32),
                EffectParameter::new("intensity", EffectParameterType::F32),
            ],
            [EffectPassDefinition::fragment(
                "gradient",
                include_str!("shaders/effects/animated_gradient.wgsl"),
            )],
        ),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimatedGradient {
    /// Animation phase supplied to the shader.
    pub phase: f32,
    /// Spatial frequency of the gradient pattern.
    pub frequency: f32,
    /// Effect intensity.
    pub intensity: f32,
}

impl AnimatedGradient {
    /// Creates an animated gradient at `phase` with default frequency and intensity.
    #[must_use]
    pub const fn new(phase: f32) -> Self {
        Self {
            phase,
            frequency: 1.4,
            intensity: 1.0,
        }
    }

    /// Sets the pattern frequency.
    #[must_use]
    pub const fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    /// Sets the effect intensity.
    #[must_use]
    pub const fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    /// Converts this preset into a paint filter.
    /// Converts this preset into a paint filter.
    /// Creates the configured Worley border-fire paint filter.
    #[must_use]
    pub fn filter(self) -> Filter {
        Filter::Effect(EffectInstance::new(
            ANIMATED_GRADIENT_ID,
            [
                ("phase", EffectValue::F32(self.phase)),
                ("frequency", EffectValue::F32(self.frequency)),
                ("intensity", EffectValue::F32(self.intensity)),
            ],
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorleyBorderFire {
    /// Animation phase supplied to the shader.
    pub phase: f32,
    /// Effect intensity.
    pub intensity: f32,
    /// Spatial frequency of the Worley pattern.
    pub frequency: f32,
    /// Effect expansion in logical pixels.
    pub expansion: f32,
}

impl WorleyBorderFire {
    /// Creates the effect at `phase` with its default intensity, frequency, and expansion.
    #[must_use]
    pub const fn new(phase: f32) -> Self {
        Self {
            phase,
            intensity: 0.92,
            frequency: 13.0,
            expansion: 30.0,
        }
    }

    /// Sets the effect intensity.
    #[must_use]
    pub const fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    /// Sets the Worley pattern frequency.
    #[must_use]
    pub const fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    /// Sets the effect expansion in logical pixels.
    #[must_use]
    pub const fn expansion(mut self, pixels: f32) -> Self {
        self.expansion = pixels;
        self
    }

    /// Creates the configured Worley border-fire paint filter.
    #[must_use]
    pub fn filter(self) -> Filter {
        Filter::Effect(
            EffectInstance::new(
                WORLEY_BORDER_FIRE_ID,
                [
                    ("phase", EffectValue::F32(self.phase)),
                    ("intensity", EffectValue::F32(self.intensity)),
                    ("frequency", EffectValue::F32(self.frequency)),
                    ("expansion", EffectValue::LogicalPixels(self.expansion)),
                ],
            )
            .expansion(self.expansion),
        )
    }
}
