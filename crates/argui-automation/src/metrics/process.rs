//! Cross-platform desktop process sampling with explicit missing values.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// One timestamped CPU and resident-memory sample for a process.
#[derive(Clone, Debug, Serialize)]
pub struct ProcessSample {
    /// Milliseconds since monitoring began.
    pub timestamp_ms: f64,
    /// Operating-system process identifier.
    pub pid: u32,
    /// Parent process identifier when reported by the operating system.
    pub parent_pid: Option<u32>,
    /// Process CPU percent; 100 means one fully used logical core.
    pub cpu_percent: Option<f32>,
    /// Resident memory bytes, or `None` if the counter was unavailable.
    pub rss_bytes: Option<u64>,
}

/// Background sampler covering a launched host and its descendants.
pub struct ProcessSampler {
    stopped: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<Vec<ProcessSample>>>,
}

impl ProcessSampler {
    /// Starts bounded-interval sampling for `pid` and child processes.
    /// `interval` should be at least sysinfo's CPU update minimum.
    /// The sampler begins immediately; call [`Self::finish`] after host exit.
    pub fn start(pid: u32, interval: Duration) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&stopped);
        let thread = thread::spawn(move || {
            sample_loop(
                pid,
                interval.max(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL),
                &signal,
            )
        });
        Self {
            stopped,
            thread: Some(thread),
        }
    }

    /// Stops monitoring and returns samples in timestamp order.
    /// Returns an empty vector if the sampling thread panicked.
    pub fn finish(mut self) -> Vec<ProcessSample> {
        self.stopped.store(true, Ordering::Relaxed);
        self.thread
            .take()
            .and_then(|thread| thread.join().ok())
            .unwrap_or_default()
    }
}

/// Samples the host and all currently visible descendants until stopped.
fn sample_loop(pid: u32, interval: Duration, stopped: &AtomicBool) -> Vec<ProcessSample> {
    let root = Pid::from_u32(pid);
    let started = Instant::now();
    let mut system = System::new();
    let mut samples = Vec::new();
    let mut iteration = 0;
    loop {
        let kind = ProcessRefreshKind::nothing()
            .with_cpu()
            .with_memory()
            .without_tasks();
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, kind);
        let mut relevant = vec![root];
        loop {
            let previous = relevant.len();
            for (child, process) in system.processes() {
                if process
                    .parent()
                    .is_some_and(|parent| relevant.contains(&parent))
                    && !relevant.contains(child)
                {
                    relevant.push(*child);
                }
            }
            if relevant.len() == previous {
                break;
            }
        }
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        for current in relevant {
            if let Some(process) = system.process(current) {
                samples.push(ProcessSample {
                    timestamp_ms: elapsed,
                    pid: current.as_u32(),
                    parent_pid: process.parent().map(Pid::as_u32),
                    cpu_percent: (iteration > 0).then(|| process.cpu_usage()),
                    rss_bytes: Some(process.memory()),
                });
            }
        }
        iteration += 1;
        if stopped.load(Ordering::Relaxed) {
            break;
        }
        let deadline = Instant::now() + interval;
        while Instant::now() < deadline && !stopped.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(10));
        }
    }
    samples
}
