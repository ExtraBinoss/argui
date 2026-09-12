use crate::EffectRegistry;
use argui_core::Color;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SurfaceAlphaMode {
    #[default]
    Opaque,
    Transparent,
    /// Use alpha when supported, otherwise keep an opaque surface.
    PreferTransparent,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EffectQuality {
    #[default]
    Normal,
    Balanced,
    Performance,
    Custom(EffectQualitySettings),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectQualitySettings {
    pub blur_downsample_bias: u32,
    pub spatial_effect_divisor: u32,
}

impl EffectQuality {
    #[must_use]
    pub const fn settings(self) -> EffectQualitySettings {
        match self {
            Self::Normal => EffectQualitySettings {
                blur_downsample_bias: 1,
                spatial_effect_divisor: 1,
            },
            Self::Balanced => EffectQualitySettings {
                blur_downsample_bias: 2,
                spatial_effect_divisor: 1,
            },
            Self::Performance => EffectQualitySettings {
                blur_downsample_bias: 2,
                spatial_effect_divisor: 2,
            },
            Self::Custom(settings) => settings,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RendererConfig {
    pub power_preference: wgpu::PowerPreference,
    pub present_mode: wgpu::PresentMode,
    pub maximum_frame_latency: u32,
    pub clear_color: Color,
    pub surface_alpha: SurfaceAlphaMode,
    pub profiling: bool,
    pub image_cache_bytes: usize,
    pub gradient_stop_capacity: usize,
    pub effects: EffectRegistry,
    pub effect_quality: EffectQuality,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            present_mode: wgpu::PresentMode::AutoVsync,
            maximum_frame_latency: 2,
            clear_color: Color::srgb(0.055, 0.065, 0.09),
            surface_alpha: SurfaceAlphaMode::Opaque,
            profiling: false,
            image_cache_bytes: 64 * 1024 * 1024,
            gradient_stop_capacity: 65_536,
            effects: EffectRegistry::default(),
            effect_quality: EffectQuality::Normal,
        }
    }
}

impl RendererConfig {
    #[must_use]
    pub fn maximum_frame_latency(mut self, frames: u32) -> Self {
        self.maximum_frame_latency = frames.clamp(1, 3);
        self
    }

    #[must_use]
    pub fn profiling(mut self, enabled: bool) -> Self {
        self.profiling = enabled;
        self
    }

    #[must_use]
    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }

    #[must_use]
    pub fn surface_alpha(mut self, mode: SurfaceAlphaMode) -> Self {
        self.surface_alpha = mode;
        if mode != SurfaceAlphaMode::Opaque {
            self.clear_color = Color::TRANSPARENT;
        }
        self
    }

    #[must_use]
    pub fn image_cache_bytes(mut self, bytes: usize) -> Self {
        self.image_cache_bytes = bytes;
        self
    }

    #[must_use]
    pub fn gradient_stop_capacity(mut self, stops: usize) -> Self {
        self.gradient_stop_capacity = stops;
        self
    }

    #[must_use]
    pub fn effects(mut self, effects: EffectRegistry) -> Self {
        self.effects = effects;
        self
    }

    #[must_use]
    pub fn effect_quality(mut self, quality: EffectQuality) -> Self {
        self.effect_quality = quality;
        self
    }

    pub(crate) fn wgpu_clear_color(&self) -> wgpu::Color {
        let [red, green, blue, alpha] = self.clear_color.to_linear_rgba();
        wgpu::Color {
            r: f64::from(red),
            g: f64::from(green),
            b: f64::from(blue),
            a: f64::from(alpha),
        }
    }
}
