use super::{DeviceTelemetry, DeviceTelemetryProvider};
use std::{
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};
use web_time::Instant;

/// Optional adapter for the separately installed `all-smi` executable's schema-1
/// JSON snapshots. No shell, server, elevated permissions or linked GPU SDKs.
#[derive(Clone, Debug)]
pub struct AllSmi {
    executable: PathBuf,
    timeout: Duration,
}

impl Default for AllSmi {
    fn default() -> Self {
        Self::new("all-smi")
    }
}

impl AllSmi {
    /// Creates a provider that launches the executable at `executable`.
    #[must_use]
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            timeout: Duration::from_secs(2),
        }
    }

    /// Sets the process timeout, clamped to the supported 100 ms–10 s interval.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.clamp(Duration::from_millis(100), Duration::from_secs(10));
        self
    }

    /// Decode the documented snapshot format; useful for recorded sensor feeds.
    /// `json` is a schema-1 snapshot from `all-smi`.
    ///
    /// # Errors
    ///
    /// Returns a diagnostic when the JSON, schema, or device records are invalid,
    /// or when no usable device snapshot is present.
    pub fn decode(json: &str) -> Result<Vec<DeviceTelemetry>, String> {
        let snapshot: serde_json::Value =
            serde_json::from_str(json).map_err(|error| error.to_string())?;
        if snapshot.get("schema").and_then(serde_json::Value::as_u64) != Some(1) {
            return Err("Unsupported all-smi snapshot schema".into());
        }
        let gpus = snapshot
            .get("gpus")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                format!(
                    "No GPU snapshot reported{}",
                    snapshot
                        .get("errors")
                        .map_or(String::new(), |errors| format!(": {errors}"))
                )
            })?;
        if gpus.len() > 128 {
            return Err("Too many devices in all-smi snapshot".into());
        }
        let errors = snapshot
            .get("errors")
            .and_then(serde_json::Value::as_array)
            .filter(|errors| !errors.is_empty());
        if gpus.is_empty()
            && let Some(errors) = errors
        {
            return Err(format!("all-smi readers: {}", serde_json::json!(errors)));
        }
        let mut devices = Vec::with_capacity(gpus.len());
        for gpu in gpus {
            let fields = gpu.as_object().ok_or("Invalid GPU record")?;
            let text = |name| {
                fields
                    .get(name)
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_owned()
            };
            let number = |name| {
                fields
                    .get(name)
                    .and_then(serde_json::Value::as_f64)
                    .filter(|value| value.is_finite() && *value >= 0.0)
            };
            let bytes = |name| fields.get(name).and_then(serde_json::Value::as_u64);
            // Some readers emit zero for every unsupported sensor. Preserve their
            // raw output, but do not present a detection-only record as measurements.
            let readings = [
                "utilization",
                "used_memory",
                "total_memory",
                "temperature",
                "power_consumption",
                "frequency",
            ]
            .into_iter()
            .any(|name| number(name).is_some_and(|value| value > 0.0));
            let mut details: Vec<_> = fields
                .iter()
                .filter(|(name, _)| {
                    !["hostname", "host_id", "time", "instance"].contains(&name.as_str())
                })
                .take(256)
                .map(|(name, value)| {
                    (
                        name.clone(),
                        match value {
                            serde_json::Value::String(value) => value.clone(),
                            serde_json::Value::Null => "Unavailable".into(),
                            value => value.to_string(),
                        },
                    )
                })
                .collect();
            if !readings {
                details.push(("Availability".into(), "No nonzero sensor reading; values cannot be distinguished from unsupported sensors".into()));
            }
            if let Some(errors) = errors {
                details.push((
                    "Reader warnings".into(),
                    serde_json::json!(errors).to_string(),
                ));
            }
            devices.push(DeviceTelemetry {
                name: text("name"),
                identifier: text("uuid"),
                utilization_percent: number("utilization").filter(|_| readings),
                used_memory_bytes: bytes("used_memory").filter(|_| readings),
                total_memory_bytes: bytes("total_memory").filter(|value| *value > 0),
                temperature_celsius: number("temperature").filter(|_| readings),
                power_watts: number("power_consumption").filter(|_| readings),
                frequency_mhz: number("frequency").filter(|_| readings),
                details,
            });
        }
        Ok(devices)
    }
}

impl DeviceTelemetryProvider for AllSmi {
    fn name(&self) -> &str {
        "all-smi"
    }

    fn sample(&mut self) -> Result<Vec<DeviceTelemetry>, String> {
        let mut command = Command::new(&self.executable);
        command
            .args([
                "snapshot",
                "--format",
                "json",
                "--include",
                "gpu",
                "--samples",
                "1",
                "--pretty=false",
                "--timeout-ms",
                "1000",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        let mut child = command
            .spawn()
            .map_err(|error| format!("{}: {error}", self.executable.display()))?;
        let stdout = child.stdout.take().ok_or("all-smi stdout unavailable")?;
        const LIMIT: u64 = 2 * 1024 * 1024;
        let reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout
                .take(LIMIT + 1)
                .read_to_end(&mut bytes)
                .map(|_| bytes)
        });
        let start = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) if start.elapsed() < self.timeout => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                result => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(match result {
                        Err(error) => error.to_string(),
                        _ => "GPU collection timed out".into(),
                    });
                }
            }
        };
        let bytes = reader
            .join()
            .map_err(|_| "GPU output reader stopped")?
            .map_err(|error| error.to_string())?;
        let status = status?;
        if !status.success() {
            return Err(format!("all-smi exited with {status}"));
        }
        if bytes.len() > LIMIT as usize {
            return Err("GPU snapshot exceeded the 2 MiB limit".into());
        }
        let json = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
        Self::decode(json)
    }
}
