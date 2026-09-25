#[path = "metrics/frame.rs"]
mod frame;
#[path = "metrics/trace.rs"]
mod trace;

#[cfg(feature = "desktop-metrics")]
use std::time::Duration;

#[cfg(feature = "desktop-metrics")]
use argui_automation::ProcessSampler;

#[cfg(feature = "desktop-metrics")]
#[test]
fn sampler_records_the_current_process_and_resident_memory() {
    let sampler = ProcessSampler::start(std::process::id(), Duration::from_millis(100));
    std::thread::sleep(Duration::from_millis(550));
    let rows = sampler.finish();
    assert!(
        rows.iter()
            .any(|row| row.pid == std::process::id() && row.rss_bytes.unwrap_or(0) > 0)
    );
    assert!(
        rows.iter()
            .any(|row| row.pid == std::process::id() && row.cpu_percent.is_some())
    );
}
