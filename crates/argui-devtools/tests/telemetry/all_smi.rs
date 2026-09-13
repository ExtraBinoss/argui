use argui_devtools::telemetry::{AllSmi, DeviceTelemetryProvider};
#[cfg(target_os = "linux")]
use web_time::Instant;

#[test]
fn snapshots_preserve_available_metrics_and_driver_details_without_inventing_missing_values() {
    let devices = AllSmi::decode(r#"{"schema":1,"gpus":[{"name":"Example GPU","uuid":"GPU-1","utilization":42.5,"used_memory":1048576,"total_memory":8388608,"temperature":65,"power_consumption":85.2,"frequency":1200,"fan_speed_rpm":1750,"performance_state":null,"nvlink_remote_devices":[]},{"name":"Integrated","uuid":"GPU-2","temperature":null,"utilization":-1}]}"#).unwrap();
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].utilization_percent, Some(42.5));
    assert_eq!(devices[0].used_memory_bytes, Some(1_048_576));
    assert_eq!(devices[0].frequency_mhz, Some(1200.0));
    assert!(
        devices[0]
            .details
            .contains(&("fan_speed_rpm".into(), "1750".into()))
    );
    assert!(
        devices[0]
            .details
            .contains(&("performance_state".into(), "Unavailable".into()))
    );
    assert_eq!(devices[1].temperature_celsius, None);
    assert_eq!(devices[1].utilization_percent, None);
    assert_eq!(devices[1].total_memory_bytes, None);
    for json in [
        "broken",
        "[]",
        r#"{"schema":2,"gpus":[]}"#,
        r#"{"schema":1,"errors":["timeout"]}"#,
        r#"{"schema":1,"gpus":[5]}"#,
    ] {
        assert!(AllSmi::decode(json).is_err());
    }
    assert!(
        AllSmi::decode(r#"{"schema":1,"gpus":[]}"#)
            .unwrap()
            .is_empty()
    );
    let many = serde_json::json!({ "schema": 1, "gpus": vec![serde_json::json!({}); 129] });
    assert!(AllSmi::decode(&many.to_string()).is_err());
}

#[test]
fn missing_optional_executable_returns_an_actionable_error() {
    let error = AllSmi::new("/nonexistent/argui-test-all-smi")
        .sample()
        .unwrap_err();
    assert!(error.contains("argui-test-all-smi"));
}

#[test]
fn detection_only_records_and_reader_failures_remain_visible_without_false_measurements() {
    let devices = AllSmi::decode(r#"{"schema":1,"errors":["secondary reader failed"],"gpus":[{"name":"Integrated","utilization":0,"used_memory":0,"total_memory":0,"temperature":0,"power_consumption":0,"frequency":0},{"name":"Idle","utilization":0,"total_memory":1024}]}"#).unwrap();
    let detected = &devices[0];
    assert_eq!(detected.utilization_percent, None);
    assert_eq!(detected.used_memory_bytes, None);
    assert_eq!(detected.total_memory_bytes, None);
    assert_eq!(detected.temperature_celsius, None);
    assert_eq!(detected.power_watts, None);
    assert_eq!(detected.frequency_mhz, None);
    assert!(
        detected
            .details
            .iter()
            .any(|(key, value)| key == "Reader warnings"
                && value.contains("secondary reader failed"))
    );
    assert!(
        detected
            .details
            .contains(&("utilization".into(), "0".into()))
    );
    assert_eq!(devices[1].utilization_percent, Some(0.0));
    assert_eq!(devices[1].total_memory_bytes, Some(1024));
    assert!(
        AllSmi::decode(r#"{"schema":1,"gpus":[],"errors":["reader failed"]}"#)
            .unwrap_err()
            .contains("reader failed")
    );
}

#[cfg(target_os = "linux")]
#[test]
fn command_adapter_reads_json_checks_status_and_bounds_timeout_and_output() {
    use std::{os::unix::fs::PermissionsExt, time::Duration};
    let folder = std::env::temp_dir().join(format!("argui-all-smi-test-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let binary = folder.join("sensor");
    let write = |body: &str| {
        std::fs::write(&binary, format!("#!/usr/bin/python3\n{body}\n")).unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    };
    write("print('{\"schema\":1,\"gpus\":[]}')");
    assert!(AllSmi::new(&binary).sample().unwrap().is_empty());
    write("raise SystemExit(7)");
    assert!(
        AllSmi::new(&binary)
            .sample()
            .unwrap_err()
            .contains("exited")
    );
    write("import time\ntime.sleep(5)");
    let start = Instant::now();
    assert!(
        AllSmi::new(&binary)
            .timeout(Duration::from_millis(100))
            .sample()
            .unwrap_err()
            .contains("timed out")
    );
    assert!(start.elapsed() < Duration::from_secs(2));
    write("import sys\nsys.stdout.buffer.write(b'x' * (2 * 1024 * 1024 + 1))");
    assert!(AllSmi::new(&binary).sample().unwrap_err().contains("limit"));
    std::fs::remove_dir_all(folder).unwrap();
}
