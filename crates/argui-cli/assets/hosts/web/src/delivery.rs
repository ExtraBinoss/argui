//! Converts retained UI deliveries into browser callback payloads.

use argui_core::{Key, KeyState};
use argui_runtime::NativeHostDelivery;
use argui_ui::{SemanticAction, SemanticValue, UiEventKind};
use serde_json::{Value, json};

/// Encodes one retained callback and its UI event for the JavaScript subscriber.
///
/// `delivery` contains the native node, callback, and event. Returns the
/// JSON object accepted by the gallery bridge.
pub fn event_json(delivery: &NativeHostDelivery) -> Value {
    let mut payload = ui_event_payload(&delivery.kind);
    if let Some(pointer) = delivery.pointer
        && let Value::Object(fields) = &mut payload
    {
        fields.insert("x".into(), json!(pointer.x));
        fields.insert("y".into(), json!(pointer.y));
        fields.insert("localX".into(), json!(pointer.local_x));
        fields.insert("localY".into(), json!(pointer.local_y));
        fields.insert("width".into(), json!(pointer.width));
        fields.insert("height".into(), json!(pointer.height));
    }
    json!({
        "node": {"slot": delivery.callback.node.slot(), "generation": delivery.callback.node.generation()},
        "callback": delivery.callback.callback.0,
        "payload": payload,
    })
}

/// Encodes the public fields of `kind` for a JavaScript event handler.
///
/// Returns a JSON payload with a stable `kind` string; scroll events include
/// the native absolute offset in logical pixels.
pub fn ui_event_payload(kind: &UiEventKind) -> Value {
    match kind {
        UiEventKind::KeyInput(input) => json!({
            "kind": "key", "key": key_name(&input.key),
            "state": match input.state { KeyState::Pressed => "pressed", KeyState::Released => "released" },
            "text": input.text,
            "shift": input.modifiers.shift, "control": input.modifiers.control,
            "alt": input.modifiers.alt, "super": input.modifiers.super_key,
            "repeat": input.repeat,
        }),
        UiEventKind::TextChanged(text) => json!({"kind": "input", "text": text}),
        UiEventKind::Submitted(text) => json!({"kind": "submit", "text": text}),
        UiEventKind::Focused => json!({"kind": "focus"}),
        UiEventKind::Blurred => json!({"kind": "blur"}),
        UiEventKind::Click(_) => json!({"kind": "click"}),
        UiEventKind::Scrolled { offset, .. } => {
            json!({"kind": "scroll", "offsetX": offset.x, "offsetY": offset.y})
        }
        UiEventKind::VirtualMeasured {
            items,
            corrected_offset,
            viewport_extent,
        } => json!({
            "kind": "measure",
            "items": items.iter().map(|item| json!({
                "index": item.index,
                "extent": item.extent,
            })).collect::<Vec<_>>(),
            "correctedOffset": corrected_offset,
            "viewportExtent": viewport_extent,
        }),
        UiEventKind::VirtualWindowChanged {
            start,
            end,
            offset,
            viewport_extent,
        } => json!({
            "kind": "window",
            "start": start,
            "end": end,
            "offset": offset,
            "viewportExtent": viewport_extent,
        }),
        UiEventKind::SemanticAction { action, value } => json!({
            "kind": "semantic_action",
            "action": match action {
                SemanticAction::Click => "click",
                SemanticAction::Focus => "focus",
                SemanticAction::Blur => "blur",
                SemanticAction::Increment => "increment",
                SemanticAction::Decrement => "decrement",
                SemanticAction::Expand => "expand",
                SemanticAction::Collapse => "collapse",
                SemanticAction::SetValue => "set_value",
                SemanticAction::ScrollIntoView => "scroll_into_view",
            },
            "value": match value {
                Some(SemanticValue::Text(text)) => json!(text),
                Some(SemanticValue::Number { value, .. }) => json!(value),
                None => Value::Null,
            },
        }),
        other => json!({"kind": format!("{:?}", other.event_type())}),
    }
}

/// Gives a native key a stable spelling for JavaScript callbacks.
///
/// `key` is the platform-independent key. Returns its character value or
/// the Rust key variant name.
fn key_name(key: &Key) -> String {
    match key {
        Key::Character(value) => value.clone(),
        other => format!("{other:?}"),
    }
}
