use std::time::Duration;

use web_time::Instant;

use crate::{EffectGraphStats, TexturePoolStats};

use argui_paint::RenderObjectId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AdapterProfile {
    pub name: String,
    pub vendor: u32,
    pub device: u32,
    pub device_type: String,
    pub driver: String,
    pub driver_info: String,
    pub backend: String,
    pub features: String,
    pub timestamp_queries: bool,
    pub max_texture_dimension_2d: u32,
    pub max_buffer_size: u64,
    pub max_storage_buffer_binding_size: u64,
    pub max_bind_groups: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuPassProfile {
    pub label: String,
    pub start: Duration,
    pub duration: Duration,
    pub pixels: u64,
    pub object: Option<RenderObjectId>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuFrameProfile {
    pub frame: u64,
    pub total: Duration,
    pub passes: Vec<GpuPassProfile>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderProfile {
    pub cpu_time: Duration,
    pub viewport_pixels: u64,
    pub draw_batches: usize,
    pub effects: EffectGraphStats,
    pub texture_pool: TexturePoolStats,
    pub direct_surface: bool,
    pub adapter: AdapterProfile,
    pub gpu: Option<GpuFrameProfile>,
}

pub(crate) struct FrameProfiler(Option<Instant>);

impl FrameProfiler {
    pub fn start(enabled: bool) -> Self {
        Self(enabled.then(Instant::now))
    }

    pub fn finish(
        self,
        viewport: [f32; 2],
        draw_batches: usize,
        effects: EffectGraphStats,
        texture_pool: TexturePoolStats,
        adapter: AdapterProfile,
        gpu: Option<GpuFrameProfile>,
    ) -> Option<RenderProfile> {
        self.0.map(|started| RenderProfile {
            cpu_time: started.elapsed(),
            viewport_pixels: viewport[0] as u64 * viewport[1] as u64,
            draw_batches,
            effects,
            texture_pool,
            direct_surface: effects.offscreen_layers == 0,
            adapter,
            gpu,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{EffectGraphStats, TexturePoolStats};

    use super::{AdapterProfile, FrameProfiler};

    #[test]
    fn profiling_is_opt_in_and_reports_scene_costs() {
        assert!(
            FrameProfiler::start(false)
                .finish(
                    [100.0, 50.0],
                    3,
                    EffectGraphStats::default(),
                    TexturePoolStats::default(),
                    AdapterProfile::default(),
                    None,
                )
                .is_none()
        );
        let profile = FrameProfiler::start(true)
            .finish(
                [100.0, 50.0],
                3,
                EffectGraphStats::default(),
                TexturePoolStats::default(),
                AdapterProfile::default(),
                None,
            )
            .unwrap();
        assert_eq!(profile.viewport_pixels, 5_000);
        assert_eq!(profile.draw_batches, 3);
        assert!(profile.direct_surface);
    }
}
