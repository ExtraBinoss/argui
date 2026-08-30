use crate::EffectRegistry;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SurfaceAlphaMode {
    #[default]
    Opaque,
    Transparent,
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
    pub clear_color: [f64; 4],
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
            clear_color: [0.055, 0.065, 0.09, 1.0],
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
    pub fn surface_alpha(mut self, mode: SurfaceAlphaMode) -> Self {
        self.surface_alpha = mode;
        if mode == SurfaceAlphaMode::Transparent {
            self.clear_color = [0.0; 4];
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

    pub(crate) const fn wgpu_clear_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.clear_color[0],
            g: self.clear_color[1],
            b: self.clear_color[2],
            a: self.clear_color[3],
        }
    }
}
