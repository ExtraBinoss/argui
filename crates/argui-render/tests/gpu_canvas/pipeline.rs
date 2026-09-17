use super::*;

/// Exercises retained GPU-canvas callback, extent, diagnostics, effect and surface behavior.
pub(super) fn exercise(
    renderer: &mut SurfaceRenderer,
    canvas_id: GpuCanvasId,
    probe: &Arc<CanvasProbe>,
    retry_id: GpuCanvasId,
    retry_probe: &Arc<CanvasProbe>,
    window: &Arc<Window>,
    render: &mut impl FnMut(&mut SurfaceRenderer, &DisplayList) -> Result<(), RendererError>,
) {
    let first = canvas_list(canvas_id, 1, Size::new(48.0, 32.0), 0, false);
    render(renderer, &first).unwrap();
    assert_eq!(probe.creates.load(Ordering::Relaxed), 1);
    assert_eq!(probe.renders.load(Ordering::Relaxed), 1);
    assert_eq!(probe.extent.load(Ordering::Relaxed), (48_u64 << 32) | 32);
    assert_eq!(renderer.gpu_canvas_stats().renders_this_frame, 1);
    assert_eq!(renderer.gpu_canvas_stats().entries, 1);
    assert_eq!(renderer.gpu_canvas_stats().allocated_bytes, 48 * 32 * 4);

    render(renderer, &first).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), 1);
    assert_eq!(renderer.gpu_canvas_stats().hits_this_frame, 1);

    let changed = canvas_list(canvas_id, 2, Size::new(48.0, 32.0), 0, false);
    render(renderer, &changed).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), 2);

    let resized = canvas_list(canvas_id, 2, Size::new(64.0, 40.0), 0, false);
    render(renderer, &resized).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), 3);
    assert_eq!(probe.extent.load(Ordering::Relaxed), (64_u64 << 32) | 40);

    let mut supersampled = canvas_primitive(canvas_id, 2, Size::new(32.0, 20.0), 1);
    supersampled.resolution_scale = 2.0;
    let mut supersampled_list = DisplayList::new();
    supersampled_list.push_gpu_canvas(supersampled);
    render(renderer, &supersampled_list).unwrap();
    assert_eq!(probe.extent.load(Ordering::Relaxed), (64_u64 << 32) | 40);
    assert!(renderer.gpu_canvas_stats().allocated_bytes <= 16 * 1024);

    let before_lru = probe.renders.load(Ordering::Relaxed);
    render(renderer, &changed).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_lru + 1);

    let before_empty = probe.renders.load(Ordering::Relaxed);
    render(
        renderer,
        &canvas_list(canvas_id, 2, Size::new(0.0, 0.0), 2, false),
    )
    .unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_empty);

    let mut invalid = canvas_primitive(canvas_id, 3, Size::new(32.0, 20.0), 3);
    invalid.resolution_scale = f32::NAN;
    assert!(failure_message(renderer, invalid, render).contains("invalid resolution scale"));

    let mut negative_scale = canvas_primitive(canvas_id, 3, Size::new(32.0, 20.0), 31);
    negative_scale.resolution_scale = -1.0;
    assert!(failure_message(renderer, negative_scale, render).contains("invalid resolution scale"));

    let mut invalid_extent = canvas_primitive(canvas_id, 3, Size::new(-1.0, 20.0), 4);
    assert!(failure_message(renderer, invalid_extent.clone(), render).contains("invalid logical"));
    invalid_extent.slot = 5;
    invalid_extent.bounds.size.width = f32::NAN;
    assert!(failure_message(renderer, invalid_extent, render).contains("invalid logical"));

    let invalid_height = canvas_primitive(canvas_id, 3, Size::new(20.0, f32::NAN), 32);
    assert!(failure_message(renderer, invalid_height, render).contains("invalid logical"));
    let negative_height = canvas_primitive(canvas_id, 3, Size::new(20.0, -1.0), 33);
    assert!(failure_message(renderer, negative_height, render).contains("invalid logical"));

    let before_zero_height = probe.renders.load(Ordering::Relaxed);
    render(
        renderer,
        &canvas_list(canvas_id, 3, Size::new(20.0, 0.0), 34, false),
    )
    .unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_zero_height);

    let mut overflow = canvas_primitive(canvas_id, 3, Size::new(32.0, 20.0), 6);
    overflow.resolution_scale = f32::MAX;
    assert!(failure_message(renderer, overflow, render).contains("cannot be represented"));

    let excessive = canvas_primitive(canvas_id, 3, Size::new(20_000.0, 1.0), 7);
    assert!(failure_message(renderer, excessive, render).contains("max_texture_dimension_2d"));

    let oversized = canvas_primitive(canvas_id, 3, Size::new(10_000.0, 10_000.0), 8);
    assert!(failure_message(renderer, oversized, render).contains("cache budget"));

    let missing = canvas_primitive(GpuCanvasId::fresh(), 1, Size::new(32.0, 20.0), 9);
    assert!(failure_message(renderer, missing, render).contains("missing registration"));

    let mut constrained = DisplayList::new();
    constrained.push_gpu_canvas(canvas_primitive(canvas_id, 3, Size::new(64.0, 40.0), 10));
    constrained.push_gpu_canvas(canvas_primitive(canvas_id, 3, Size::new(64.0, 40.0), 11));
    let before_constrained = probe.renders.load(Ordering::Relaxed);
    render(renderer, &constrained).unwrap();
    assert_eq!(
        probe.renders.load(Ordering::Relaxed),
        before_constrained + 1
    );
    assert_eq!(renderer.gpu_canvas_stats().failures_this_frame, 1);
    assert!(renderer.gpu_canvas_stats().allocated_bytes <= 16 * 1024);
    assert!(
        renderer.take_gpu_canvas_diagnostics()[0]
            .message
            .contains("visible canvases")
    );

    probe.fail.store(true, Ordering::Relaxed);
    let before_failure = probe.renders.load(Ordering::Relaxed);
    let failing = canvas_list(canvas_id, 3, Size::new(64.0, 40.0), 0, false);
    render(renderer, &failing).unwrap();
    assert_eq!(renderer.gpu_canvas_stats().failures_this_frame, 1);
    let failed = renderer.take_gpu_canvas_diagnostics();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].kind, GpuCanvasDiagnosticKind::Failed);
    assert!(
        failed[0]
            .message
            .contains("intentional integration-test failure")
    );

    render(renderer, &failing).unwrap();
    assert!(renderer.take_gpu_canvas_diagnostics().is_empty());
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_failure + 1);

    probe.fail.store(false, Ordering::Relaxed);
    let recovered = canvas_list(canvas_id, 4, Size::new(64.0, 40.0), 0, false);
    render(renderer, &recovered).unwrap();
    let recovered = renderer.take_gpu_canvas_diagnostics();
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].kind, GpuCanvasDiagnosticKind::Recovered);

    let effected = canvas_list(canvas_id, 5, Size::new(64.0, 40.0), 0, true);
    render(renderer, &effected).unwrap();
    assert!(renderer.last_profile().effects.offscreen_layers >= 1);
    assert!(renderer.last_profile().effects.filter_passes >= 1);

    let mut duplicate = canvas_list(canvas_id, 6, Size::new(64.0, 40.0), 0, false);
    duplicate.push_gpu_canvas(canvas_primitive(canvas_id, 6, Size::new(64.0, 40.0), 0));
    render(renderer, &duplicate).unwrap();
    assert!(
        renderer
            .take_gpu_canvas_diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.message.contains("appears more than once"))
    );

    let resolved = canvas_list(canvas_id, 6, Size::new(64.0, 40.0), 0, false);
    let before_resolved = probe.renders.load(Ordering::Relaxed);
    render(renderer, &resolved).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_resolved);
    assert_eq!(
        renderer.take_gpu_canvas_diagnostics()[0].kind,
        GpuCanvasDiagnosticKind::Recovered
    );

    let retained = canvas_list(canvas_id, 7, Size::new(64.0, 40.0), 0, false);
    render(renderer, &retained).unwrap();
    let before_recreate = probe.renders.load(Ordering::Relaxed);
    renderer.recreate_surface(window.clone()).unwrap();
    render(renderer, &retained).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_recreate);
    assert_eq!(renderer.gpu_canvas_stats().hits_this_frame, 1);

    let creation_failure = canvas_list(retry_id, 1, Size::new(32.0, 20.0), 0, false);
    render(renderer, &creation_failure).unwrap();
    assert_eq!(retry_probe.creates.load(Ordering::Relaxed), 1);
    assert!(
        renderer.take_gpu_canvas_diagnostics()[0]
            .message
            .contains("factory failure")
    );
    render(renderer, &creation_failure).unwrap();
    assert_eq!(retry_probe.creates.load(Ordering::Relaxed), 1);
    assert!(renderer.take_gpu_canvas_diagnostics().is_empty());

    retry_probe.create_fail.store(false, Ordering::Relaxed);
    let creation_recovery = canvas_list(retry_id, 2, Size::new(32.0, 20.0), 0, false);
    render(renderer, &creation_recovery).unwrap();
    assert_eq!(retry_probe.creates.load(Ordering::Relaxed), 2);
    assert_eq!(retry_probe.renders.load(Ordering::Relaxed), 1);
    assert_eq!(
        renderer.take_gpu_canvas_diagnostics()[0].kind,
        GpuCanvasDiagnosticKind::Recovered
    );
}

/// Renders one invalid `primitive` and returns its single diagnostic message.
fn failure_message(
    renderer: &mut SurfaceRenderer,
    primitive: GpuCanvasPrimitive,
    render: &mut impl FnMut(&mut SurfaceRenderer, &DisplayList) -> Result<(), RendererError>,
) -> String {
    let mut list = DisplayList::new();
    list.push_gpu_canvas(primitive);
    render(renderer, &list).unwrap();
    let diagnostics = renderer.take_gpu_canvas_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    diagnostics[0].message.clone()
}
