use std::{sync::mpsc, time::Instant};

use argui_host::Host;
use argui_ui::UiTree;

use argui_gallery_quickjs::{QuickJsGallery, decode_wire_operations};
use argui_runtime::WireOperation;
use serde_json::{Value, json};

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
    let initial = drain_batches(&batches, "Solid mount");
    assert!(
        initial
            .iter()
            .any(|operation| matches!(operation, WireOperation::SetRoot { .. }))
    );
    assert!(contains_value(&initial, "theme-toggle"));
    assert!(
        asset_property_count(&initial) >= 5,
        "retained Media page mounts its five assets at startup"
    );

    gallery
        .deliver(&callback_for(&initial, "theme-toggle").to_string())
        .expect("theme callback should run in QuickJS");
    let changed = drain_batches(&batches, "theme change");
    assert!(
        changed
            .iter()
            .all(|operation| matches!(operation, WireOperation::SetProperty { .. }))
    );

    gallery
        .deliver(&callback_for(&initial, "page-select").to_string())
        .expect("Select navigation should run in QuickJS");
    assert_retained_navigation(&drain_batches(&batches, "Select navigation"));
    gallery
        .deliver(&callback_for(&initial, "topic-select").to_string())
        .expect("Select should open in QuickJS");
    let popup = drain_batches(&batches, "Select popup");
    gallery
        .deliver(&callback_for(&popup, "topic-select-option-1").to_string())
        .expect("option should be selected in QuickJS");
    assert!(contains_value(
        &drain_batches(&batches, "selection"),
        "DirectX 12"
    ));

    gallery
        .deliver(&callback_for(&initial, "page-animation-lab").to_string())
        .expect("Animation Lab navigation should run in QuickJS");
    assert_retained_navigation(&drain_batches(&batches, "Animation Lab navigation"));
    gallery
        .deliver(&callback_for(&initial, "motion-target").to_string())
        .expect("animation retarget should run in QuickJS");
    assert!(
        drain_batches(&batches, "animation retarget")
            .iter()
            .any(|operation| matches!(operation, WireOperation::SetProperty { .. }))
    );

    gallery
        .deliver(&callback_for(&initial, "page-media").to_string())
        .expect("Media navigation should run in QuickJS");
    assert_retained_navigation(&drain_batches(&batches, "Media navigation"));

    gallery.dispose().expect("QuickJS should unmount gallery");
    assert!(
        decode_wire_operations(&batches.recv().expect("disposal batch"))
            .unwrap()
            .iter()
            .any(|operation| matches!(operation, WireOperation::Remove { .. }))
    );
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
            .any(|operation| matches!(operation, WireOperation::SetRoot { .. }))
    );
    assert!(
        asset_property_count(&initial) >= 5,
        "retained React Media page mounts its five assets at startup"
    );
    gallery
        .deliver(&callback_for(&initial, "page-media").to_string())
        .expect("React Media navigation should run in QuickJS");
    let mut media = collect_batches(&batches);
    for step in 0..8 {
        if !media.is_empty() {
            break;
        }
        let now = f64::from(step) * 20.0;
        let delay = gallery.next_wake(now).expect("React timer deadline");
        gallery
            .tick(now + delay.as_secs_f64() * 1000.0 + 1.0)
            .expect("React timer tick");
        media.extend(collect_batches(&batches));
    }
    assert_retained_navigation(&media);
    gallery
        .dispose()
        .expect("QuickJS should unmount React gallery");
    assert!(
        decode_wire_operations(&batches.recv().expect("disposal batch"))
            .unwrap()
            .iter()
            .any(|operation| matches!(operation, WireOperation::Remove { .. }))
    );
}

