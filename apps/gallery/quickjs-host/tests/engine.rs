use std::sync::mpsc;

use argui_gallery_quickjs::QuickJsGallery;
use serde_json::{Value, json};

#[test]
fn quickjs_executes_module_bridge_events_microtasks_and_timers() {
    let (sender, batches) = mpsc::channel();
    let source = r#"
        export function mountGallery(bridge, hash) {
            if (bridge.contract().abiHash !== hash) throw Error('schema mismatch')
            bridge.commit([{kind: 'setRoot', id: {slot: 1, generation: 1}}])
            const unsubscribe = bridge.subscribe(event => {
                bridge.commit([{kind: 'event', callback: event.callback}])
            })
            queueMicrotask(() => bridge.commit([{kind: 'microtask'}]))
            const timer = setInterval(() => bridge.commit([{kind: 'timer'}]), 10)
            setTimeout(() => bridge.commit([{kind: 'timeout'}]), 4)
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
    assert!(batches.recv().expect("root batch").contains("setRoot"));
    assert!(batches.recv().expect("job batch").contains("microtask"));
    gallery
        .deliver(r#"{"node":{"slot":1,"generation":1},"callback":7}"#)
        .expect("native callback should be delivered");
    assert!(
        batches
            .recv()
            .expect("event batch")
            .contains("\"callback\":7")
    );
    gallery.tick(0.0).expect("timer should initialize");
    gallery.tick(4.0).expect("one-shot timer should fire");
    assert!(batches.recv().expect("timeout batch").contains("timeout"));
    gallery.tick(10.0).expect("timer should fire");
    assert!(batches.recv().expect("timer batch").contains("timer"));
    gallery.dispose().expect("disposal should succeed");
    assert!(
        batches
            .recv()
            .expect("disposal batch")
            .contains("\"id\":null")
    );
}

#[test]
#[ignore = "requires bun run build:gallery"]
fn neutral_solid_gallery_runs_in_quickjs() {
    let bundle = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../dist/gallery-core.mjs"
    ))
    .expect("build the runtime-neutral gallery first");
    let contract = include_str!("../../../../packages/host/src/contract.generated.json");
    let (sender, batches) = mpsc::channel();
    let gallery = QuickJsGallery::new(&bundle, contract, "mountGallery", move |batch| {
        sender.send(batch).expect("collector is alive");
        String::new()
    })
    .expect("QuickJS should mount Solid gallery");
    let initial = batches.recv().expect("native mount batch");
    assert!(initial.contains("setRoot"));
    assert!(initial.contains("theme-toggle"));

    let initial: Vec<Value> = serde_json::from_str(&initial).expect("initial operations are JSON");
    gallery
        .deliver(&callback_for(&initial, "theme-toggle").to_string())
        .expect("theme callback should run in QuickJS");
    let changed: Vec<Value> = serde_json::from_str(&batches.recv().expect("theme batch"))
        .expect("theme operations are JSON");
    assert!(
        changed
            .iter()
            .all(|operation| operation["kind"] == "setProperty")
    );

    gallery
        .deliver(&callback_for(&initial, "page-select").to_string())
        .expect("Select navigation should run in QuickJS");
    let select: Vec<Value> = serde_json::from_str(&batches.recv().expect("Select page batch"))
        .expect("Select operations are JSON");
    gallery
        .deliver(&callback_for(&select, "topic-select").to_string())
        .expect("Select should open in QuickJS");
    let popup: Vec<Value> = serde_json::from_str(&batches.recv().expect("popup batch"))
        .expect("popup operations are JSON");
    gallery
        .deliver(&callback_for(&popup, "topic-select-option-1").to_string())
        .expect("option should be selected in QuickJS");
    let choice = batches.recv().expect("selection batch");
    assert!(choice.contains("DirectX 12"));

    gallery
        .deliver(&callback_for(&initial, "page-animation-lab").to_string())
        .expect("Animation Lab navigation should run in QuickJS");
    let animation: Vec<Value> =
        serde_json::from_str(&batches.recv().expect("animation page batch"))
            .expect("animation operations are JSON");
    gallery
        .deliver(&callback_for(&animation, "motion-target").to_string())
        .expect("animation retarget should run in QuickJS");
    assert!(
        batches
            .recv()
            .expect("retarget batch")
            .contains("setProperty")
    );

    gallery
        .deliver(&callback_for(&initial, "page-media").to_string())
        .expect("Media navigation should run in QuickJS");
    let media: Vec<Value> = serde_json::from_str(&batches.recv().expect("Media page batch"))
        .expect("Media operations are JSON");
    assert_eq!(
        media
            .iter()
            .filter(|operation| operation["kind"] == "setProperty"
                && operation["value"]["type"] == "Asset")
            .count(),
        5
    );

    gallery.dispose().expect("QuickJS should unmount gallery");
    assert!(batches.recv().expect("disposal batch").contains("remove"));
}

