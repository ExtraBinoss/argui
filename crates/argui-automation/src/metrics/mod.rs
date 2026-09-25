//! Structured measurements for automation reports.

mod frame;
#[cfg(feature = "desktop-metrics")]
mod process;
mod trace;

pub use frame::{ActionWindow, FrameDiagnostic, frame_diagnostics};
#[cfg(feature = "desktop-metrics")]
pub use process::{ProcessSample, ProcessSampler};
pub use trace::{MetricReport, summarize_metrics};
