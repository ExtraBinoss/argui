//! Native surface size and presentation lifecycle.

use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Resizes the surface; returns whether its drawable extent changed.
    /// * `width`, `height` — requested drawable dimensions in physical pixels.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn resize(&mut self, width: u32, height: u32) -> bool {
        let Some((width, height)) = drawable_size(width, height) else {
            return false;
        };
        if self.surface_config.width == width && self.surface_config.height == height {
            return false;
        }
        self.surface_config.width = width;
        self.surface_config.height = height;
        if let Some(surface) = &self.surface {
            surface.configure(&self.device, &self.surface_config);
        } else {
            self.offscreen_target = Some(offscreen_texture(&self.device, width, height));
        }
        self.layer_cache.clear();
        self.damage.invalidate();
        self.scene_snapshot = None;
        self.effect_root = None;
        true
    }

    /// Reconfigures a native surface after WGPU reports an outdated or suboptimal frame.
    ///
    /// This also handles a swapchain change that leaves the drawable width and
    /// height unchanged. Retained paint is invalidated because remapping may
    /// discard surface contents. An offscreen renderer has no surface to configure.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn reconfigure_surface(&mut self) {
        if let Some(surface) = &self.surface {
            surface.configure(&self.device, &self.surface_config);
        }
        self.layer_cache.clear();
        self.damage.invalidate();
        self.scene_snapshot = None;
        self.effect_root = None;
    }

    /// Replaces the native surface target while retaining the existing GPU device.
    ///
    /// # Errors
    /// Returns an error for a windowless renderer or if surface creation fails.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn recreate_surface(
        &mut self,
        target: impl Into<SurfaceTarget<'static>>,
    ) -> Result<(), RendererError> {
        self.surface
            .as_ref()
            .ok_or(RendererError::UnsupportedSurface)?;
        self.surface = Some(
            self.instance
                .create_surface(target)
                .map_err(|error| RendererError::SurfaceCreation(error.to_string()))?,
        );
        self.offscreen_target = None;
        self.surface
            .as_ref()
            .expect("replacement surface exists")
            .configure(&self.device, &self.surface_config);
        self.layer_cache.clear();
        self.damage.invalidate();
        self.effect_root = None;
        Ok(())
    }
}
