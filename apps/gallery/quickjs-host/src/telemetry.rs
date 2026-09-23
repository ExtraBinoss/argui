//! Bounded native-frame and JavaScript comparison measurements.

use argui_paint::RenderObjectId;
use argui_render::DamageMode;
use argui_runtime::{RuntimeEvent, WindowRuntimeEvent};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

static PRESENTATION_PROBE: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

/// Marks a large host batch's commit as the start of first-presentation timing.
/// This timestamp excludes JavaScript callback and native commit work.
pub(crate) fn mark_commit_for_presentation() {
    *PRESENTATION_PROBE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("probe lock") = Some(Instant::now());
}

/// Counters separating one-time mount work from live JavaScript transactions.
pub(crate) struct JsCounts {
    batches: Arc<AtomicU64>,
    operations: Arc<AtomicU64>,
    startup_batches: u64,
    startup_operations: u64,
}

impl JsCounts {
    /// Captures shared counters and their values after Animation Lab startup.
    /// `batches` and `operations` count emitted native transactions and operations.
    /// Returns a snapshot used to report work after the scene is ready.
    pub(crate) fn new(batches: Arc<AtomicU64>, operations: Arc<AtomicU64>) -> Self {
        let startup_batches = batches.load(Ordering::Relaxed);
        let startup_operations = operations.load(Ordering::Relaxed);
        Self {
            batches,
            operations,
            startup_batches,
            startup_operations,
        }
    }

    /// Returns the one-time mount and navigation counts.
    pub(crate) fn startup(&self) -> (u64, u64) {
        (self.startup_batches, self.startup_operations)
    }

    /// Reports cumulative post-startup transactions and JavaScript work.
    /// `deliveries` and `ticks` count live callbacks and scheduler iterations;
    /// `work` is time in QuickJS APIs, and `elapsed` is the full actor interval.
    pub(crate) fn report(&self, deliveries: u64, ticks: u64, work: Duration, elapsed: Duration) {
        let batches = self.batches.load(Ordering::Relaxed);
        let operations = self.operations.load(Ordering::Relaxed);
        eprintln!(
            "argui-comparison js_batches_after_startup={} js_operations_after_startup={} js_deliveries={} js_ticks={} js_work_ms={:.3} elapsed_ms={:.3}",
            batches.saturating_sub(self.startup_batches),
            operations.saturating_sub(self.startup_operations),
            deliveries,
            ticks,
            work.as_secs_f64() * 1000.0,
            elapsed.as_secs_f64() * 1000.0,
        );
    }
}

/// Records one native profile, sending complete bounded samples to an output thread.
/// `profiles` is the shared sample state; `event` is a runtime profile event.
/// `label` names the presentation under test.
pub(crate) fn observe_profile(
    profiles: &Arc<Mutex<ProfileSummary>>,
    event: &RuntimeEvent,
    label: &'static str,
) {
    if matches!(
        event,
        RuntimeEvent::RenderProfile(_)
            | RuntimeEvent::Window {
                event: WindowRuntimeEvent::RenderProfile(_),
                ..
            }
    ) && let Some(entered) = PRESENTATION_PROBE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("probe lock")
        .take()
    {
        let elapsed = entered.elapsed();
        std::thread::spawn(move || {
            eprintln!(
                "argui-comparison commit_to_first_render_ms={:.3}",
                elapsed.as_secs_f64() * 1000.0
            )
        });
    }
    let sample = profiles.lock().expect("profile lock").record(event);
    if let Some(sample) = sample {
        std::thread::spawn(move || sample.report(label));
    }
}

/// Reports a partial final sample after the event loop ends.
/// `profiles` holds any frames that did not fill a 120-frame sample.
/// `label` names the presentation under test.
pub(crate) fn report_remaining(profiles: &Arc<Mutex<ProfileSummary>>, label: &str) {
    let sample = std::mem::take(&mut profiles.lock().expect("profile lock").sample);
    if sample.frames > 0 {
        sample.report(label);
    }
}

/// Bounded native profile accumulator; each completed sample contains at most 120 frames.
#[derive(Default)]
pub(crate) struct ProfileSummary {
    sample: ProfileSample,
}

