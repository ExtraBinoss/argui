use argui_paint::{EffectId, EffectInstance, EffectValue, Filter};
use argui_render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition};

pub const WORLEY_BORDER_FIRE_ID: EffectId = EffectId::new("argui.artistic.worley-border-fire");
pub const ANIMATED_GRADIENT_ID: EffectId = EffectId::new("argui.artistic.animated-gradient");

const GRADIENT_PARAMETERS: &[EffectParameter] = &[
    EffectParameter::new("phase", EffectParameterType::F32),
    EffectParameter::new("frequency", EffectParameterType::F32),
    EffectParameter::new("intensity", EffectParameterType::F32),
];
const FIRE_PARAMETERS: &[EffectParameter] = &[
    EffectParameter::new("phase", EffectParameterType::F32),
    EffectParameter::new("intensity", EffectParameterType::F32),
    EffectParameter::new("frequency", EffectParameterType::F32),
    EffectParameter::new("expansion", EffectParameterType::LogicalPixels),
];
const GRADIENT_PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment(
    "gradient",
    include_str!("shaders/effects/animated_gradient.wgsl"),
)];
const FIRE_PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment(
    "fire",
    include_str!("shaders/effects/worley_border_fire.wgsl"),
)];

pub(crate) fn definitions() -> [EffectDefinition; 2] {
    [
        EffectDefinition::new(WORLEY_BORDER_FIRE_ID, FIRE_PARAMETERS, FIRE_PASSES),
        EffectDefinition::new(ANIMATED_GRADIENT_ID, GRADIENT_PARAMETERS, GRADIENT_PASSES),
    ]
}

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
