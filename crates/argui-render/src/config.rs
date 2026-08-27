#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RendererConfig {
    pub power_preference: wgpu::PowerPreference,
    pub present_mode: wgpu::PresentMode,
    pub maximum_frame_latency: u32,
    pub clear_color: [f64; 4],
    pub profiling: bool,
    pub image_cache_bytes: usize,
    pub gradient_stop_capacity: usize,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            present_mode: wgpu::PresentMode::AutoVsync,
            maximum_frame_latency: 2,
            clear_color: [0.055, 0.065, 0.09, 1.0],
            profiling: false,
            image_cache_bytes: 64 * 1024 * 1024,
            gradient_stop_capacity: 65_536,
        }
    }
}

impl RendererConfig {
    #[must_use]
    pub const fn maximum_frame_latency(mut self, frames: u32) -> Self {
        self.maximum_frame_latency = if frames < 1 {
            1
        } else if frames > 3 {
            3
        } else {
            frames
        };
        self
    }

    #[must_use]
    pub const fn profiling(mut self, enabled: bool) -> Self {
        self.profiling = enabled;
        self
    }

    #[must_use]
    pub const fn image_cache_bytes(mut self, bytes: usize) -> Self {
        self.image_cache_bytes = bytes;
        self
    }

    #[must_use]
    pub const fn gradient_stop_capacity(mut self, stops: usize) -> Self {
        self.gradient_stop_capacity = stops;
        self
    }

    pub(crate) const fn wgpu_clear_color(self) -> wgpu::Color {
        wgpu::Color {
            r: self.clear_color[0],
            g: self.clear_color[1],
            b: self.clear_color[2],
            a: self.clear_color[3],
        }
    }
}
