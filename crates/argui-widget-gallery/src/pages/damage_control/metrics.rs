use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

use argui::{
    render::{DamageMode, RenderProfile},
    ui::{Element, Sides, length},
    widgets::WidgetTheme,
};

use crate::app::text;

const SAMPLE_LIMIT: usize = 120;

/// Shared renderer measurements consumed by the damage-control showcase.
pub(crate) type DamageTelemetryHandle = Rc<RefCell<DamageTelemetry>>;

#[derive(Clone, Copy, Debug, Default)]
struct DamageSample {
    mode: DamageMode,
    regions: usize,
    damaged_pixels: u64,
    viewport_pixels: u64,
    retained_bytes: u64,
    cpu_time: Duration,
    gpu_time: Option<Duration>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct DamageMetrics {
    pub(super) samples: usize,
    pub(super) mode: DamageMode,
    pub(super) regions: usize,
    pub(super) damaged_pixels: u64,
    pub(super) viewport_pixels: u64,
    pub(super) retained_bytes: u64,
    pub(super) average_ratio: f64,
    pub(super) average_cpu_ms: f64,
    pub(super) average_gpu_ms: Option<f64>,
}

/// Bounded rolling history of renderer profiles for the live comparison page.
#[derive(Debug, Default)]
pub(crate) struct DamageTelemetry {
    samples: VecDeque<DamageSample>,
}

impl DamageTelemetry {
    /// Creates an empty handle suitable for sharing with the runtime callback.
    #[must_use]
    pub(crate) fn handle() -> DamageTelemetryHandle {
        Rc::new(RefCell::new(Self::default()))
    }

    /// Records the relevant counters from one completed renderer frame.
    ///
    /// `profile` is the renderer profile emitted after presentation.
    pub(crate) fn record(&mut self, profile: &RenderProfile) {
        self.samples.push_back(DamageSample {
            mode: profile.damage.mode,
            regions: profile.damage.regions,
            damaged_pixels: profile.damage.damaged_pixels,
            viewport_pixels: profile.viewport_pixels,
            retained_bytes: profile.damage.retained_bytes,
            cpu_time: profile.cpu_time,
            gpu_time: profile.gpu.as_ref().map(|gpu| gpu.total),
        });
        if self.samples.len() > SAMPLE_LIMIT {
            self.samples.pop_front();
        }
    }

    /// Clears all samples so two renderer modes start with independent history.
    pub(crate) fn clear(&mut self) {
        self.samples.clear();
    }

    /// Summarizes the current rolling history for presentation.
    pub(super) fn metrics(&self) -> DamageMetrics {
        let Some(latest) = self.samples.back().copied() else {
            return DamageMetrics::default();
        };
        let count = self.samples.len() as f64;
        let average_ratio = self
            .samples
            .iter()
            .map(|sample| {
                if sample.viewport_pixels == 0 {
                    0.0
                } else {
                    sample.damaged_pixels as f64 / sample.viewport_pixels as f64
                }
            })
            .sum::<f64>()
            / count;
        let average_cpu_ms = self
            .samples
            .iter()
            .map(|sample| sample.cpu_time.as_secs_f64() * 1_000.0)
            .sum::<f64>()
            / count;
        let (gpu_total, gpu_count) = self
            .samples
            .iter()
            .filter_map(|sample| sample.gpu_time)
            .fold((0.0, 0_usize), |(total, count), duration| {
                (total + duration.as_secs_f64() * 1_000.0, count + 1)
            });
        let average_gpu_ms = (gpu_count > 0).then(|| gpu_total / gpu_count as f64);
        DamageMetrics {
            samples: self.samples.len(),
            mode: latest.mode,
            regions: latest.regions,
            damaged_pixels: latest.damaged_pixels,
            viewport_pixels: latest.viewport_pixels,
            retained_bytes: latest.retained_bytes,
            average_ratio,
            average_cpu_ms,
            average_gpu_ms,
        }
    }
}

/// Chooses a measured value once renderer samples are available.
pub(super) fn when_ready(ready: bool, value: String, pending: &str) -> String {
    if ready { value } else { pending.into() }
}

/// Builds one compact renderer-metric card.
pub(super) fn metric_card(
    label: &str,
    value: String,
    detail: impl Into<String>,
    theme: &WidgetTheme,
) -> Element {
    Element::column([
        text(label, 11.0, theme.muted_foreground, 650),
        text(value, 22.0, theme.foreground, 720),
        text(detail, 11.0, theme.muted_foreground, 450),
    ])
    .width(length(190.0))
    .min_width(length(150.0))
    .grow(1.0)
    .padding(Sides::length(14.0))
    .gap(4.0)
    .background(theme.card)
    .border(argui::paint::Border::all(1.0, theme.border))
    .radius(argui::paint::CornerRadii::all(10.0))
}

/// Formats the latest damaged and viewport pixel counts compactly.
pub(super) fn format_pixels(damaged: u64, viewport: u64) -> String {
    format!(
        "{:.2}M / {:.2}M px",
        damaged as f64 / 1_000_000.0,
        viewport as f64 / 1_000_000.0
    )
}

/// Returns a concise label for one renderer damage decision.
pub(super) const fn damage_mode(mode: DamageMode) -> &'static str {
    match mode {
        DamageMode::Full => "Full",
        DamageMode::Seed => "Seed",
        DamageMode::Partial => "Partial",
        DamageMode::Reused => "Reused",
    }
}
