use argui_inspect::{FrameRecord, InspectorHandle};

/// Glyph counters survive renderer/UI merging and strict trace serialization.
#[test]
fn glyph_cache_metrics_survive_recording_and_trace_round_trip() {
    let inspector = InspectorHandle::default();
    inspector.record_ui(FrameRecord::default());
    inspector.record_render(FrameRecord {
        text_atlas_bytes: 12 * 1024 * 1024,
        text_atlas_entries: 250,
        text_atlas_hits: 1200,
        text_raster_requests: 8,
        text_upload_bytes: 3072,
        text_page_evictions: 1,
        ..FrameRecord::default()
    });
    let imported = InspectorHandle::default();
    imported
        .import_trace_json(&inspector.trace_json().unwrap())
        .unwrap();
    let frame = imported.frames().pop().unwrap();
    assert_eq!(frame.text_atlas_bytes, 12 * 1024 * 1024);
    assert_eq!(frame.text_atlas_entries, 250);
    assert_eq!(frame.text_atlas_hits, 1200);
    assert_eq!(frame.text_raster_requests, 8);
    assert_eq!(frame.text_upload_bytes, 3072);
    assert_eq!(frame.text_page_evictions, 1);
    assert_eq!(inspector.frames(), imported.frames());
}
