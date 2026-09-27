#![cfg(test)]

use super::process_report;
use argui_automation::ProcessSample;
use serde_json::json;

#[test]
/// Process diagnostics retain child, peak memory, and each timeline phase.
fn process_report_labels_activity_and_missing_counters() {
    let report = json!({
        "startedUnixMs": 1_000.0,
        "steps": [
            {"started_ms": 10.0, "duration_ms": 5.0},
            {"started_ms": 30.0, "duration_ms": 5.0}
        ]
    });
    let samples = [
        (0.0, 7, Some(5.0), Some(20)),
        (12.0, 8, Some(12.0), Some(30)),
        (20.0, 8, Some(0.0), Some(25)),
        (40.0, 7, Some(1.0), None),
    ]
    .map(
        |(timestamp_ms, pid, cpu_percent, rss_bytes)| ProcessSample {
            timestamp_ms,
            pid,
            parent_pid: None,
            cpu_percent,
            rss_bytes,
        },
    );
    let result = process_report(7, 1_000.0, &report, samples.to_vec());
    let phases = result["samples"]
        .as_array()
        .unwrap()
        .iter()
        .map(|sample| sample["phase"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(phases, ["startup", "interaction", "idle", "teardown"]);
    assert_eq!(result["childPids"], json!([8]));
    assert_eq!(result["peakRssBytes"], 30);
    assert!(result["unavailable"].is_null());

    let empty = process_report(7, 1_000.0, &json!({}), Vec::new());
    assert!(
        empty["unavailable"]
            .as_str()
            .unwrap()
            .contains("before the sampler")
    );
    let absent = process_report(
        7,
        1_000.0,
        &json!({}),
        vec![ProcessSample {
            timestamp_ms: 0.0,
            pid: 7,
            parent_pid: None,
            cpu_percent: None,
            rss_bytes: None,
        }],
    );
    assert!(
        absent["unavailable"]
            .as_str()
            .unwrap()
            .contains("second CPU sample")
    );
}
