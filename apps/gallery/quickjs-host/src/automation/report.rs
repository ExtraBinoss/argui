//! Recorded frame summaries and requested automation performance assertions.

use super::Step;
use argui_automation::{ActionWindow, Driver, frame_diagnostics, summarize_metrics};
use serde_json::{Value, json};

/// Computes an optional assertion against recorded Argui frame intervals.
/// `budget` contains only thresholds explicitly requested by the TS test.
///
/// # Errors
/// Returns a descriptive failure when a requested threshold is exceeded.
pub(super) fn assert_performance(driver: &Driver, budget: &Value) -> Result<(), String> {
    let frame_times = driver
        .frame_records()
        .iter()
        .map(|frame| frame.total_cpu().as_secs_f64() * 1000.0)
        .collect::<Vec<_>>();
    if frame_times.is_empty() {
        return Err("no completed frame records for performance assertion".into());
    }
    let summary = frame_summary(&frame_times, frame_budget_ms());
    for (key, measured) in [
        ("p95FrameTimeMsBelow", summary["p95Ms"].as_f64()),
        ("maxFrameTimeMsBelow", summary["maxMs"].as_f64()),
    ] {
        if let Some(limit) = budget[key].as_f64()
            && measured.is_some_and(|value| value >= limit)
        {
            return Err(format!(
                "{key} requested < {limit:.2} ms; measured {:.2} ms",
                measured.unwrap_or_default()
            ));
        }
    }
    if let Some(limit) = budget["framesOverBudgetAtMost"].as_u64()
        && summary["overBudget"]
            .as_u64()
            .is_some_and(|value| value > limit)
    {
        return Err(format!(
            "frames over {:.2} ms exceeded {limit}",
            frame_budget_ms()
        ));
    }
    Ok(())
}

/// Produces frame quantiles and budget count from engine intervals.
/// `intervals` are milliseconds and `budget_ms` is the configured threshold.
fn frame_summary(intervals: &[f64], budget_ms: f64) -> Value {
    let mut sorted = intervals.to_vec();
    sorted.sort_by(f64::total_cmp);
    let percentile = |p: f64| {
        sorted
            .get(((sorted.len() as f64 * p).ceil() as usize).saturating_sub(1))
            .copied()
    };
    json!({ "count": sorted.len(), "budgetMs": budget_ms,
        "p50Ms": percentile(0.50), "p95Ms": percentile(0.95),
        "p99Ms": percentile(0.99), "maxMs": sorted.last(),
        "overBudget": sorted.iter().filter(|&&value| value > budget_ms).count() })
}

/// Reads a positive finite frame budget in milliseconds for reports and assertions.
fn frame_budget_ms() -> f64 {
    std::env::var("ARGUI_FRAME_BUDGET_MS")
        .ok()
        .and_then(|text| text.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(16.67)
}

/// Builds a report that preserves missing adapter and frame metrics as null.
/// `driver` supplies raw engine records; `steps`, `adapter`, and `result`
/// describe actions, GPU identity, and final status.
pub(super) fn report(
    driver: &Driver,
    steps: &[Step],
    adapter: Option<Value>,
    result: &Result<(), String>,
    started_unix_ms: f64,
) -> Value {
    let budget = frame_budget_ms();
    let actions = steps
        .iter()
        .map(|step| ActionWindow {
            started_ms: step.started_ms,
            duration_ms: step.duration_ms,
        })
        .collect::<Vec<_>>();
    let diagnostics = frame_diagnostics(driver.frames(), driver.frame_records(), &actions);
    let mut slow = diagnostics.clone();
    slow.sort_by(|left, right| right.total_cpu_ms.total_cmp(&left.total_cpu_ms));
    slow.truncate(10);
    let frame_times = driver
        .frame_records()
        .iter()
        .map(|frame| frame.total_cpu().as_secs_f64() * 1000.0)
        .collect::<Vec<_>>();
    let intervals = driver
        .frame_records()
        .iter()
        .map(|frame| frame.interval.as_secs_f64() * 1000.0)
        .filter(|value| *value > 0.0)
        .collect::<Vec<_>>();
    json!({ "ok": result.is_ok(), "error": result.as_ref().err(),
        "startedUnixMs": started_unix_ms,
        "viewport": driver.viewport(), "steps": steps,
        "artifacts": steps.iter().filter_map(|step| step.artifact.as_deref()).collect::<Vec<_>>(),
        "adapter": adapter, "frames": driver.frames(),
        "metrics": summarize_metrics(&driver.metrics()),
        "frameDiagnostics": diagnostics,
        "slowFrames": slow,
        "frameMetric": "Argui headless CPU frame time; intervals describe time between committed updates",
        "frameSummary": frame_summary(&frame_times, budget),
        "frameIntervals": frame_summary(&intervals, budget),
        "limitations": ["WebView and OS-owned surfaces are not captured", "GPU adapter required only for screenshots"] })
}
