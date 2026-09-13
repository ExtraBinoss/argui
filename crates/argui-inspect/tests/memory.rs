use argui_inspect::{InspectorHandle, MemorySnapshot};

#[test]
fn memory_requests_coalesce_are_gated_and_do_not_expand_frame_history() {
    let inspector = InspectorHandle::default();
    inspector.set_recording(false);
    inspector.request_memory_sample();
    assert!(!inspector.take_memory_request());
    assert_eq!(inspector.memory(), None);
    inspector.set_recording(true);
    inspector.request_memory_sample();
    inspector.request_memory_sample();
    assert!(inspector.take_memory_request());
    assert!(!inspector.take_memory_request());
    let snapshot = MemorySnapshot {
        ui_nodes: 100,
        ui_index_bytes: 4096,
        ..Default::default()
    };
    inspector.publish_memory(snapshot);
    assert_eq!(inspector.memory(), Some(snapshot));
    inspector.request_memory_sample();
    inspector.set_paused(true);
    assert!(!inspector.take_memory_request());
    inspector.request_memory_sample();
    assert!(!inspector.take_memory_request());
    assert!(inspector.frames().is_empty());
}
