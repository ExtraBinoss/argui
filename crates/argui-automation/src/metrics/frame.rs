//! Per-frame CPU, GPU, workload, and action correlation.

use argui_inspect::FrameRecord;
use serde::Serialize;

use crate::StepFrame;

/// A test action's interval on the shared automation clock.
#[derive(Clone, Copy, Debug)]
pub struct ActionWindow {
    /// Milliseconds when the action began.
    pub started_ms: f64,
    /// Milliseconds spent executing the action.
    pub duration_ms: f64,
}

/// One measured GPU render pass; absent when timestamp queries are unavailable.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuPassDiagnostic {
    /// Renderer pass label.
    pub name: String,
    /// Time after the first captured GPU timestamp.
    pub start_ms: f64,
    /// Time spent on this GPU pass.
    pub duration_ms: f64,
    /// Rasterized pixels reported by the renderer.
    pub pixels: u64,
}

/// Measured renderer work associated with a captured frame.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderWorkload {
    /// Number of submitted draw batches.
    pub draw_batches: usize,
    /// Repainted pixels after damage tracking.
    pub damaged_pixels: u64,
    /// CPU glyph rasterization requests.
    pub text_raster_requests: usize,
    /// Bytes uploaded to the glyph atlas.
    pub text_upload_bytes: u64,
    /// CPU vector rasterizations.
    pub vector_rasterizations: usize,
}

/// One scene update with measured CPU phases, optional GPU passes, and actions.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameDiagnostic {
    /// Zero-based scene update index.
    pub frame_index: usize,
    /// When the scene update completed on the automation clock.
    pub updated_at_ms: f64,
    /// Action running when the scene update completed, if any.
    pub update_action_index: Option<usize>,
    /// When optional renderer CPU work completed.
    pub rendered_at_ms: Option<f64>,
    /// Action running when the scene was rendered, if any.
    pub render_action_index: Option<usize>,
    /// Time since the previous scene update, excluding the first frame.
    pub interval_ms: Option<f64>,
    /// CPU work before geometry computation, including host or input updates.
    pub pre_layout_cpu_ms: f64,
    /// CPU geometry, reconciliation, measurement, and placement time.
    pub layout_cpu_ms: f64,
    /// CPU paint output or scroll presentation time.
    pub paint_cpu_ms: f64,
    /// Other scene update CPU time around the measured phases.
    pub other_update_cpu_ms: f64,
    /// CPU text preparation and renderer submission, when captured.
    pub render_submit_cpu_ms: Option<f64>,
    /// Sum of scene update CPU and optional renderer submission CPU.
    pub total_cpu_ms: f64,
    /// Largest measured CPU phase name in this frame.
    pub dominant_cpu_phase: &'static str,
    /// Number of container query layout passes.
    pub layout_passes: usize,
    /// GPU frame sequence, when timestamp queries completed.
    pub gpu_sequence: Option<u64>,
    /// GPU duration measured by timestamp queries, independently of CPU time.
    pub gpu_ms: Option<f64>,
    /// GPU passes from timestamp queries; empty when unavailable.
    pub gpu_passes: Vec<GpuPassDiagnostic>,
    /// Renderer counters, when this scene was rendered.
    pub render_workload: Option<RenderWorkload>,
}

/// Correlates `frames` and raw `records` with action intervals.
/// `actions` use the same monotonic clock as frame timestamps. Missing renderer
/// and GPU observations remain `None` instead of being interpreted as zero.
#[must_use]
pub fn frame_diagnostics(
    frames: &[StepFrame],
    records: &[FrameRecord],
    actions: &[ActionWindow],
) -> Vec<FrameDiagnostic> {
    frames
        .iter()
        .zip(records)
        .enumerate()
        .map(|(frame_index, (frame, record))| {
            let pre_layout_cpu_ms = ms(record.model);
            let other_update_cpu_ms =
                (frame.cpu_ms - pre_layout_cpu_ms - frame.layout_ms - frame.paint_ms).max(0.0);
            let phases = [
                ("pre_layout", pre_layout_cpu_ms),
                ("layout", frame.layout_ms),
                ("paint", frame.paint_ms),
                ("other_update", other_update_cpu_ms),
                ("render_submit", frame.render_cpu_ms.unwrap_or(0.0)),
            ];
            let dominant_cpu_phase = phases
                .into_iter()
                .max_by(|left, right| left.1.total_cmp(&right.1))
                .map_or("none", |phase| phase.0);
            let gpu = record.gpu.as_ref();
            FrameDiagnostic {
                frame_index,
                updated_at_ms: frame.timestamp_ms,
                update_action_index: action_at(frame.timestamp_ms, actions),
                rendered_at_ms: frame.rendered_at_ms,
                render_action_index: frame
                    .rendered_at_ms
                    .and_then(|time| action_at(time, actions)),
                interval_ms: frame.interval_ms,
                pre_layout_cpu_ms,
                layout_cpu_ms: frame.layout_ms,
                paint_cpu_ms: frame.paint_ms,
                other_update_cpu_ms,
                render_submit_cpu_ms: frame.render_cpu_ms,
                total_cpu_ms: frame.cpu_ms + frame.render_cpu_ms.unwrap_or(0.0),
                dominant_cpu_phase,
                layout_passes: frame.layout_passes,
                gpu_sequence: gpu.map(|profile| profile.sequence),
                gpu_ms: gpu.map(|profile| ms(profile.total)),
                gpu_passes: gpu.map_or_else(Vec::new, |profile| {
                    profile
                        .passes
                        .iter()
                        .map(|pass| GpuPassDiagnostic {
                            name: pass.label.clone(),
                            start_ms: ms(pass.start),
                            duration_ms: ms(pass.duration),
                            pixels: pass.pixels,
                        })
                        .collect()
                }),
                render_workload: frame.render_cpu_ms.map(|_| RenderWorkload {
                    draw_batches: record.passes,
                    damaged_pixels: record.damaged_pixels,
                    text_raster_requests: record.text_raster_requests,
                    text_upload_bytes: record.text_upload_bytes,
                    vector_rasterizations: record.vector_rasterizations,
                }),
            }
        })
        .collect()
}

/// Finds the most recent action that contains `time`.
fn action_at(time: f64, actions: &[ActionWindow]) -> Option<usize> {
    actions
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, action)| {
            (time >= action.started_ms && time <= action.started_ms + action.duration_ms)
                .then_some(index)
        })
}

/// Converts a duration to milliseconds for JSON output.
fn ms(value: std::time::Duration) -> f64 {
    value.as_secs_f64() * 1000.0
}
