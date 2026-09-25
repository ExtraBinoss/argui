use crate::{EffectRegistry, GpuCanvasRegistry};
use argui_core::Color;

/// Adaptive thresholds used to choose partial or full-surface rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageTracking {
    /// Whether retained damage rendering is available.
    pub enabled: bool,
    /// Maximum merged damage rectangles allowed in one partial frame.
    pub max_regions: usize,
    /// Maximum damaged viewport fraction allowed in one partial frame.
    pub max_area_ratio: f32,
}

impl Default for DamageTracking {
    fn default() -> Self {
        Self {
            enabled: true,
            max_regions: 8,
            max_area_ratio: 0.45,
        }
    }
}

impl DamageTracking {
    /// Returns damage tracking with the adaptive defaults enabled.
    #[must_use]
    pub const fn enabled() -> Self {
        Self {
            enabled: true,
            max_regions: 8,
            max_area_ratio: 0.45,
        }
    }

    /// Returns a configuration that always renders the complete surface.
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            max_regions: 0,
            max_area_ratio: 0.0,
        }
    }

    /// Sets the maximum number of merged regions in a partial frame.
    ///
    /// * `regions` — requested limit, clamped to at least one.
    #[must_use]
    pub fn max_regions(mut self, regions: usize) -> Self {
        self.max_regions = regions.max(1);
        self
    }

    /// Sets the maximum damaged viewport fraction in a partial frame.
    ///
    /// * `ratio` — fraction clamped to the inclusive `0.05..=1.0` range.
    #[must_use]
    pub fn max_area_ratio(mut self, ratio: f32) -> Self {
        self.max_area_ratio = if ratio.is_finite() {
            ratio.clamp(0.05, 1.0)
        } else {
            Self::default().max_area_ratio
        };
        self
    }
}

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

/// Selects the filter used for CSS-style blur and blurred shadows.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlurAlgorithm {
    /// Selects dual filtering for medium radii, Gaussian otherwise.
    #[default]
    Auto,
    /// Uses two separable Gaussian render passes, with bilinear-paired taps.
    Gaussian,
    /// Uses the SIGGRAPH 2015 downsample and upsample filter pyramid.
    DualKawase,
}

impl BlurAlgorithm {
    /// Resolves automatic selection for physical `radius` and input `extent`.
    ///
    /// Returns the selected fixed algorithm. The current crossover favors the
    /// dual filter for medium radii, where it avoids Gaussian's full-resolution
    /// samples or extra downsample copy. Explicit selections are preserved.
    #[must_use]
    pub fn resolve(self, radius: f32, extent: [u32; 2]) -> Self {
        match self {
            Self::Auto if (3.0..7.0).contains(&radius) && extent[0].min(extent[1]) >= 16 => {
                Self::DualKawase
            }
            Self::Auto => Self::Gaussian,
            fixed => fixed,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectQualitySettings {
    pub blur_downsample_bias: u32,
    pub spatial_effect_divisor: u32,
}

impl EffectQuality {
    /// Returns numeric settings corresponding to this quality preset.
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
    /// Whether renderer initialization may retry a compatible backend after the preferred one fails.
    pub renderer_fallback: bool,
    pub present_mode: wgpu::PresentMode,
    pub maximum_frame_latency: u32,
    /// Wait for submitted GPU work after each frame when bounded memory matters more than throughput.
    pub wait_for_submitted_gpu_work: bool,
    pub clear_color: Color,
    pub surface_alpha: SurfaceAlphaMode,
    pub profiling: bool,
    pub image_cache_bytes: usize,
    /// Maximum retained GPU-canvas texture bytes per surface renderer.
    pub gpu_canvas_cache_bytes: usize,
    pub gradient_stop_capacity: usize,
    pub effects: EffectRegistry,
    /// Immutable factories and device requirements available to GPU canvases.
    pub gpu_canvases: GpuCanvasRegistry,
    pub effect_quality: EffectQuality,
    /// Blur algorithm used for backdrop, foreground, and shadow filters.
    pub blur_algorithm: BlurAlgorithm,
    /// Adaptive retained-surface damage rendering configuration.
    pub damage_tracking: DamageTracking,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            renderer_fallback: true,
            present_mode: wgpu::PresentMode::AutoVsync,
            maximum_frame_latency: 2,
            wait_for_submitted_gpu_work: false,
            clear_color: Color::srgb(0.055, 0.065, 0.09),
            surface_alpha: SurfaceAlphaMode::Opaque,
            profiling: false,
            image_cache_bytes: 64 * 1024 * 1024,
            gpu_canvas_cache_bytes: 128 * 1024 * 1024,
            gradient_stop_capacity: 65_536,
            effects: EffectRegistry::default(),
            gpu_canvases: GpuCanvasRegistry::default(),
            effect_quality: EffectQuality::Normal,
            blur_algorithm: BlurAlgorithm::Auto,
            damage_tracking: DamageTracking::default(),
        }
    }
}

impl RendererConfig {
    /// Enables or disables renderer fallback during GPU initialization.
    ///
    /// When `enabled` is `false`, Windows only attempts DirectX 12 with
    /// DirectComposition, while Linux and Android only attempt Vulkan.
    /// `enabled` has no effect on other platforms.
    #[must_use]
    pub fn renderer_fallback(mut self, enabled: bool) -> Self {
        self.renderer_fallback = enabled;
        self
    }

