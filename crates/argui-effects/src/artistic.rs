use argui_paint::{EffectId, EffectInstance, EffectValue, Filter};
use argui_render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition};

pub const WORLEY_BORDER_FIRE_ID: EffectId = EffectId::new("argui.artistic.worley-border-fire");
pub const LIQUID_GLASS_ID: EffectId = EffectId::new("argui.artistic.liquid-glass");
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
const GLASS_PARAMETERS: &[EffectParameter] = &[
    EffectParameter::new("refraction", EffectParameterType::LogicalPixels),
    EffectParameter::new("chromatic-aberration", EffectParameterType::LogicalPixels),
    EffectParameter::new("blur", EffectParameterType::LogicalPixels),
    EffectParameter::new("highlight", EffectParameterType::F32),
    EffectParameter::new("edge-width", EffectParameterType::LogicalPixels),
    EffectParameter::new("saturation", EffectParameterType::F32),
];
const GRADIENT_PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment(
    "gradient",
    include_str!("shaders/effects/animated_gradient.wgsl"),
)];
const FIRE_PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment(
    "fire",
    include_str!("shaders/effects/worley_border_fire.wgsl"),
)];
const GLASS_PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment(
    "glass",
    include_str!("shaders/effects/liquid_glass.wgsl"),
)];

pub(crate) fn definitions() -> [EffectDefinition; 3] {
    [
        EffectDefinition::new(WORLEY_BORDER_FIRE_ID, FIRE_PARAMETERS, FIRE_PASSES),
        EffectDefinition::new(LIQUID_GLASS_ID, GLASS_PARAMETERS, GLASS_PASSES),
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
        Self::new()
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
    pub const fn refraction(mut self, value: f32) -> Self {
        self.refraction = value;
        self
    }
    #[must_use]
    pub const fn chromatic_aberration(mut self, value: f32) -> Self {
        self.chromatic_aberration = value;
        self
    }
    #[must_use]
    pub const fn blur(mut self, value: f32) -> Self {
        self.blur = value;
        self
    }
    #[must_use]
    pub const fn highlight(mut self, value: f32) -> Self {
        self.highlight = value;
        self
    }
    #[must_use]
    pub const fn edge_width(mut self, value: f32) -> Self {
        self.edge_width = value;
        self
    }
    #[must_use]
    pub const fn saturation(mut self, value: f32) -> Self {
        self.saturation = value;
        self
    }

    #[must_use]
    pub fn filter(self) -> Filter {
        Filter::Effect(EffectInstance::new(
            LIQUID_GLASS_ID,
            [
                ("refraction", EffectValue::LogicalPixels(self.refraction)),
                (
                    "chromatic-aberration",
                    EffectValue::LogicalPixels(self.chromatic_aberration),
                ),
                ("blur", EffectValue::LogicalPixels(self.blur)),
                ("highlight", EffectValue::F32(self.highlight)),
                ("edge-width", EffectValue::LogicalPixels(self.edge_width)),
                ("saturation", EffectValue::F32(self.saturation)),
            ],
        ))
    }
}
