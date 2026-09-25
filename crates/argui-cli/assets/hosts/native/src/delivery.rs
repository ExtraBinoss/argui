//! Converts native UI deliveries into the JavaScript bridge event payload.

use std::collections::HashMap;

use argui_core::{Key, KeyState};
use argui_runtime::{HostId, NativeHostDelivery};
use argui_ui::{GestureKind, GesturePhase, SemanticAction, SemanticValue, UiEventKind};
use serde_json::{Value, json};

/// Keeps the newest virtual-window callback for each native node in a burst.
///
/// `deliveries` is the native FIFO burst. Returns deliveries in order, with
/// obsolete window ranges removed until a different event forms an ordering
/// boundary. Clicks and measurements are never discarded.
pub fn coalesce_virtual_windows(deliveries: Vec<NativeHostDelivery>) -> Vec<NativeHostDelivery> {
    let mut result = Vec::with_capacity(deliveries.len());
    let mut pending: HashMap<(HostId, u32), usize> = HashMap::new();
    for delivery in deliveries {
        if matches!(delivery.kind, UiEventKind::VirtualWindowChanged { .. }) {
            let key = (delivery.callback.node, delivery.callback.callback.0);
            if let Some(index) = pending.get(&key) {
                result[*index] = delivery;
            } else {
                pending.insert(key, result.len());
                result.push(delivery);
            }
        } else {
            pending.clear();
            result.push(delivery);
        }
    }
    result
}

/// Encodes one native callback and its UI event for the JavaScript subscriber.
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
        "node": {"slot": delivery.callback.node.slot(), "generation": 1},
        "callback": delivery.callback.callback.0,
        "payload": payload,
    })
}

/// Encodes the public fields of `kind` for a JavaScript event handler.
///
/// Returns a JSON payload with a stable `kind` string. Text edits carry UTF-8
/// byte range endpoints and replacement text. Scroll and pan distances use
/// logical pixels, while pan velocity uses logical pixels per second.
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
        UiEventKind::TextEdited(edit) => json!({
            "kind": "edit", "start": edit.range.start, "end": edit.range.end,
            "text": edit.replacement,
        }),
        UiEventKind::Submitted(text) => json!({"kind": "submit", "text": text}),
        UiEventKind::Focused => json!({"kind": "focus"}),
        UiEventKind::Blurred => json!({"kind": "blur"}),
        UiEventKind::Click(_) => json!({"kind": "click"}),
        UiEventKind::Scrolled { offset, .. } => {
            json!({"kind": "scroll", "offsetX": offset.x, "offsetY": offset.y})
        }
        UiEventKind::Gesture(gesture) => match gesture.kind {
            GestureKind::Pan { delta, total, velocity, .. } => json!({
                "kind": "pan", "deltaX": delta.x, "deltaY": delta.y,
                "totalX": total.x, "totalY": total.y,
                "velocityX": velocity.x, "velocityY": velocity.y,
                "phase": match gesture.phase {
                    GesturePhase::Started => "started",
                    GesturePhase::Changed => "changed",
                    GesturePhase::Ended => "ended",
                    GesturePhase::Cancelled => "cancelled",
                },
            }),
            _ => json!({"kind": "gesture"}),
        },
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
