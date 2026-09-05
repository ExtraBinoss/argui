use argui_inspect::{FrameCursor, FrameRecord, InspectorHandle};

#[test]
fn renderer_profiling_is_independent_from_tree_and_cpu_recording() {
    let inspector = InspectorHandle::default();
    assert!(inspector.gpu_profiling());
    inspector.set_gpu_profiling(false);
    assert!(!inspector.gpu_profiling());
    assert!(inspector.enabled());
    inspector.record_ui(FrameRecord::default());
    assert_eq!(inspector.frames().len(), 1);
    inspector.set_gpu_profiling(true);
    inspector.set_paused(true);
    assert!(!inspector.gpu_profiling());
    assert!(inspector.enabled());
    inspector.set_paused(false);
    assert!(inspector.gpu_profiling());
    inspector.set_recording(false);
    assert!(!inspector.gpu_profiling());
    assert!(!inspector.enabled());
}

#[test]
fn synchronized_history_handles_eviction_pause_and_clear() {
    let inspector = InspectorHandle::new(2);
    let mut cursor = FrameCursor::default();
    let mut frames = vec![];
    assert!(!inspector.sync_frames(&mut frames, &mut cursor));
    for index in 1..=4 {
        inspector.record_ui(FrameRecord {
            passes: index,
            ..FrameRecord::default()
        });
        assert!(inspector.sync_frames(&mut frames, &mut cursor));
        assert_eq!(frames, inspector.frames());
        assert!(!inspector.sync_frames(&mut frames, &mut cursor));
    }
    inspector.set_paused(true);
    inspector.record_ui(FrameRecord::default());
    assert!(!inspector.sync_frames(&mut frames, &mut cursor));
    inspector.clear_frames();
    assert!(inspector.sync_frames(&mut frames, &mut cursor));
    assert!(frames.is_empty());
}
