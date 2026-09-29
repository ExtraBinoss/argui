//! Native surface size and presentation lifecycle.

use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Uses tear-free mailbox presentation when automatic vsync and surface
    /// capabilities permit it. Returns whether mailbox pacing is available.
    ///
    /// A covered native FIFO swapchain can exhaust its images and block the
    /// shared UI thread. Mailbox replaces queued frames instead of waiting for
    /// the compositor to consume them. The caller must bound animation cadence.
    pub fn prefer_mailbox_presentation(&mut self) -> bool {
        let Some(surface) = &self.surface else {
            return false;
        };
        let supported = surface
            .get_capabilities(&self.device_handle.0.adapter)
            .present_modes;
        let mode = mailbox_present_mode(self.renderer_config.present_mode, &supported);
        if mode != self.surface_config.present_mode {
            self.surface_config.present_mode = mode;
            self.reconfigure_surface();
        }
        mode == wgpu::PresentMode::Mailbox
    }

    /// Runs `ready` after previously submitted GPU work completes.
    ///
    /// A new native window can wait for its first pixels before mapping. Polling
    /// runs on a short-lived worker, so readiness never blocks the UI event loop.
    /// The callback must be thread-safe; it may request a native redraw.
    pub fn after_frame_ready(&self, ready: impl FnOnce() + Send + 'static) {
        let device = self.device.clone();
        self.queue.on_submitted_work_done(ready);
        std::thread::spawn(move || {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
        });
    }

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
        self.deferred_reconfigure = false;
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
        self.deferred_reconfigure = false;
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
        self.deferred_reconfigure = false;
        self.surface
            .as_ref()
            .expect("replacement surface exists")
            .configure(&self.device, &self.surface_config);
        self.layer_cache.clear();
        self.damage.invalidate();
        self.scene_snapshot = None;
        self.effect_root = None;
        Ok(())
    }
}

/// Resolves automatic presentation against `supported`, preserving explicit modes.
fn mailbox_present_mode(
    requested: wgpu::PresentMode,
    supported: &[wgpu::PresentMode],
) -> wgpu::PresentMode {
    if requested == wgpu::PresentMode::AutoVsync && supported.contains(&wgpu::PresentMode::Mailbox)
    {
        wgpu::PresentMode::Mailbox
    } else {
        requested
    }
}

#[cfg(test)]
#[path = "../../tests/surface/presentation.rs"]
mod tests;
