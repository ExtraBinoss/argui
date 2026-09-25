use argui_runtime::{CallbackDelivery, CallbackId, HostId, NativeHostDelivery};
use argui_ui::UiEventKind;
use argui_web_host::delivery::event_json;
use serde_json::json;

#[test]
fn delivery_retains_slot_generation_and_callback() {
    let delivery = NativeHostDelivery {
        callback: CallbackDelivery {
            node: HostId::new(3, 7),
            callback: CallbackId(12),
        },
        kind: UiEventKind::Focused,
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
