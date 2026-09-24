use argui_core::Point;
use argui_gallery_quickjs::{coalesce_virtual_windows, ui_event_payload};
use argui_runtime::{CallbackDelivery, CallbackId, HostId, NativeHostDelivery};
use argui_ui::{UiEventKind, VirtualMeasurement};
use serde_json::json;

#[test]
fn scroll_delivery_keeps_native_absolute_offsets() {
    let payload = ui_event_payload(&UiEventKind::Scrolled {
        delta: Point::new(-12.5, 44.0),
        offset: Point::new(3.25, 812.75),
    });
    assert_eq!(
        payload,
        json!({"kind": "scroll", "offsetX": 3.25, "offsetY": 812.75})
    );
}

#[test]
fn virtual_measurement_delivery_preserves_extents_and_correction() {
    let payload = ui_event_payload(&UiEventKind::VirtualMeasured {
        items: vec![
            VirtualMeasurement {
                index: 7,
                extent: 31.5,
            },
            VirtualMeasurement {
                index: 8,
                extent: 52.25,
            },
        ],
        corrected_offset: 221.0,
        viewport_extent: 360.0,
    });
    assert_eq!(
        payload,
        json!({
            "kind": "measure",
            "items": [{"index": 7, "extent": 31.5}, {"index": 8, "extent": 52.25}],
            "correctedOffset": 221.0,
            "viewportExtent": 360.0,
        })
    );
}

#[test]
fn virtual_window_delivery_reports_chunk_boundary() {
    let payload = ui_event_payload(&UiEventKind::VirtualWindowChanged {
        start: 18,
        end: 26,
        offset: 810.5,
        viewport_extent: 360.0,
    });
    assert_eq!(
        payload,
        json!({
            "kind": "window",
            "start": 18,
            "end": 26,
            "offset": 810.5,
            "viewportExtent": 360.0,
        })
    );
}

#[test]
fn obsolete_window_ranges_coalesce_without_dropping_other_events() {
    let callback = CallbackDelivery {
        node: HostId::new(4, 1),
        callback: CallbackId(9),
    };
    let window = |start| NativeHostDelivery {
        callback,
        kind: UiEventKind::VirtualWindowChanged {
            start,
            end: start + 8,
            offset: start as f32 * 40.0,
            viewport_extent: 320.0,
        },
    };
    let burst = coalesce_virtual_windows(vec![
        window(0),
        window(4),
        NativeHostDelivery {
            callback,
            kind: UiEventKind::Focused,
        },
        window(8),
        window(12),
    ]);
    assert_eq!(burst.len(), 3);
    assert!(matches!(
        burst[0].kind,
        UiEventKind::VirtualWindowChanged { start: 4, .. }
    ));
    assert!(matches!(burst[1].kind, UiEventKind::Focused));
    assert!(matches!(
        burst[2].kind,
        UiEventKind::VirtualWindowChanged { start: 12, .. }
    ));
}
