use std::{sync::mpsc, time::Instant};

use argui_host::Host;
use argui_ui::UiTree;

use argui_gallery_quickjs::{QuickJsGallery, decode_wire_operations};
use argui_runtime::WireOperation;
use serde_json::{Value, json};

#[path = "engine/controlled_input.rs"]
mod controlled_input;

/// Covers every compact operation shape and rejects malformed transport fields.
#[test]
fn compact_bridge_batch_validates_operations() {
    let operations = decode_wire_operations(
        r#"[[0,1,2,3],[1,1,2,9,"String","hello"],[2,1,2,1,7],[3,1,2,4,2,null,null],[4,4,2],[5,1,2],[1,1,2,9,null,null],[2,1,2,1,null],[5,null,null]]"#,
    )
    .expect("valid compact transaction");
    assert_eq!(operations.len(), 9);
    assert!(matches!(operations[0], WireOperation::Create { .. }));
    assert!(contains_value(&operations, "hello"));
    assert!(matches!(
        operations[2],
        WireOperation::SetListener {
            callback: Some(7),
            ..
        }
    ));
    assert!(matches!(
        operations[3],
        WireOperation::Insert { before: None, .. }
    ));
    assert!(matches!(operations[4], WireOperation::Remove { .. }));
    assert!(matches!(
        operations[5],
        WireOperation::SetRoot { id: Some(_), .. }
    ));
    assert!(matches!(
        operations[6],
        WireOperation::SetProperty { value: None, .. }
    ));
    assert!(matches!(
        operations[7],
        WireOperation::SetListener { callback: None, .. }
    ));
    assert!(matches!(
        operations[8],
        WireOperation::SetRoot { id: None, .. }
    ));
    for invalid in [
        r#"[[6,1,2]]"#,
        r#"[[0,1,2]]"#,
        r#"[[0,4294967296,2,3]]"#,
        r#"[[1,1,2,9,null,"orphan"]]"#,
        r#"[[3,1,2,4,2,5,null]]"#,
        r#"[[5,null,2]]"#,
        r#"[[0,1,2,3],[5,null,2]]"#,
    ] {
        assert!(decode_wire_operations(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn quickjs_executes_module_bridge_events_microtasks_and_timers() {
    let (sender, batches) = mpsc::channel();
    let source = r#"
        export function mountGallery(bridge, hash) {
            if (bridge.contract().abiHash !== hash) throw Error('schema mismatch')
            bridge.commit([{kind: 'setRoot', id: {slot: 1, generation: 1}}])
            const unsubscribe = bridge.subscribe(event => {
                bridge.commit([{kind: 'setProperty', id: {slot: 1, generation: 1}, property: 9, value: {type: 'String', value: `event-${event.callback}`}}])
            })
            queueMicrotask(() => bridge.commit([{kind: 'setProperty', id: {slot: 1, generation: 1}, property: 9, value: {type: 'String', value: 'microtask'}}]))
            const timer = setInterval(() => bridge.commit([{kind: 'setProperty', id: {slot: 1, generation: 1}, property: 9, value: {type: 'String', value: 'timer'}}]), 10)
            setTimeout(() => bridge.commit([{kind: 'setProperty', id: {slot: 1, generation: 1}, property: 9, value: {type: 'String', value: 'timeout'}}]), 4)
            return () => {
                clearInterval(timer)
                unsubscribe()
                bridge.commit([{kind: 'setRoot', id: null}])
            }
        }
    "#;
    let gallery = QuickJsGallery::new(
        source,
        r#"{"abiHash":"test","natives":[]}"#,
        "mountGallery",
        move |batch| {
            sender.send(batch).expect("collector is alive");
            String::new()
        },
    )
    .expect("QuickJS should mount an ES module");
    assert!(matches!(
        decode_wire_operations(&batches.recv().expect("root batch")).unwrap()[0],
        WireOperation::SetRoot { .. }
    ));
    assert!(contains_value(
        &decode_wire_operations(&batches.recv().expect("job batch")).unwrap(),
        "microtask"
    ));
    gallery
        .deliver(r#"{"node":{"slot":1,"generation":1},"callback":7}"#)
        .expect("native callback should be delivered");
    assert!(contains_value(
        &decode_wire_operations(&batches.recv().expect("event batch")).unwrap(),
        "event-7"
    ));
    gallery.tick(0.0).expect("timer should initialize");
    gallery.tick(4.0).expect("one-shot timer should fire");
    assert!(contains_value(
        &decode_wire_operations(&batches.recv().expect("timeout batch")).unwrap(),
        "timeout"
    ));
    gallery.tick(10.0).expect("timer should fire");
    assert!(contains_value(
        &decode_wire_operations(&batches.recv().expect("timer batch")).unwrap(),
        "timer"
    ));
    gallery.dispose().expect("disposal should succeed");
    assert!(matches!(
        decode_wire_operations(&batches.recv().expect("disposal batch")).unwrap()[0],
        WireOperation::SetRoot { id: None }
    ));
}

#[test]
fn quickjs_reports_callback_message_and_stack() {
    let source = r#"
        export function mountGallery(bridge) {
            bridge.subscribe(() => { throw new Error('breadcrumb callback failed') })
            return () => {}
        }
    "#;
    let gallery = QuickJsGallery::new(
        source,
        r#"{"abiHash":"test","natives":[]}"#,
        "mountGallery",
        |_| String::new(),
    )
    .expect("QuickJS should mount the callback");
    let error = gallery.deliver("{}").expect_err("callback must fail");
    assert!(error.contains("breadcrumb callback failed"), "{error}");
    assert!(error.contains("gallery-core.mjs"), "{error}");
}

#[test]
fn quickjs_reports_timer_message_and_stack() {
    let source = r#"
        export function mountGallery() {
            setTimeout(() => { throw new Error('breadcrumb timer failed') }, 1)
            return () => {}
        }
    "#;
    let gallery = QuickJsGallery::new(
        source,
        r#"{"abiHash":"test","natives":[]}"#,
        "mountGallery",
        |_| String::new(),
    )
    .expect("timer should mount");
    gallery.tick(0.0).expect("timer should initialize");
    let error = gallery.tick(1.0).expect_err("timer must fail");
    assert!(error.contains("breadcrumb timer failed"), "{error}");
    assert!(error.contains("gallery-core.mjs"), "{error}");
}

#[test]
fn quickjs_reports_microtask_message_and_stack() {
    let source = r#"
        export function mountGallery() {
            queueMicrotask(() => { throw new Error('breadcrumb effect failed') })
            return () => {}
        }
    "#;
    let error = QuickJsGallery::new(
        source,
        r#"{"abiHash":"test","natives":[]}"#,
        "mountGallery",
        |_| String::new(),
    )
    .err()
    .expect("microtask must fail");
    assert!(error.contains("breadcrumb effect failed"), "{error}");
    assert!(error.contains("gallery-core.mjs"), "{error}");
}

#[test]
#[ignore = "requires bun run build:gallery"]
fn neutral_solid_gallery_runs_in_quickjs() {
    run_gallery_interactions("solid", "../dist/gallery-core.mjs", "mountGallery");
}

#[test]
#[ignore = "requires bun run build:gallery"]
fn neutral_react_gallery_runs_in_quickjs() {
    run_gallery_interactions(
        "react",
        "../dist/gallery-react-core.mjs",
        "mountReactGallery",
    );
}

/// Mounts one adapter and exercises real button, theme, and controlled input callbacks.
fn run_gallery_interactions(adapter: &str, path: &str, entry: &str) {
    let contract = include_str!("../../../../packages/host/src/contract.generated.json");
    let source =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
            .expect("build the gallery bundle first");
    let (sender, batches) = mpsc::channel();
    let gallery = QuickJsGallery::new(&source, contract, entry, move |batch| {
        sender.send(batch).expect("collector is alive");
        String::new()
    })
    .expect("gallery mount");
    let initial = drain_batches(&batches, "gallery mount");
    gallery
        .deliver(&callback_for(&initial, "button-primary").to_string())
        .expect("button callback should run in QuickJS");
    let clicked = collect_after_ticks(&gallery, &batches, 10.0);
    assert!(
        contains_value(&clicked, "Clicked 1 times"),
        "{adapter} button state updates"
    );

    gallery
        .deliver(&callback_for(&initial, "gallery-settings").to_string())
        .expect("settings popover should open in QuickJS");
    let settings = collect_after_ticks(&gallery, &batches, 80.0);
    gallery
        .deliver(&callback_for(&settings, "theme-dark").to_string())
        .expect("theme callback should run in QuickJS");
    let themed = collect_after_ticks(&gallery, &batches, 100.0);
    assert!(
        !themed.is_empty(),
        "{adapter} theme change emits native properties"
    );
    assert!(
        themed
            .iter()
            .all(|operation| matches!(operation, WireOperation::SetProperty { .. }))
    );

    gallery
        .deliver(&callback_for(&initial, "page-example").to_string())
        .expect("layout Example page should open");
    let example = collect_after_ticks(&gallery, &batches, 160.0);
    assert!(contains_value(&example, "layout-fixed-grow-320"));
    assert!(contains_value(&example, "layout-bounded-scroll"));
    assert!(contains_value(&example, "layout-rtl-marker"));

    gallery
        .deliver(&callback_for(&initial, "page-input-field").to_string())
        .expect("InputField page should open");
    let input_page = collect_after_ticks(&gallery, &batches, 200.0);
    assert!(contains_value(&input_page, "input-name"));
    let (input_id, callback) = callback_for_event(&input_page, "input-name", 23);
    gallery
        .deliver(
            &json!({
                "node": {"slot": input_id.slot, "generation": input_id.generation},
                "callback": callback,
                "payload": {"kind": "edit", "start": 0, "end": 3, "text": "Oct"},
            })
            .to_string(),
        )
        .expect("native edit callback should run");
    let edited = collect_after_ticks(&gallery, &batches, 300.0);
    assert!(
        contains_value(&edited, "Current value: Oct Lovelace"),
        "{adapter} controlled edit updates"
    );

    gallery.dispose().expect("gallery should unmount");
    assert!(
        decode_wire_operations(&batches.recv().expect("disposal batch"))
            .unwrap()
            .iter()
            .any(|operation| matches!(operation, WireOperation::Remove { .. }))
    );
}

/// Collects native operations immediately or after a bounded set of adapter ticks.
fn collect_after_ticks(
    gallery: &QuickJsGallery,
    receiver: &mpsc::Receiver<String>,
    start_ms: f64,
) -> Vec<WireOperation> {
    let mut operations = collect_batches(receiver);
    for step in 0..8 {
        if !operations.is_empty() {
            break;
        }
        gallery
            .tick(start_ms + f64::from(step) * 20.0)
            .expect("adapter scheduler tick");
        operations.extend(collect_batches(receiver));
    }
    operations
}

/// Collects synchronous native commits emitted during one React render boundary.
fn drain_batches(receiver: &mpsc::Receiver<String>, phase: &str) -> Vec<WireOperation> {
    let operations = collect_batches(receiver);
    assert!(
        !operations.is_empty(),
        "React emitted no native operations during {phase}"
    );
    operations
}

/// Collects native commits already emitted by the JavaScript scheduler.
fn collect_batches(receiver: &mpsc::Receiver<String>) -> Vec<WireOperation> {
    let mut operations = Vec::new();
    while let Ok(batch) = receiver.try_recv() {
        operations.extend(decode_wire_operations(&batch).expect("native operations are JSON"));
    }
    operations
}

/// Finds a click callback for a keyed native control in one commit batch.
fn callback_for(operations: &[WireOperation], key: &str) -> Value {
    let node = operations
        .iter()
        .find_map(|operation| {
            if let WireOperation::SetProperty {
                id,
                value: Some(value),
                ..
            } = operation
                && value.value == key
            {
                Some(*id)
            } else {
                None
            }
        })
        .expect("keyed control is present");
    let callback = operations
        .iter()
        .find_map(|operation| {
            if let WireOperation::SetListener {
                id,
                event: 1,
                callback,
            } = operation
                && *id == node
            {
                *callback
            } else {
                None
            }
        })
        .expect("control has click listener");
    json!({"node": {"slot": node.slot, "generation": node.generation}, "callback": callback, "payload": {"kind": "click"}})
}

/// Finds a native callback for a keyed control and a specific event type.
fn callback_for_event(
    operations: &[WireOperation],
    key: &str,
    event_type: u16,
) -> (argui_runtime::WireHostId, u32) {
    let node = operations
        .iter()
        .find_map(|operation| match operation {
            WireOperation::SetProperty {
                id,
                value: Some(value),
                ..
            } if value.value == key => Some(*id),
            _ => None,
        })
        .expect("keyed control is present");
    let callback = operations
        .iter()
        .find_map(|operation| match operation {
            WireOperation::SetListener {
                id,
                event,
                callback,
            } if *id == node && *event == event_type => *callback,
            _ => None,
        })
        .expect("control has the requested event listener");
    (node, callback)
}

/// Reports whether a transaction sets a string property to `expected`.
fn contains_value(operations: &[WireOperation], expected: &str) -> bool {
    operations.iter().any(|operation| {
        matches!(operation, WireOperation::SetProperty { value: Some(value), .. } if value.value == expected)
    })
}