impl ProfileSummary {
    /// Records one profile event and returns a completed 120-frame sample.
    /// `event` is emitted by the same native renderer for both variants.
    fn record(&mut self, event: &RuntimeEvent) -> Option<ProfileSample> {
        match event {
            RuntimeEvent::AnimationProfile(profile)
            | RuntimeEvent::Window {
                event: WindowRuntimeEvent::AnimationProfile(profile),
                ..
            } => {
                self.sample.frames += 1;
                if !profile.frame_interval.is_zero() {
                    self.sample.intervals.push(profile.frame_interval);
                }
                self.sample.model_ns += profile.model_time.as_nanos();
                self.sample.tree_ns += profile.tree_time.as_nanos();
                self.sample.paint_ns += profile.paint_time.as_nanos();
            }
            RuntimeEvent::RenderProfile(profile)
            | RuntimeEvent::Window {
                event: WindowRuntimeEvent::RenderProfile(profile),
                ..
            } => {
                self.sample.renders += 1;
                self.sample.render_ns += profile.cpu_time.as_nanos();
                self.sample.render_max_ns =
                    self.sample.render_max_ns.max(profile.cpu_time.as_nanos());
                self.sample.damaged_pixels += u128::from(profile.damage.damaged_pixels);
                self.sample.viewport_pixels += u128::from(profile.viewport_pixels);
                self.sample.damage_full += u64::from(matches!(
                    profile.damage.mode,
                    DamageMode::Full | DamageMode::Seed
                ));
                self.sample.damage_partial +=
                    u64::from(matches!(profile.damage.mode, DamageMode::Partial));
                self.sample.cached_layers += profile.effects.cached_layers as u64;
                self.sample.offscreen_layers += profile.effects.offscreen_layers as u64;
                self.sample.filter_passes += profile.effects.filter_passes as u64;
                self.sample.draw_batches += profile.draw_batches as u64;
                self.sample.offscreen_pixels += u128::from(profile.effects.offscreen_pixels);
                self.sample.timestamp_queries = profile.adapter.timestamp_queries;
                if let Some(gpu) = &profile.gpu {
                    self.sample.gpu_frames += 1;
                    self.sample.gpu_total_ns += gpu.total.as_nanos();
                    for pass in &gpu.passes {
                        let key = (pass.label.clone(), pass.object);
                        let entry = self.sample.gpu_passes.entry(key).or_default();
                        entry.count += 1;
                        entry.duration_ns += pass.duration.as_nanos();
                        entry.pixels += u128::from(pass.pixels);
                    }
                }
                if self.sample.frames >= 120 {
                    return Some(std::mem::take(&mut self.sample));
                }
            }
            _ => {}
        }
        None
    }
}

/// One fixed-size native frame sample printed outside the UI callback.
#[derive(Default)]
struct ProfileSample {
    frames: u64,
    renders: u64,
    intervals: Vec<Duration>,
    model_ns: u128,
    tree_ns: u128,
    paint_ns: u128,
    render_ns: u128,
    render_max_ns: u128,
    damaged_pixels: u128,
    viewport_pixels: u128,
    damage_full: u64,
    damage_partial: u64,
    cached_layers: u64,
    offscreen_layers: u64,
    filter_passes: u64,
    draw_batches: u64,
    offscreen_pixels: u128,
    timestamp_queries: bool,
    gpu_frames: u64,
    gpu_total_ns: u128,
    gpu_passes: HashMap<(String, Option<RenderObjectId>), GpuPassSample>,
}

#[derive(Default)]
struct GpuPassSample {
    count: u64,
    duration_ns: u128,
    pixels: u128,
}

impl ProfileSample {
    /// Writes this aggregate sample, using `label` to identify the presentation.
    fn report(mut self, label: &str) {
        self.intervals.sort_unstable();
        let p95 = self
            .intervals
            .get(self.intervals.len().saturating_mul(95) / 100)
            .copied()
            .unwrap_or_default();
        let frames = u128::from(self.frames.max(1));
        let renders = u128::from(self.renders.max(1));
        eprintln!(
            "argui-comparison mode={label} frames={} renders={} interval_p95_ms={:.3} model_mean_ms={:.3} tree_mean_ms={:.3} paint_mean_ms={:.3} render_mean_ms={:.3} render_max_ms={:.3} damaged_fraction={:.3} damage_full={} damage_partial={} cached_layers_mean={:.2} offscreen_layers_mean={:.2} filter_passes_mean={:.2} draw_batches_mean={:.2} offscreen_pixels_mean={:.0} timestamp_queries={} gpu_frames={} gpu_total_mean_ms={:.3}",
            self.frames,
            self.renders,
            p95.as_secs_f64() * 1000.0,
            self.model_ns as f64 / frames as f64 / 1e6,
            self.tree_ns as f64 / frames as f64 / 1e6,
            self.paint_ns as f64 / frames as f64 / 1e6,
            self.render_ns as f64 / renders as f64 / 1e6,
            self.render_max_ns as f64 / 1e6,
            self.damaged_pixels as f64 / self.viewport_pixels.max(1) as f64,
            self.damage_full,
            self.damage_partial,
            self.cached_layers as f64 / renders as f64,
            self.offscreen_layers as f64 / renders as f64,
            self.filter_passes as f64 / renders as f64,
            self.draw_batches as f64 / renders as f64,
            self.offscreen_pixels as f64 / renders as f64,
            self.timestamp_queries,
            self.gpu_frames,
            self.gpu_total_ns as f64 / f64::from(self.gpu_frames.max(1) as u32) / 1e6,
        );
        let mut passes = self.gpu_passes.into_iter().collect::<Vec<_>>();
        passes.sort_unstable_by_key(|entry| std::cmp::Reverse(entry.1.duration_ns));
        for ((pass_label, object), sample) in passes.into_iter().take(12) {
            eprintln!(
                "argui-gpu-pass mode={label} label={pass_label} object={object:?} count={} mean_per_gpu_frame_ms={:.3} mean_pixels={:.0}",
                sample.count,
                sample.duration_ns as f64 / self.gpu_frames.max(1) as f64 / 1e6,
                sample.pixels as f64 / sample.count.max(1) as f64,
            );
        }
    }
}
