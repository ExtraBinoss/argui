use std::time::Duration;

#[cfg(all(feature = "all-smi", not(target_arch = "wasm32")))]
mod all_smi;
#[cfg(not(target_arch = "wasm32"))]
mod process;
#[cfg(not(target_arch = "wasm32"))]
mod worker;
#[cfg(all(feature = "all-smi", not(target_arch = "wasm32")))]
pub use all_smi::AllSmi;

/// Physical device counters. They describe the whole device, independently of
/// Argui's GPU pass timings and texture allocations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeviceTelemetry {
    pub name: String,
    pub identifier: String,
    pub utilization_percent: Option<f64>,
    pub used_memory_bytes: Option<u64>,
    pub total_memory_bytes: Option<u64>,
    pub temperature_celsius: Option<f64>,
    pub power_watts: Option<f64>,
    pub frequency_mhz: Option<f64>,
    /// Additional driver counters and metadata, retaining their reported units.
    pub details: Vec<(String, String)>,
}

/// Implementations run on a dedicated worker, only after sensor collection is
/// enabled. Bound blocking work and report unsupported metrics as `None`.
pub trait DeviceTelemetryProvider: Send + 'static {
    fn name(&self) -> &str;
    fn sample(&mut self) -> Result<Vec<DeviceTelemetry>, String>;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MemoryCategory {
    pub name: String,
    pub resident_bytes: u64,
    pub details: Vec<(String, u64)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessTelemetry {
    pub resident_bytes: u64,
    pub virtual_bytes: u64,
    pub private_bytes: Option<u64>,
    pub proportional_bytes: Option<u64>,
    /// 100% represents one fully occupied logical CPU; values can exceed 100%.
    /// The first sample is unavailable because it has no preceding interval.
    pub cpu_percent: Option<f32>,
    /// When present, these measured resident categories sum to `resident_bytes`.
    pub categories: Vec<MemoryCategory>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TelemetrySnapshot {
    pub process: Option<ProcessTelemetry>,
    pub devices: Vec<DeviceTelemetry>,
    pub errors: Vec<String>,
    pub collection_time: Duration,
}

pub(crate) struct Telemetry {
    pub snapshot: TelemetrySnapshot,
    pub devices_enabled: bool,
    pub source: Option<String>,
    pub samples: u64,
    pub expanded: std::collections::BTreeSet<String>,
    elapsed: Duration,
    #[cfg(not(target_arch = "wasm32"))]
    worker: worker::Worker,
}

impl Default for Telemetry {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let worker = worker::Worker::default();
        Self {
            snapshot: TelemetrySnapshot::default(),
            devices_enabled: false,
            #[cfg(not(target_arch = "wasm32"))]
            source: worker.source(),
            #[cfg(target_arch = "wasm32")]
            source: None,
            samples: 0,
            expanded: Default::default(),
            elapsed: Duration::from_secs(1),
            #[cfg(not(target_arch = "wasm32"))]
            worker,
        }
    }
}

impl Telemetry {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn provider(&mut self, provider: impl DeviceTelemetryProvider) {
        self.source = Some(provider.name().into());
        self.worker = worker::Worker::new(Box::new(provider));
        self.snapshot.devices.clear();
    }

    pub fn advance(&mut self, elapsed: Duration) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut changed = false;
            if let Some(snapshot) = self.worker.poll() {
                self.snapshot = snapshot;
                if !self.devices_enabled {
                    self.snapshot.devices.clear();
                }
                self.samples = self.samples.wrapping_add(1);
                changed = true;
            }
            self.elapsed = self.elapsed.saturating_add(elapsed);
            if self.elapsed >= Duration::from_secs(1) && self.worker.request(self.devices_enabled) {
                self.elapsed = Duration::ZERO;
            }
            changed
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.elapsed = self.elapsed.saturating_add(elapsed);
            if self.elapsed < Duration::from_secs(1) {
                return false;
            }
            self.elapsed = Duration::ZERO;
            // Renderer-owned capacity counters remain available without OS APIs.
            true
        }
    }
}
