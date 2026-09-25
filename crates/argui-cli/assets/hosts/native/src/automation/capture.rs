//! Offscreen captures share one GPU renderer across automation steps.

use std::{path::Path, time::Instant};

use argui_automation::Driver;
use argui_core::MetricTrace;
use argui_render::{RendererConfig, SurfaceRenderer};
use argui_runtime::NativeHostAssets;
use image::{ColorType, ImageFormat};
use serde_json::{Value, json};

use super::safe_capture_path;

/// Lazily created offscreen renderer and its artifact destination.
pub(super) struct CaptureState<'a> {
    out: &'a Path,
    assets: &'a NativeHostAssets,
    renderer: Option<SurfaceRenderer>,
    pub(super) adapter: Option<Value>,
}

impl<'a> CaptureState<'a> {
    /// Creates capture state for `out` with `assets` registered on first render.
    pub(super) fn new(out: &'a Path, assets: &'a NativeHostAssets) -> Self {
        Self { out, assets, renderer: None, adapter: None }
    }

    /// Renders the current `driver` scene at its latest viewport size.
    /// `trace` receives the renderer preparation and submission spans.
    ///
    /// # Errors
    /// Returns an offscreen GPU or scene error.
    pub(super) fn render_scene(
        &mut self,
        driver: &mut Driver,
        trace: &MetricTrace,
    ) -> Result<(), String> {
        let viewport = driver.viewport();
        let (width, height) = viewport.physical_size()?;
        if self.renderer.is_none() {
            let _init = trace.span("render.adapter_init");
            let mut gpu = pollster::block_on(SurfaceRenderer::new_offscreen(
                width,
                height,
                RendererConfig::default().profiling(true),
            ))
            .map_err(|error| format!("screenshot GPU: {error}"))?;
            for image in &self.assets.images {
                gpu.register_image(image)
                    .map_err(|error| format!("screenshot image: {error}"))?;
            }
            for vector in &self.assets.vectors {
                gpu.register_vector(vector)
                    .map_err(|error| format!("screenshot vector: {error}"))?;
            }
            self.renderer = Some(gpu);
        }
        let gpu = self.renderer.as_mut().expect("renderer initialized");
        let _resize = trace.span("render.surface_resize");
        let began_resize = Instant::now();
        let changed = gpu.resize(width, height);
        if changed {
            driver.record_surface_resize(began_resize.elapsed());
        }
        drop(_resize);
        let began = Instant::now();
        {
            let _prepare = trace.span("render.prepare_text");
            let (layout, text) = driver.scene()?;
            let prepared = text.prepare(&layout.text, viewport.scale);
            drop(_prepare);
            let _submit = trace.span("render.submit_cpu");
            gpu.render_ui(text, &prepared, &layout.display_list, viewport.scale)
                .map_err(|error| format!("screenshot render: {error}"))?;
        }
        driver.record_render(began.elapsed());
        let profile = gpu.last_profile();
        driver.record_render_profile(&profile);
        self.adapter = Some(json!({ "name": profile.adapter.name,
            "backend": profile.adapter.backend, "deviceType": profile.adapter.device_type,
            "timestampQueries": profile.adapter.timestamp_queries }));
        Ok(())
    }

    /// Writes `name` using the current `driver` scene and adds GPU work to `trace`.
    ///
    /// # Errors
    /// Returns an artifact, GPU, or blank-capture error.
    pub(super) fn screenshot(
        &mut self,
        driver: &mut Driver,
        name: &str,
        trace: &MetricTrace,
    ) -> Result<(), String> {
        let _capture = trace.span("render.capture");
        let path = safe_capture_path(self.out, name)?;
        self.render_scene(driver, trace)?;
        let (width, height) = driver.viewport().physical_size()?;
        let gpu = self.renderer.as_mut().expect("renderer initialized");
        let _readback = trace.span("render.readback_wait");
        let pixels = gpu
            .read_offscreen_rgba()
            .map_err(|error| format!("screenshot readback: {error}"))?;
        drop(_readback);
        if !pixels.as_chunks::<4>().0.iter().any(|pixel| pixel != &pixels[..4]) {
            return Err("screenshot was blank; check the mounted app and GPU renderer".into());
        }
        let _write = trace.span("artifact.png_write");
        image::save_buffer_with_format(
            &path,
            &pixels,
            width,
            height,
            ColorType::Rgba8,
            ImageFormat::Png,
        )
        .map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(())
    }
}
