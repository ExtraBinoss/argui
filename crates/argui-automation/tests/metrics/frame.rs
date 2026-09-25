use std::time::Duration;

use argui_automation::{ActionWindow, StepFrame, frame_diagnostics};
use argui_inspect::{FrameRecord, GpuFrameRecord, GpuPassRecord};

#[test]
fn frame_diagnostic_correlates_distinct_update_and_render_actions() {
    let frames = [StepFrame {
        timestamp_ms: 12.0,
        interval_ms: None,
        cpu_ms: 8.0,
        layout_ms: 4.0,
        paint_ms: 2.0,
        layout_passes: 2,
        render_cpu_ms: Some(3.0),
        rendered_at_ms: Some(42.0),
    }];
    let records = [FrameRecord {
        model: Duration::from_millis(1),
        surface: Duration::from_millis(2),
        resize_events: 3,
        passes: 3,
        damaged_pixels: 123,
        gpu: Some(GpuFrameRecord {
            sequence: 7,
            total: Duration::from_millis(5),
            passes: vec![GpuPassRecord {
                label: "surface.main".into(),
                duration: Duration::from_millis(4),
                ..GpuPassRecord::default()
            }],
        }),
        ..FrameRecord::default()
    }];
    let actions = [
        ActionWindow {
            started_ms: 10.0,
            duration_ms: 10.0,
        },
        ActionWindow {
            started_ms: 40.0,
            duration_ms: 10.0,
        },
    ];
    let frame = frame_diagnostics(&frames, &records, &actions).remove(0);
    assert_eq!(frame.update_action_index, Some(0));
    assert_eq!(frame.render_action_index, Some(1));
    assert_eq!(frame.total_cpu_ms, 13.0);
    assert_eq!(frame.surface_cpu_ms, 2.0);
    assert_eq!(frame.resize_events, 3);
    assert_eq!(frame.dominant_cpu_phase, "layout");
    assert_eq!(frame.gpu_ms, Some(5.0));
    assert_eq!(frame.gpu_passes[0].name, "surface.main");
    assert_eq!(frame.render_workload.unwrap().draw_batches, 3);
}

#[test]
fn unavailable_gpu_and_renderer_data_stay_missing() {
    let frames = [StepFrame {
        timestamp_ms: 1.0,
        interval_ms: None,
        cpu_ms: 1.0,
        layout_ms: 0.4,
        paint_ms: 0.2,
        layout_passes: 1,
        render_cpu_ms: None,
        rendered_at_ms: None,
    }];
    let frame = frame_diagnostics(&frames, &[FrameRecord::default()], &[]).remove(0);
    assert_eq!(frame.gpu_ms, None);
    assert!(frame.render_workload.is_none());
    assert_eq!(frame.render_action_index, None);
}
