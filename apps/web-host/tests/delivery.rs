use argui_runtime::{CallbackDelivery, CallbackId, HostId, NativeHostDelivery, NativePointerPosition};
use argui_ui::UiEventKind;
use argui_web_host::delivery::event_json;
use serde_json::json;

#[test]
fn pointer_delivery_exposes_measured_relative_coordinates() {
    let delivery = NativeHostDelivery {
        callback: CallbackDelivery { node: HostId::new(3, 7), callback: CallbackId(12) },
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
fn delivery_retains_slot_generation_and_callback() {
    let delivery = NativeHostDelivery {
        callback: CallbackDelivery {
            node: HostId::new(3, 7),
            callback: CallbackId(12),
        },
        kind: UiEventKind::Focused,
        pointer: None,
    };
    assert_eq!(
        event_json(&delivery),
        json!({
            "node": {"slot": 3, "generation": 7},
            "callback": 12,
            "payload": {"kind": "focus"},
        })
    );
}