#[test]
#[ignore = "requires bun run build:gallery"]
fn retained_button_to_animation_lab_profiles_both_adapters() {
    let contract = include_str!("../../../../packages/host/src/contract.generated.json");
    for (adapter, path, entry) in [
        ("solid", "../dist/gallery-core.mjs", "mountGallery"),
        (
            "react",
            "../dist/gallery-react-core.mjs",
            "mountReactGallery",
        ),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        let source = std::fs::read_to_string(path).expect("build the gallery bundle first");
        let (sender, batches) = mpsc::channel();
        let gallery = QuickJsGallery::new(&source, contract, entry, move |batch| {
            sender.send(batch).expect("collector is alive");
            String::new()
        })
        .expect("gallery mount");
        let mut initial = drain_batches(&batches, "profile mount");
        let mut warm_cpu_ms = 0.0;
        let mut warm_operations = 0;
        for step in 0..5 {
            let warm_started = Instant::now();
            gallery
                .tick(f64::from(step * 20))
                .expect("warm adapter tick");
            warm_cpu_ms += warm_started.elapsed().as_secs_f64() * 1000.0;
            let additions = collect_batches(&batches);
            warm_operations += additions.len();
            initial.extend(additions);
        }
        let mut host = Host::with_builtins().expect("schema");
        host.commit(
            &initial
                .iter()
                .cloned()
                .map(WireOperation::into_native)
                .collect::<Result<Vec<_>, _>>()
                .expect("native mount values"),
        )
        .expect("host mount");
        let mut tree = UiTree::new(host.root_element().expect("mounted root"));
        let animation_callback = callback_for(&initial, "page-animation-lab");
        let button_callback = callback_for(&initial, "page-button");
        let mut ready_samples = Vec::new();
        let mut tick_samples = Vec::new();
        for sample in 0..10 {
            let callback = if sample % 2 == 0 {
                &animation_callback
            } else {
                &button_callback
            };
            let started = Instant::now();
            gallery
                .deliver(&callback.to_string())
                .expect("navigation callback");
            let callback_ms = started.elapsed().as_secs_f64() * 1000.0;
            let mut navigation = collect_batches(&batches);
            let mut tick_cpu_ms = 0.0;
            for step in 0..8 {
                if !navigation.is_empty() {
                    break;
                }
                let now = f64::from(sample * 200 + step * 20);
                let delay = gallery.next_wake(now).expect("adapter deadline");
                let tick_started = Instant::now();
                gallery
                    .tick(now + delay.as_secs_f64() * 1000.0 + 1.0)
                    .expect("adapter tick");
                tick_cpu_ms += tick_started.elapsed().as_secs_f64() * 1000.0;
                navigation.extend(collect_batches(&batches));
            }
            let js_ready_ms = started.elapsed().as_secs_f64() * 1000.0;
            assert_retained_navigation(&navigation);
            let creates = navigation
                .iter()
                .filter(|operation| {
                    matches!(
                        operation,
                        WireOperation::Create { .. }
                            | WireOperation::Insert { .. }
                            | WireOperation::Remove { .. }
                    )
                })
                .count();
            assert_eq!(creates, 0);
            let convert_started = Instant::now();
            let native = navigation
                .iter()
                .cloned()
                .map(WireOperation::into_native)
                .collect::<Result<Vec<_>, _>>()
                .expect("native navigation values");
            let decode_ms = convert_started.elapsed().as_secs_f64() * 1000.0;
            let host_started = Instant::now();
            host.commit(&native).expect("host navigation commit");
            let host_ms = host_started.elapsed().as_secs_f64() * 1000.0;
            let tree_started = Instant::now();
            tree.update(host.root_element().expect("navigation root"));
            let tree_ms = tree_started.elapsed().as_secs_f64() * 1000.0;
            ready_samples.push(js_ready_ms);
            tick_samples.push(tick_cpu_ms);
            eprintln!(
                "retained-nav adapter={adapter} sample={sample} warm_operations={warm_operations} warm_cpu_ms={warm_cpu_ms:.3} operations={} creates={creates} callback_ms={callback_ms:.3} native_convert_ms={decode_ms:.3} js_ready_ms={js_ready_ms:.3} tick_cpu_ms={tick_cpu_ms:.3} host_commit_ms={host_ms:.3} ui_tree_ms={tree_ms:.3}",
                navigation.len()
            );
        }
        ready_samples.sort_by(f64::total_cmp);
        tick_samples.sort_by(f64::total_cmp);
        eprintln!(
            "retained-nav adapter={adapter} js_ready_p50_ms={:.3} js_ready_p95_ms={:.3} tick_p50_ms={:.3} tick_p95_ms={:.3}",
            ready_samples[4], ready_samples[9], tick_samples[4], tick_samples[9]
        );
    }
}

/// Counts native media asset bindings already mounted by the retained gallery.
fn asset_property_count(operations: &[WireOperation]) -> usize {
    operations
        .iter()
        .filter(|operation| {
            matches!(operation,
                WireOperation::SetProperty { value: Some(value), .. } if value.value_type == "Asset"
            )
        })
        .count()
}

/// Verifies a tab switch changed only properties on its retained native tree.
fn assert_retained_navigation(operations: &[WireOperation]) {
    assert!(
        !operations.is_empty(),
        "tab switch emitted no visibility update"
    );
    assert!(
        operations
            .iter()
            .any(|operation| matches!(operation, WireOperation::SetProperty { .. })),
        "tab switch emitted no visible property update"
    );
    assert!(
        operations.iter().all(|operation| matches!(
            operation,
            WireOperation::SetProperty { .. } | WireOperation::SetListener { .. }
        )),
        "tab switch rebuilt native nodes"
    );
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

/// Reports whether a transaction sets a string property to `expected`.
fn contains_value(operations: &[WireOperation], expected: &str) -> bool {
    operations.iter().any(|operation| {
        matches!(operation, WireOperation::SetProperty { value: Some(value), .. } if value.value == expected)
    })
}
