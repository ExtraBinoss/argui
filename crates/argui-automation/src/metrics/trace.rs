//! Serializes nested engine spans and aggregate phase totals.

use std::collections::BTreeMap;

use argui_core::{MetricKind, MetricTrace};
use serde::Serialize;

/// One trace event with an explicit unit and optional enclosing scope.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricEntry {
    /// Stable event identifier within this trace.
    pub id: usize,
    /// The containing span, when one exists.
    pub parent_id: Option<usize>,
    /// Stable phase or counter name.
    pub name: &'static str,
    /// Either `span` or `gauge`.
    pub kind: &'static str,
    /// Monotonic milliseconds since the automation driver was created.
    pub started_ms: f64,
    /// Elapsed wall-clock milliseconds for spans; absent for gauges.
    pub duration_ms: Option<f64>,
    /// A gauge's numeric sample; absent for spans.
    pub value: Option<f64>,
    /// A gauge's unit; absent for spans.
    pub unit: Option<&'static str>,
}

/// Aggregate completed span durations for one phase name.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseSummary {
    /// Number of completed scopes with this name.
    pub count: usize,
    /// Sum of scope durations, including overlapping child scopes only in their own summaries.
    pub total_ms: f64,
    /// Nearest-rank 95th percentile of scope duration.
    pub p95_ms: f64,
    /// Largest scope duration.
    pub max_ms: f64,
}

/// Machine-readable trace with named phase summaries.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricReport {
    /// The clock shared by events, frame timestamps, and test steps.
    pub clock: &'static str,
    /// Nested events in start order.
    pub events: Vec<MetricEntry>,
    /// Span duration summaries, keyed by phase name.
    pub phases: BTreeMap<&'static str, PhaseSummary>,
}

/// Takes a completed snapshot of `trace` for machine-readable reports.
/// Open scopes are omitted from aggregate totals until their guards drop.
#[must_use]
pub fn summarize_metrics(trace: &MetricTrace) -> MetricReport {
    let events = trace.events();
    let mut durations = BTreeMap::<&'static str, Vec<f64>>::new();
    let events = events
        .into_iter()
        .map(|event| match event.kind {
            MetricKind::Span => {
                durations.entry(event.name).or_default().push(event.value);
                MetricEntry {
                    id: event.id,
                    parent_id: event.parent_id,
                    name: event.name,
                    kind: "span",
                    started_ms: event.started_ms,
                    duration_ms: Some(event.value),
                    value: None,
                    unit: None,
                }
            }
            MetricKind::Gauge { unit } => MetricEntry {
                id: event.id,
                parent_id: event.parent_id,
                name: event.name,
                kind: "gauge",
                started_ms: event.started_ms,
                duration_ms: None,
                value: Some(event.value),
                unit: Some(unit),
            },
        })
        .collect();
    let phases = durations
        .into_iter()
        .map(|(name, mut values)| {
            values.sort_by(f64::total_cmp);
            let count = values.len();
            let p95 = (count as f64 * 0.95).ceil() as usize;
            (
                name,
                PhaseSummary {
                    count,
                    total_ms: values.iter().sum(),
                    p95_ms: values[p95.saturating_sub(1)],
                    max_ms: values[count - 1],
                },
            )
        })
        .collect();
    MetricReport {
        clock: "monotonic milliseconds since automation driver creation",
        events,
        phases,
    }
}
