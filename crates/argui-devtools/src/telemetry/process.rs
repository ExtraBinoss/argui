use super::ProcessTelemetry;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use web_time::Instant;

#[cfg(target_os = "linux")]
mod memory;

#[derive(Default)]
pub(super) struct ProcessSampler {
    system: System,
    previous: Option<Instant>,
}

impl ProcessSampler {
    pub fn sample(&mut self) -> Result<ProcessTelemetry, String> {
        let pid = Pid::from_u32(std::process::id());
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing()
                .without_tasks()
                .with_memory()
                .with_cpu(),
        );
        let process = self
            .system
            .process(pid)
            .ok_or("Process metrics unavailable on this platform")?;
        let now = Instant::now();
        let cpu_percent = self
            .previous
            .filter(|last| now.duration_since(*last).as_secs() < 4)
            .map(|_| process.cpu_usage())
            .filter(|value| value.is_finite());
        self.previous = Some(now);
        let metrics = ProcessTelemetry {
            resident_bytes: process.memory(),
            virtual_bytes: process.virtual_memory(),
            cpu_percent,
            ..Default::default()
        };
        #[cfg(target_os = "linux")]
        let metrics = memory::enrich(metrics);
        Ok(metrics)
    }
}
