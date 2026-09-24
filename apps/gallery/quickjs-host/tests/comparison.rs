use std::{sync::mpsc, time::Instant};

use argui_gallery_quickjs::{AnimationSnapshot, QuickJsGallery, decode_wire_operations};
use argui_host::Host;
use argui_platform::WindowKey;
use argui_runtime::{AppModel, WindowEnvironment, WireOperation};
use argui_ui::UiTree;
use serde_json::json;

#[test]
#[ignore = "requires bun run build:gallery"]
fn rust_snapshot_matches_live_animation_lab_and_loops_emit_no_js_commits() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../dist/gallery-core.mjs"
    ))
    .expect("build gallery bundle");
    let contract = include_str!("../../../../packages/host/src/contract.generated.json");
    let snapshot = AnimationSnapshot::build(
        &source,
        contract,
        argui_gallery_assets::load().expect("gallery media"),
    )
    .expect("Animation Lab snapshot");
    let (sender, batches) = mpsc::channel();
    let gallery = QuickJsGallery::new(&source, contract, "mountGallery", move |batch| {
        sender.send(batch).expect("collector remains live");
        String::new()
    })
    .expect("live gallery");
    let initial = batches.recv().expect("mount transaction");
    let initial_operations = decode_wire_operations(&initial).expect("mount JSON");
    let node = initial_operations
        .iter()
        .find_map(|operation| {
            if let WireOperation::SetProperty {
                id,
                value: Some(value),
                ..
            } = operation
                && value.value == "page-animation-lab"
            {
                Some(*id)
            } else {
                None
            }
        })
        .expect("Animation Lab control");
    let callback = initial_operations
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
        .expect("Animation Lab click listener");
    let navigation_started = Instant::now();
    gallery
        .deliver(
            &json!({
                "node": {"slot": node.slot, "generation": node.generation}, "callback": callback, "payload": {"kind": "click"}
            })
            .to_string(),
        )
        .expect("navigation");
    eprintln!(
        "scene-quickjs-navigation-ms={:.3}",
        navigation_started.elapsed().as_secs_f64() * 1000.0
    );
    let navigation = batches.recv().expect("navigation transaction");
    let mut navigation_batches = vec![navigation];
    while let Ok(batch) = batches.try_recv() {
        navigation_batches.push(batch);
    }
    let mut host = Host::with_builtins().expect("schema");
    let live_wire = std::iter::once(&initial)
        .chain(navigation_batches.iter())
        .map(|batch| decode_wire_operations(batch).expect("wire batch"))
        .collect::<Vec<_>>();
    assert_eq!(snapshot.wire_batches(), live_wire);
    let mut tree: Option<UiTree> = None;
    for batch in live_wire {
        let operation_count = batch.len();
        let operations = batch
            .into_iter()
            .map(WireOperation::into_native)
            .collect::<Result<Vec<_>, _>>()
            .expect("typed batch");
        let started = Instant::now();
        host.commit(&operations).expect("host commit");
        let host_time = started.elapsed();
        let started = Instant::now();
        let root = host.root_element().expect("visible root");
        if let Some(tree) = tree.as_mut() {
            tree.update(root);
        } else {
            tree = Some(UiTree::new(root));
        }
        eprintln!(
            "scene-batch operations={operation_count} host_ms={:.3} tree_ms={:.3}",
            host_time.as_secs_f64() * 1000.0,
            started.elapsed().as_secs_f64() * 1000.0
        );
    }
    assert_eq!(
        snapshot.root().children.len(),
        host.root_element().expect("visible root").children.len()
    );
    assert!(AppModel::view(&snapshot, &WindowKey::main(), WindowEnvironment::default()).is_some());
    gallery.tick(0.0).expect("start timer clock");
    gallery.tick(1000.0).expect("advance native loops");
    assert!(
        batches.try_recv().is_err(),
        "native loops must not post JS transactions"
    );
}
