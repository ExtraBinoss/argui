use std::time::Duration;

use web_time::Instant;

use crate::{EffectGraphStats, TexturePoolStats};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderProfile {
    pub cpu_time: Duration,
    pub viewport_pixels: u64,
    pub draw_batches: usize,
    pub effects: EffectGraphStats,
    pub texture_pool: TexturePoolStats,
    pub direct_surface: bool,
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
    ) -> Option<RenderProfile> {
        self.0.map(|started| RenderProfile {
            cpu_time: started.elapsed(),
            viewport_pixels: viewport[0] as u64 * viewport[1] as u64,
            draw_batches,
            effects,
            texture_pool,
            direct_surface: effects.offscreen_layers == 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{EffectGraphStats, TexturePoolStats};

    use super::FrameProfiler;

    #[test]
    fn profiling_is_opt_in_and_reports_scene_costs() {
        assert!(
            FrameProfiler::start(false)
                .finish(
                    [100.0, 50.0],
                    3,
                    EffectGraphStats::default(),
                    TexturePoolStats::default(),
                )
                .is_none()
        );
        let profile = FrameProfiler::start(true)
            .finish(
                [100.0, 50.0],
                3,
                EffectGraphStats::default(),
                TexturePoolStats::default(),
            )
            .unwrap();
        assert_eq!(profile.viewport_pixels, 5_000);
        assert_eq!(profile.draw_batches, 3);
        assert!(profile.direct_surface);
    }
}
