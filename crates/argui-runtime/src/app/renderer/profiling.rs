use crate::{RuntimeEvent, app::Application};
use argui_render::SurfaceRenderer;

#[cfg(feature = "inspect")]
use argui_inspect::{AdapterRecord, FrameRecord, GpuFrameRecord, GpuPassRecord};
#[cfg(feature = "inspect")]
use argui_render::{AdapterProfile, DamageMode, GpuFrameProfile};

impl Application {
    /// Reports the presented frame from `renderer` to enabled profiling consumers.
    pub(super) fn report_render(&mut self, renderer: &SurfaceRenderer) {
        #[cfg(feature = "inspect")]
        if let Some(inspector) = &self.inspector {
            let profile = renderer.last_profile();
            inspector.record_render(FrameRecord {
                render_cpu: profile.cpu_time,
                layers: profile.effects.offscreen_layers,
                passes: profile.effects.filter_passes,
                offscreen_pixels: profile.effects.offscreen_pixels,
                cached_layers: profile.effects.cached_layers,
                damaged_pixels: profile.damage.damaged_pixels,
                textures: profile.texture_pool.textures
                    + 2
                    + profile.gpu_canvases.entries
                    + usize::from(profile.damage.retained_bytes > 0)
                    + 1,
                reused_textures: profile.texture_pool.reused_this_frame
                    + profile.gpu_canvases.hits_this_frame
                    + usize::from(matches!(
                        profile.damage.mode,
                        DamageMode::Partial | DamageMode::Reused
                    )),
                texture_bytes: profile.texture_pool.allocated_bytes
                    + profile.text_atlas.allocated_bytes
                    + profile.vector_atlas.allocated_bytes
                    + profile.gpu_canvases.allocated_bytes
                    + profile.damage.retained_bytes,
                gpu_canvas_entries: profile.gpu_canvases.entries,
                gpu_canvas_bytes: profile.gpu_canvases.allocated_bytes,
                gpu_canvas_renders: profile.gpu_canvases.renders_this_frame,
                gpu_canvas_hits: profile.gpu_canvases.hits_this_frame,
                gpu_canvas_failures: profile.gpu_canvases.failures_this_frame,
                gpu_canvas_encode_cpu: profile.gpu_canvases.encode_time,
                text_atlas_bytes: profile.text_atlas.allocated_bytes,
                text_atlas_entries: profile.text_atlas.entries,
                text_atlas_hits: profile.text_atlas.hits_this_frame,
                text_raster_requests: profile.text_atlas.raster_requests_this_frame,
                text_upload_bytes: profile.text_atlas.uploaded_bytes_this_frame,
                text_page_evictions: profile.text_atlas.evictions_this_frame,
                vector_atlas_entries: profile.vector_atlas.entries,
                vector_atlas_bytes: profile.vector_atlas.allocated_bytes,
                vector_atlas_hits: profile.vector_atlas.hits_this_frame,
                vector_rasterizations: profile.vector_atlas.rasterizations_this_frame,
                adapter: adapter_record(&profile.adapter),
                gpu: profile.gpu.as_ref().map(gpu_record),
                ..FrameRecord::default()
            });
        }
        if self.renderer_profiling_requested {
            (self.on_event)(RuntimeEvent::RenderProfile(Box::new(
                renderer.last_profile(),
            )));
        }
    }
}

#[cfg(feature = "inspect")]
fn adapter_record(profile: &AdapterProfile) -> AdapterRecord {
    AdapterRecord {
        name: profile.name.clone(),
        vendor: profile.vendor,
        device: profile.device,
        device_type: profile.device_type.clone(),
        driver: profile.driver.clone(),
        driver_info: profile.driver_info.clone(),
        backend: profile.backend.clone(),
        features: profile.features.clone(),
        timestamp_queries: profile.timestamp_queries,
        max_texture_dimension_2d: profile.max_texture_dimension_2d,
        max_buffer_size: profile.max_buffer_size,
        max_storage_buffer_binding_size: profile.max_storage_buffer_binding_size,
        max_bind_groups: profile.max_bind_groups,
    }
}

#[cfg(feature = "inspect")]
fn gpu_record(profile: &GpuFrameProfile) -> GpuFrameRecord {
    GpuFrameRecord {
        sequence: profile.frame,
        total: profile.total,
        passes: profile
            .passes
            .iter()
            .map(|pass| GpuPassRecord {
                label: pass.label.clone(),
                start: pass.start,
                duration: pass.duration,
                pixels: pass.pixels,
                object_domain: pass.object.map(|object| format!("{:?}", object.domain)),
                object_id: pass.object.map(|object| object.value),
            })
            .collect(),
    }
}
