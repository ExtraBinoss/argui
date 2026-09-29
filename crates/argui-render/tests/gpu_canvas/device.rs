//! Device negotiation must precede lazy GPU canvas allocation.
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct DeviceFactory(Arc<AtomicUsize>, Arc<AtomicUsize>);
impl GpuCanvasFactory for DeviceFactory {
    fn device_ready(&self, context: &GpuCanvasDeviceContext<'_>) {
        assert_eq!(context.features(), context.device().features());
        assert_eq!(context.limits(), &context.device().limits());
        assert!(context.device_generation() > 0);
        self.0.fetch_add(1, Ordering::SeqCst);
    }
    fn create(
        &self,
        _: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        self.1.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(super::Renderer))
    }
}
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn device_is_announced_before_any_scene_or_canvas_allocation() {
    let ready = Arc::new(AtomicUsize::new(0));
    let creates = Arc::new(AtomicUsize::new(0));
    let registration = GpuCanvasRegistration::new(
        "device.ready",
        DeviceFactory(Arc::clone(&ready), Arc::clone(&creates)),
    );
    let config =
        RendererConfig::default().gpu_canvases(GpuCanvasRegistry::new([registration]).unwrap());
    let renderer = pollster::block_on(SurfaceRenderer::new_offscreen(16, 16, config)).unwrap();
    assert_eq!(ready.load(Ordering::SeqCst), 1);
    assert_eq!(creates.load(Ordering::SeqCst), 0);
    assert_eq!(renderer.gpu_canvas_stats().entries, 0);
}
