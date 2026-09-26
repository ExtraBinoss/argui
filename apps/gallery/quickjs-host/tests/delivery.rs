use argui_core::Point;
use argui_gallery_quickjs::{coalesce_virtual_windows, event_json, ui_event_payload};
use argui_runtime::{CallbackDelivery, CallbackId, HostId, NativeHostDelivery, NativePointerPosition};
use argui_ui::{SemanticAction, SemanticValue};
use argui_ui::{UiEventKind, VirtualMeasurement};
use serde_json::json;

#[test]
fn pointer_delivery_exposes_window_and_local_coordinates() {
    let delivery = NativeHostDelivery {
        callback: CallbackDelivery { node: HostId::new(4, 1), callback: CallbackId(9) },
        kind: UiEventKind::Focused,
        pointer: Some(NativePointerPosition {
            x: 90.0, y: 42.0, local_x: 45.0, local_y: 12.0, width: 180.0, height: 24.0,
        }),
    };
    let payload = event_json(&delivery)["payload"].clone();
    assert_eq!(payload["localX"], 45.0);
    assert_eq!(payload["width"], 180.0);
    assert_eq!(payload["x"], 90.0);
}

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
        pointer: None,
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
            pointer: None,
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
#[test]
fn semantic_action_delivery_keeps_action_and_requested_value() {
    let payload = ui_event_payload(&UiEventKind::SemanticAction {
        action: SemanticAction::SetValue,
        value: Some(SemanticValue::Number {
            value: 55.0,
            minimum: None,
            maximum: None,
            step: None,
        }),
    });
    assert_eq!(payload["kind"], "semanticAction");
    assert_eq!(payload["action"], "setValue");
    assert_eq!(payload["value"], 55.0);
}

#[test]
fn semantic_scroll_action_uses_camel_case_wire_name() {
    let payload = ui_event_payload(&UiEventKind::SemanticAction {
        action: SemanticAction::ScrollIntoView,
        value: None,
    });
    assert_eq!(payload["kind"], "semanticAction");
    assert_eq!(payload["action"], "scrollIntoView");
}

#[test]
fn fallback_event_kind_uses_stable_lower_camel_wire_name() {
    let payload = ui_event_payload(&UiEventKind::DocumentSelectionChanged {
        text: None,
        bounds: None,
        touch: false,
        dragging: false,
    });
    assert_eq!(payload["kind"], "selectionChange");
}
