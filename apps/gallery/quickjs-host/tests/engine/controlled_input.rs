use super::*;
use argui_gallery_quickjs::ui_event_payload;
use argui_ui::{FocusTarget, UiEventKind};

/// Profiles a controlled million-character edit through the actual Solid and React gallery adapters.
#[test]
#[ignore = "requires bun run build:gallery"]
fn controlled_million_character_input_profiles_both_adapters() {
    let contract = include_str!("../../../../../packages/host/src/contract.generated.json");
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
        let mut initial = drain_batches(&batches, "gallery mount");
        for step in 0..5 {
            gallery
                .tick(f64::from(step * 20))
                .expect("warm adapter tick");
            initial.extend(collect_batches(&batches));
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
        let input_page = callback_for(&initial, "page-input");
        gallery
            .deliver(&input_page.to_string())
            .expect("open Input page");
        let mut navigation = collect_batches(&batches);
        for step in 0..8 {
            if !navigation.is_empty() {
                break;
            }
            gallery
                .tick(f64::from(100 + step * 20))
                .expect("React navigation tick");
            navigation.extend(collect_batches(&batches));
        }
        assert!(
            !navigation.is_empty(),
            "Input navigation emitted no native operations"
        );
        host.commit(
            &navigation
                .iter()
                .cloned()
                .map(WireOperation::into_native)
                .collect::<Result<Vec<_>, _>>()
                .expect("native navigation values"),
        )
        .expect("host navigation");
        tree.update(host.root_element().expect("navigation root"));
        let editor = tree
            .resolve_node(&FocusTarget::Key("input-name".into()))
            .expect("named input is mounted");
        let (input_id, callback) = listener_for(&initial, "input-name", 23);
        let mut value = tree
            .text_input_value(editor)
            .expect("input value")
            .to_owned();
        assert!(value.is_empty(), "gallery name input starts empty");
        for (sample, position) in ["initial", "append", "middle"].iter().enumerate() {
            let (at, insertion) = match *position {
                "initial" => (0, "a".repeat(1_000_000)),
                "append" => (value.len(), "x".to_owned()),
                "middle" => (value.len() / 2, "x".to_owned()),
                _ => unreachable!(),
            };
            tree.place_text_cursor(editor, at, false);
            let native_started = Instant::now();
            let update = tree.paste_text(Some(editor), &insertion);
            let native_ms = native_started.elapsed().as_secs_f64() * 1_000.0;
            value.insert_str(at, &insertion);
            assert!(
                !update
                    .events
                    .iter()
                    .any(|event| matches!(event.kind, UiEventKind::TextChanged(_)))
            );
            let changed = update
                .events
                .into_iter()
                .find_map(|event| match event.kind {
                    UiEventKind::TextEdited(edit) => Some(UiEventKind::TextEdited(edit)),
                    _ => None,
                })
                .expect("native edit emits a delta");
            let encode_started = Instant::now();
            let delivery = json!({
                "node": {"slot": input_id.slot, "generation": input_id.generation},
                "callback": callback,
                "payload": ui_event_payload(&changed),
            })
            .to_string();
            let encode_ms = encode_started.elapsed().as_secs_f64() * 1_000.0;
            let js_started = Instant::now();
            gallery
                .deliver(&delivery)
                .expect("controlled input callback");
            let js_ms = js_started.elapsed().as_secs_f64() * 1_000.0;
            let mut wire = batches.try_iter().collect::<Vec<_>>();
            let scheduler_started = Instant::now();
            for step in 0..8 {
                gallery
                    .tick((300 + sample * 200 + step * 20) as f64)
                    .expect("adapter input tick");
                wire.extend(batches.try_iter());
            }
            let scheduler_ms = scheduler_started.elapsed().as_secs_f64() * 1_000.0;
            let decode_started = Instant::now();
            let wire_bytes = wire.iter().map(String::len).sum::<usize>();
            let operations = wire
                .iter()
                .flat_map(|batch| decode_wire_operations(batch).expect("native input batch"))
                .collect::<Vec<_>>();
            let decode_ms = decode_started.elapsed().as_secs_f64() * 1_000.0;
            assert!(
                !contains_value(&operations, &value),
                "unchanged controlled value must not echo"
            );
            let native = operations
                .into_iter()
                .map(WireOperation::into_native)
                .collect::<Result<Vec<_>, _>>()
                .expect("native controlled values");
            let host_started = Instant::now();
            if !native.is_empty() {
                host.commit(&native).expect("controlled host commit");
            }
            let host_ms = host_started.elapsed().as_secs_f64() * 1_000.0;
            let tree_started = Instant::now();
            let invalidation = if native.is_empty() {
                argui_ui::TreeUpdate::None
            } else {
                tree.update(host.root_element().expect("controlled root"))
            };
            let tree_ms = tree_started.elapsed().as_secs_f64() * 1_000.0;
            assert_eq!(tree.text_input_value(editor), Some(value.as_str()));
            eprintln!(
                "controlled-input adapter={adapter} sample={sample} position={position} bytes={} event_bytes={} wire_bytes={wire_bytes} native_edit_ms={native_ms:.2} event_encode_ms={encode_ms:.2} quickjs_ms={js_ms:.2} scheduler_tick_ms={scheduler_ms:.2} wire_decode_ms={decode_ms:.2} host_commit_ms={host_ms:.2} tree_update_ms={tree_ms:.2} invalidation={invalidation:?}",
                value.len(),
                delivery.len(),
            );
        }
        let (submit_id, submit_callback) = listener_for(&initial, "input-name", 5);
        gallery
            .deliver(
                &json!({
                    "node": {"slot": submit_id.slot, "generation": submit_id.generation},
                    "callback": submit_callback,
                    "payload": {"kind": "submit", "text": value},
                })
                .to_string(),
            )
            .expect("submit controlled text");
        let mut submitted = collect_batches(&batches);
        for step in 0..8 {
            if contains_value(&submitted, &format!("Submitted: {value}")) {
                break;
            }
            gallery
                .tick(f64::from(900 + step * 20))
                .expect("submit state tick");
            submitted.extend(collect_batches(&batches));
        }
        assert!(
            contains_value(&submitted, &format!("Submitted: {value}")),
            "framework state includes all edits"
        );
        gallery.dispose().expect("gallery disposal");
    }
}

/// Resolves a native event listener for the keyed control in the initial gallery batch.
fn listener_for(
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
        .expect("control has event listener");
    (node, callback)
}
