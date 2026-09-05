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

#[test]
fn sync_frames_clones_delayed_render_updates_without_reported_growth() {
    let inspector = InspectorHandle::new(3);
    let mut cursor = FrameCursor::default();
    let mut frames = Vec::new();
    inspector.record_ui(FrameRecord {
        passes: 1,
        ..FrameRecord::default()
    });
    assert!(inspector.sync_frames(&mut frames, &mut cursor));
    assert_eq!(frames[0].passes, 1);

    inspector.record_render(FrameRecord {
        render_cpu: std::time::Duration::from_millis(2),
        passes: 9,
        ..FrameRecord::default()
    });
    assert!(inspector.sync_frames(&mut frames, &mut cursor));
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].passes, 9);
    assert_eq!(frames[0].render_cpu, std::time::Duration::from_millis(2));
    assert!(!inspector.sync_frames(&mut frames, &mut cursor));
}

#[test]
fn sync_frames_recovers_when_the_frontend_destination_is_ahead() {
    let inspector = InspectorHandle::new(3);
    let mut cursor = FrameCursor::default();
    let mut frames = vec![FrameRecord {
        passes: 99,
        ..FrameRecord::default()
    }];
    assert!(inspector.sync_frames(&mut frames, &mut cursor));
    assert!(frames.is_empty());

    inspector.record_ui(FrameRecord {
        passes: 4,
        ..FrameRecord::default()
    });
    inspector.record_ui(FrameRecord {
        passes: 5,
        ..FrameRecord::default()
    });
    assert!(inspector.sync_frames(&mut frames, &mut cursor));
    assert_eq!(
        frames.iter().map(|frame| frame.passes).collect::<Vec<_>>(),
        [4, 5]
    );
    inspector.set_recording(false);
    inspector.clear_frames();
    assert!(inspector.sync_frames(&mut frames, &mut cursor));
    assert!(frames.is_empty());
}
