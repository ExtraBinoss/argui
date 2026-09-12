use argui_paint::{EffectId, EffectInstance, EffectValue, Filter};
use argui_render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition};

pub const LIQUID_GLASS_ID: EffectId = EffectId::new("argui.liquid-glass");
const PARAMETERS: &[EffectParameter] = &[
    EffectParameter::new("refraction", EffectParameterType::LogicalPixels),
    EffectParameter::new("chromatic-aberration", EffectParameterType::F32),
    EffectParameter::new("blur", EffectParameterType::LogicalPixels),
    EffectParameter::new("highlight", EffectParameterType::F32),
    EffectParameter::new("edge-width", EffectParameterType::LogicalPixels),
    EffectParameter::new("saturation", EffectParameterType::F32),
    EffectParameter::new("brightness", EffectParameterType::F32),
    EffectParameter::new("contrast", EffectParameterType::F32),
    EffectParameter::new("depth-effect", EffectParameterType::Bool),
    EffectParameter::new("tint", EffectParameterType::Vec4),
];
const PASSES: &[EffectPassDefinition] = &[
    EffectPassDefinition::fragment("blur-x", concat!(
        include_str!("shaders/effects/liquid_glass_blur.wgsl"),
        "
fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> { return glass_blur_axis(uv, source, vec2<f32>(1.0, 0.0)); }"
    )),
    EffectPassDefinition::fragment("blur-y", concat!(
        include_str!("shaders/effects/liquid_glass_blur.wgsl"),
        "
fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> { return glass_blur_axis(uv, source, vec2<f32>(0.0, 1.0)); }"
    )),
    EffectPassDefinition::fragment("glass", include_str!("shaders/effects/liquid_glass.wgsl")),
];
pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition::new(LIQUID_GLASS_ID, PARAMETERS, PASSES)
}
fn finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

/// Rounded lens adapted from Kyant's Backdrop 2.0.0, as used by SimpMusic.
/// Lengths are logical pixels; dispersion, color controls and depth are dimensionless.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiquidGlass {
    pub refraction: f32,
    pub chromatic_aberration: f32,
    pub blur: f32,
    pub highlight: f32,
    pub edge_width: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub depth_effect: bool,
    /// Linear RGB and mix amount; does not change source coverage.
    pub tint: [f32; 4],
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
            refraction: 38.0,
            chromatic_aberration: 0.0,
            blur: 8.0,
            highlight: 0.5,
            edge_width: 19.0,
            saturation: 2.25,
            brightness: 0.05,
            contrast: 1.0,
            depth_effect: false,
            tint: [0.0, 0.0, 0.0, 0.272],
        }
    }
    /// Maximum inward lens displacement, in logical pixels (0–64).
    #[must_use]
    pub const fn refraction(mut self, value: f32) -> Self {
        self.refraction = value;
        self
    }
    /// Spectral dispersion amount (0–1); 1 matches Backdrop’s enabled setting.
    #[must_use]
    pub const fn chromatic_aberration(mut self, value: f32) -> Self {
        self.chromatic_aberration = value;
        self
    }
    /// Gaussian blur sigma in logical pixels (0–16); zero keeps details sharp.
    #[must_use]
    pub const fn blur(mut self, value: f32) -> Self {
        self.blur = value;
        self
    }
    /// Directional reflection on the glass rim (0–1).
    #[must_use]
    pub const fn highlight(mut self, value: f32) -> Self {
        self.highlight = value;
        self
    }
    /// Depth of the refracting rim, in logical pixels (0–64).
    #[must_use]
    pub const fn edge_width(mut self, value: f32) -> Self {
        self.edge_width = value;
        self
    }
    /// Color saturation (0–4); 1 preserves source saturation.
    #[must_use]
    pub const fn saturation(mut self, value: f32) -> Self {
        self.saturation = value;
        self
    }
    /// Linear brightness adjustment (−1–1).
    #[must_use]
    pub const fn brightness(mut self, value: f32) -> Self {
        self.brightness = value;
        self
    }
    /// Linear contrast around mid-gray (0–4).
    #[must_use]
    pub const fn contrast(mut self, value: f32) -> Self {
        self.contrast = value;
        self
    }
    /// Blend radial depth into the rounded rectangle's refraction normal.
    #[must_use]
    pub const fn depth_effect(mut self, value: bool) -> Self {
        self.depth_effect = value;
        self
    }
    /// Linear RGB plus tint amount. Alpha does not change source coverage.
    #[must_use]
    pub const fn tint(mut self, value: [f32; 4]) -> Self {
        self.tint = value;
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
                        EffectValue::F32(finite(self.chromatic_aberration, 0.0).clamp(0.0, 1.0)),
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
                        EffectValue::F32(finite(self.saturation, 1.5).clamp(0.0, 4.0)),
                    ),
                    (
                        "brightness",
                        EffectValue::F32(finite(self.brightness, 0.0).clamp(-1.0, 1.0)),
                    ),
                    (
                        "contrast",
                        EffectValue::F32(finite(self.contrast, 1.0).clamp(0.0, 4.0)),
                    ),
                    ("depth-effect", EffectValue::Bool(self.depth_effect)),
                    (
                        "tint",
                        EffectValue::Vec4(self.tint.map(|v| finite(v, 0.0).clamp(0.0, 1.0))),
                    ),
                ],
            )
            .expansion(
                finite(self.refraction, 0.0).clamp(0.0, 64.0)
                    * (1.0 + finite(self.chromatic_aberration, 0.0).clamp(0.0, 1.0))
                    + 3.0 * finite(self.blur, 0.0).clamp(0.0, 16.0),
            ),
        )
    }
}
