//! Changed failures and recovery notifications for retained GPU canvases.
use super::{
    GpuCanvasDiagnostic, GpuCanvasDiagnosticKind, GpuCanvasFailureStage,
    cache::CanvasKey,
    pipeline::{CanvasGpu, FailureRecord},
};
use argui_paint::GpuCanvasPrimitive;
#[cfg_attr(coverage_nightly, coverage(off))]
impl CanvasGpu {
    /// Records and emits a changed failure for retained `key` exactly once.
    pub(super) fn record_failure(
        &mut self,
        key: CanvasKey,
        canvas: &GpuCanvasPrimitive,
        label: String,
        stage: GpuCanvasFailureStage,
        message: String,
    ) {
        if self
            .failures
            .get(&key)
            .is_some_and(|current| current.stage == stage && current.message == message)
        {
            return;
        }
        let readable = format!("GPU canvas '{label}' failed during {stage:?}: {message}");
        eprintln!("{readable}");
        self.failures.insert(
            key,
            FailureRecord {
                stage,
                label: label.clone(),
                message: message.clone(),
            },
        );
        self.diagnostics.push(GpuCanvasDiagnostic {
            kind: GpuCanvasDiagnosticKind::Failed,
            stage,
            label,
            canvas: canvas.canvas,
            object: canvas.object,
            slot: canvas.slot,
            message: readable,
        });
    }

    /// Emits one recovery when `key` previously had a recorded failure.
    pub(super) fn record_recovery(
        &mut self,
        key: CanvasKey,
        canvas: &GpuCanvasPrimitive,
        fallback_label: &str,
    ) {
        let Some(previous) = self.failures.remove(&key) else {
            return;
        };
        let label = if previous.label.is_empty() {
            fallback_label.to_owned()
        } else {
            previous.label
        };
        self.diagnostics.push(GpuCanvasDiagnostic {
            kind: GpuCanvasDiagnosticKind::Recovered,
            stage: previous.stage,
            label: label.clone(),
            canvas: canvas.canvas,
            object: canvas.object,
            slot: canvas.slot,
            message: format!("GPU canvas '{label}' recovered"),
        });
    }
}
