use argui_paint::{EffectId, EffectInstance, EffectValue, Filter};
use argui_render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition};

pub const LIQUID_GLASS_ID: EffectId = EffectId::new("argui.liquid-glass");
const PARAMETERS: &[EffectParameter] = &[
    EffectParameter::new("refraction", EffectParameterType::LogicalPixels),
    EffectParameter::new("chromatic-aberration", EffectParameterType::LogicalPixels),
    EffectParameter::new("blur", EffectParameterType::LogicalPixels),
    EffectParameter::new("highlight", EffectParameterType::F32),
    EffectParameter::new("edge-width", EffectParameterType::LogicalPixels),
    EffectParameter::new("saturation", EffectParameterType::F32),
    EffectParameter::new("wavelength", EffectParameterType::LogicalPixels),
    EffectParameter::new("octaves", EffectParameterType::U32),
    EffectParameter::new("seed", EffectParameterType::U32),
    EffectParameter::new("turbulence", EffectParameterType::F32),
    EffectParameter::new("tint", EffectParameterType::Vec4),
    EffectParameter::new("ior", EffectParameterType::F32),
    EffectParameter::new("fresnel", EffectParameterType::F32),
];
const PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment(
    "glass",
    include_str!("shaders/effects/liquid_glass.wgsl"),
)];
pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition::new(LIQUID_GLASS_ID, PARAMETERS, PASSES)
}
fn finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiquidGlass {
    pub refraction: f32,
    pub chromatic_aberration: f32,
    pub blur: f32,
    pub highlight: f32,
    pub edge_width: f32,
    pub saturation: f32,
    pub frequency: f32,
    pub octaves: u32,
    pub seed: u32,
    pub turbulence: f32,
    pub tint: [f32; 4],
    pub ior: f32,
    pub fresnel: f32,
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
            chromatic_aberration: 0.25,
            blur: 0.8,
            highlight: 0.12,
            edge_width: 18.0,
            saturation: 1.0,
            frequency: 0.025,
            octaves: 3,
            seed: 0,
            turbulence: 0.0,
            tint: [1.0, 1.0, 1.0, 0.06],
            ior: 1.45,
            fresnel: 0.12,
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
    pub const fn frequency(mut self, value: f32) -> Self {
        self.frequency = value;
        self
    }
    #[must_use]
    pub const fn octaves(mut self, value: u32) -> Self {
        self.octaves = value;
        self
    }
    #[must_use]
    pub const fn seed(mut self, value: u32) -> Self {
        self.seed = value;
        self
    }
    #[must_use]
    pub const fn turbulence(mut self, value: f32) -> Self {
        self.turbulence = value;
        self
    }
    /// Linear RGB plus tint amount. Alpha does not change the source coverage.
    #[must_use]
    pub const fn tint(mut self, value: [f32; 4]) -> Self {
        self.tint = value;
        self
    }

    #[must_use]
    pub const fn ior(mut self, value: f32) -> Self {
        self.ior = value;
        self
    }
    #[must_use]
    pub const fn fresnel(mut self, value: f32) -> Self {
        self.fresnel = value;
        self
    }

    #[must_use]
    pub fn filter(self) -> Filter {
        Filter::Effect(
            EffectInstance::new(
                LIQUID_GLASS_ID,
                [
                    (
                        "refraction",
                        EffectValue::LogicalPixels(finite(self.refraction, 0.0).clamp(0.0, 64.0)),
                    ),
                    (
                        "chromatic-aberration",
                        EffectValue::LogicalPixels(
                            finite(self.chromatic_aberration, 0.0).clamp(0.0, 8.0),
                        ),
                    ),
                    (
                        "blur",
                        EffectValue::LogicalPixels(finite(self.blur, 0.0).clamp(0.0, 16.0)),
                    ),
                    (
                        "highlight",
                        EffectValue::F32(finite(self.highlight, 0.0).clamp(0.0, 1.0)),
                    ),
                    (
                        "edge-width",
                        EffectValue::LogicalPixels(finite(self.edge_width, 0.0).clamp(0.0, 64.0)),
                    ),
                    (
                        "saturation",
                        EffectValue::F32(finite(self.saturation, 1.0).clamp(0.0, 4.0)),
                    ),
                    (
                        "wavelength",
                        EffectValue::LogicalPixels(
                            1.0 / finite(self.frequency, 0.025).clamp(0.001, 1.0),
                        ),
                    ),
                    ("octaves", EffectValue::U32(self.octaves.clamp(1, 6))),
                    ("seed", EffectValue::U32(self.seed)),
                    (
                        "turbulence",
                        EffectValue::F32(finite(self.turbulence, 0.0).clamp(0.0, 1.0)),
                    ),
                    (
                        "tint",
                        EffectValue::Vec4(self.tint.map(|v| finite(v, 0.0).clamp(0.0, 1.0))),
                    ),
                    (
                        "ior",
                        EffectValue::F32(finite(self.ior, 1.45).clamp(1.0, 2.5)),
                    ),
                    (
                        "fresnel",
                        EffectValue::F32(finite(self.fresnel, 0.0).clamp(0.0, 1.0)),
                    ),
                ],
            )
            .expansion(
                finite(self.refraction, 0.0).clamp(0.0, 64.0) * 1.5
                    + finite(self.chromatic_aberration, 0.0).clamp(0.0, 8.0)
                    + finite(self.blur, 0.0).clamp(0.0, 16.0),
            ),
        )
    }
}