    /// Sets the maximum number of frames queued for presentation, clamped to 1–3.
    /// * `frames` — requested frame latency; values are clamped to the supported range.
    #[must_use]
    pub fn maximum_frame_latency(mut self, frames: u32) -> Self {
        self.maximum_frame_latency = frames.clamp(1, 3);
        self
    }

    /// Waits for each submitted frame to complete before rendering another one when `enabled`.
    ///
    /// * `enabled` — whether to bound outstanding GPU command buffers at the cost of throughput.
    #[must_use]
    pub fn wait_for_submitted_gpu_work(mut self, enabled: bool) -> Self {
        self.wait_for_submitted_gpu_work = enabled;
        self
    }

    /// Enables or disables renderer profiling.
    /// * `enabled` — whether frame profiling is active.
    #[must_use]
    pub fn profiling(mut self, enabled: bool) -> Self {
        self.profiling = enabled;
        self
    }

    /// Sets the color used to clear the render target.
    /// * `color` — clear color.
    #[must_use]
    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }

    /// Sets surface transparency behavior; non-opaque modes use a transparent clear color.
    /// * `mode` — alpha compositing mode for the surface.
    #[must_use]
    pub fn surface_alpha(mut self, mode: SurfaceAlphaMode) -> Self {
        self.surface_alpha = mode;
        if mode != SurfaceAlphaMode::Opaque {
            self.clear_color = Color::TRANSPARENT;
        }
        self
    }

    /// Sets the image cache budget in bytes.
    /// * `bytes` — maximum cache budget in bytes.
    #[must_use]
    pub fn image_cache_bytes(mut self, bytes: usize) -> Self {
        self.image_cache_bytes = bytes;
        self
    }

    /// Sets the retained GPU-canvas texture budget in bytes.
    ///
    /// Requests that cannot fit this budget display a recoverable placeholder
    /// without allocating the oversized texture.
    #[must_use]
    pub fn gpu_canvas_cache_bytes(mut self, bytes: usize) -> Self {
        self.gpu_canvas_cache_bytes = bytes;
        self
    }

    /// Sets capacity reserved for gradient stops.
    /// * `stops` — reserved number of gradient stops.
    #[must_use]
    pub fn gradient_stop_capacity(mut self, stops: usize) -> Self {
        self.gradient_stop_capacity = stops;
        self
    }

    /// Sets the registry of custom effects available to the renderer.
    /// * `effects` — registry of effect definitions.
    #[must_use]
    pub fn effects(mut self, effects: EffectRegistry) -> Self {
        self.effects = effects;
        self
    }

    /// Sets the immutable GPU-canvas registrations known before device creation.
    #[must_use]
    pub fn gpu_canvases(mut self, gpu_canvases: GpuCanvasRegistry) -> Self {
        self.gpu_canvases = gpu_canvases;
        self
    }

    /// Sets the rendering quality preset used for effects.
    /// * `quality` — quality preset controlling effect resolution and cost.
    #[must_use]
    pub fn effect_quality(mut self, quality: EffectQuality) -> Self {
        self.effect_quality = quality;
        self
    }

    /// Selects the blur algorithm for all filtered layers and shadows.
    ///
    /// * `algorithm` — automatic selection or a fixed algorithm for comparison.
    ///
    /// Returns the updated renderer configuration.
    #[must_use]
    pub fn blur_algorithm(mut self, algorithm: BlurAlgorithm) -> Self {
        self.blur_algorithm = algorithm;
        self
    }

    /// Sets adaptive retained-surface damage rendering behavior.
    ///
    /// * `damage_tracking` — thresholds or a disabled configuration.
    #[must_use]
    pub fn damage_tracking(mut self, damage_tracking: DamageTracking) -> Self {
        self.damage_tracking = damage_tracking;
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
