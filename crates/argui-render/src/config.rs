#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RendererConfig {
    pub power_preference: wgpu::PowerPreference,
    pub present_mode: wgpu::PresentMode,
    pub clear_color: [f64; 4],
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            present_mode: wgpu::PresentMode::AutoVsync,
            clear_color: [0.055, 0.065, 0.09, 1.0],
        }
    }
}

impl RendererConfig {
    pub(crate) const fn wgpu_clear_color(self) -> wgpu::Color {
        wgpu::Color {
            r: self.clear_color[0],
            g: self.clear_color[1],
            b: self.clear_color[2],
            a: self.clear_color[3],
        }
    }
}