#[test]
#[ignore = "requires bun run build:gallery"]
fn neutral_react_gallery_runs_in_quickjs() {
    let bundle = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../dist/gallery-react-core.mjs"
    ))
    .expect("build the runtime-neutral React gallery first");
    let contract = include_str!("../../../../packages/host/src/contract.generated.json");
    let (sender, batches) = mpsc::channel();
    let gallery = QuickJsGallery::new(&bundle, contract, "mountReactGallery", move |batch| {
        sender.send(batch).expect("collector is alive");
        String::new()
    })
    .expect("QuickJS should mount React gallery");
    let initial = drain_batches(&batches, "mount");
    assert!(
        initial
            .iter()
            .any(|operation| operation["kind"] == "setRoot")
    );
    gallery
        .deliver(&callback_for(&initial, "page-media").to_string())
        .expect("React Media navigation should run in QuickJS");
    let mut media = collect_batches(&batches);
    for step in 0..8 {
        if media
            .iter()
            .any(|operation| operation["value"]["type"] == "Asset")
        {
            break;
        }
        let now = f64::from(step) * 20.0;
        let delay = gallery.next_wake(now).expect("React timer deadline");
        gallery
            .tick(now + delay.as_secs_f64() * 1000.0 + 1.0)
            .expect("React timer tick");
        media.extend(collect_batches(&batches));
    }
    assert_eq!(
        media
            .iter()
            .filter(|operation| operation["kind"] == "setProperty"
                && operation["value"]["type"] == "Asset")
            .count(),
        5
    );
    gallery
        .dispose()
        .expect("QuickJS should unmount React gallery");
    assert!(batches.recv().expect("disposal batch").contains("remove"));
}

/// Collects synchronous native commits emitted during one React render boundary.
fn drain_batches(receiver: &mpsc::Receiver<String>, phase: &str) -> Vec<Value> {
    let operations = collect_batches(receiver);
    assert!(
        !operations.is_empty(),
        "React emitted no native operations during {phase}"
    );
    operations
}

/// Collects native commits already emitted by the JavaScript scheduler.
fn collect_batches(receiver: &mpsc::Receiver<String>) -> Vec<Value> {
    let mut operations = Vec::new();
    while let Ok(batch) = receiver.try_recv() {
        operations.extend(
            serde_json::from_str::<Vec<Value>>(&batch).expect("native operations are JSON"),
        );
    }
    operations
}

/// Finds a click callback for a keyed native control in one commit batch.
fn callback_for(operations: &[Value], key: &str) -> Value {
    let node = &operations
        .iter()
        .find(|operation| operation["kind"] == "setProperty" && operation["value"]["value"] == key)
        .expect("keyed control is present")["id"];
    let listener = operations
        .iter()
        .find(|operation| {
            operation["kind"] == "setListener"
                && operation["id"] == *node
                && operation["event"] == 1
        })
        .expect("control has click listener");
    json!({"node": node, "callback": listener["callback"], "payload": {"kind": "click"}})
}
