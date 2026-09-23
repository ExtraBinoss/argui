//! Native window and embedded QuickJS process for the shared gallery module.

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, Instant},
};

use crate::{
    AnimationSnapshot, QuickJsGallery,
    comparison::navigation_callback,
    telemetry::{
        JsCounts, ProfileSummary, mark_commit_for_presentation, observe_profile, report_remaining,
    },
};
use argui_core::{Key, KeyState};
use argui_host::Host;
use argui_platform::{
    ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig, WindowKey,
};
use argui_render::RendererConfig;
use argui_runtime::{
    NativeHostAssets, NativeHostBatch, NativeHostDelivery, RuntimeError, WireOperation,
    run_application, run_native_host,
};
use argui_ui::UiEventKind;
use serde_json::{Value, json};

#[cfg(target_os = "android")]
const NOTO_SANS: &[u8] =
    include_bytes!("../../../../crates/argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

/// Runs the embedded Solid gallery in a desktop native window.
///
/// # Errors
/// Returns an error if the schema, gallery bundle, QuickJS engine, or native window cannot start.
pub fn run_desktop() -> Result<(), Box<dyn std::error::Error>> {
    if comparison_mode() == Some("rust") {
        return run_snapshot_desktop();
    }
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let result = run_gallery(|host, assets, batches, deliveries| {
        run_native_host(
            WindowConfig {
                title: "Argui Gallery / QuickJS".into(),
                ..WindowConfig::default()
            },
            RendererConfig {
                profiling: comparison_mode().is_some(),
                ..RendererConfig::default()
            },
            host,
            assets,
            batches,
            deliveries,
            move |event| observe_profile(&observed, &event, "quickjs"),
        )
    });
    if comparison_mode().is_some() {
        report_remaining(&profiles, "quickjs");
    }
    result
}

/// Runs the embedded Solid gallery from Android's native activity.
///
/// `android_app` is the activity handle passed by Android. Returns after the
/// activity's event loop exits.
///
/// # Errors
/// Returns an error if the schema, gallery bundle, QuickJS engine, or native window cannot start.
#[cfg(target_os = "android")]
pub fn run_android(
    android_app: argui_android::AndroidApp,
) -> Result<(), Box<dyn std::error::Error>> {
    if comparison_mode() == Some("rust") {
        return run_snapshot_android(android_app);
    }
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let result = run_gallery(|host, assets, batches, deliveries| {
        let text_engine = argui_text::TextEngine::from_embedded_fonts(
            [NOTO_SANS],
            "Noto Sans",
            "Noto Sans",
            "Noto Sans",
        );
        argui_runtime::run_android_native_host_with_text_engine(
            android_app,
            WindowConfig {
                title: "Argui Gallery / QuickJS".into(),
                ..WindowConfig::default()
            },
            RendererConfig {
                profiling: comparison_mode().is_some(),
                ..RendererConfig::default()
            },
            text_engine,
            host,
            assets,
            batches,
            deliveries,
            move |event| observe_profile(&observed, &event, "quickjs"),
        )
    });
    if comparison_mode().is_some() {
        report_remaining(&profiles, "quickjs");
    }
    result
}

/// Starts the JavaScript actor and passes its host channels to `launch`.
///
/// `launch` owns the native UI event loop and returns its runtime result.
/// Returns after the event loop exits and signals the JavaScript actor to stop.
///
/// # Errors
/// Returns an error if the schema, embedded bundle, QuickJS engine, or native window cannot start.
fn run_gallery(
    launch: impl FnOnce(
        Host,
        NativeHostAssets,
        Receiver<NativeHostBatch>,
        Sender<NativeHostDelivery>,
    ) -> Result<(), RuntimeError>,
) -> Result<(), Box<dyn std::error::Error>> {
    let host = Host::with_builtins()?;
    let contract_json = include_str!("../../../../packages/host/src/contract.generated.json");
    let contract: Value = serde_json::from_str(contract_json)?;
    if contract["abiHash"].as_str() != Some(&host.abi_hash().to_string()) {
        return Err("generated JavaScript contract is stale; run bun run generate:jsx".into());
    }
    let source = include_str!("../../dist/gallery-core.mjs");

    let (wire_sender, wire_receiver) = mpsc::channel();
    let (batch_sender, batches) = mpsc::channel();
    let (errors_sender, errors) = mpsc::channel();
    std::thread::spawn(move || relay_batches(wire_receiver, batch_sender, errors_sender));

    let (deliveries, events) = mpsc::channel();
    let (stop_sender, stop) = mpsc::channel();
    let (ready_sender, ready) = mpsc::channel();
    std::thread::spawn(move || {
        let initial = std::rc::Rc::new(std::cell::RefCell::new(Some(Vec::<String>::new())));
        let captured = std::rc::Rc::clone(&initial);
        let batch_count = Arc::new(AtomicU64::new(0));
        let operation_count = Arc::new(AtomicU64::new(0));
        let counted = Arc::clone(&batch_count);
        let counted_operations = Arc::clone(&operation_count);
        let sender = wire_sender;
        let gallery = QuickJsGallery::new(source, contract_json, "mountGallery", move |json| {
            counted.fetch_add(1, Ordering::Relaxed);
            if let Some(initial) = captured.borrow_mut().as_mut() {
                initial.push(json.clone());
            }
            match serde_json::from_str::<Vec<WireOperation>>(&json) {
                Ok(batch) => {
                    counted_operations.fetch_add(batch.len() as u64, Ordering::Relaxed);
                    sender.send(batch).map_or_else(
                        |_| "native UI thread closed".to_string(),
                        |()| String::new(),
                    )
                }
                Err(error) => format!("invalid QuickJS operation batch: {error}"),
            }
        });
        let gallery = match gallery {
            Ok(gallery) => gallery,
            Err(error) => {
                let _ = ready_sender.send(Err(error));
                return;
            }
        };
        if comparison_mode().is_some() {
            let navigation = initial.borrow().as_ref().cloned().unwrap_or_default();
            if let Err(error) = navigation_callback(&navigation)
                .and_then(|callback| gallery.deliver(&callback.to_string()))
            {
                let _ = ready_sender.send(Err(error));
                return;
            }
        }
        *initial.borrow_mut() = None;
        let counts = JsCounts::new(batch_count, operation_count);
        let (startup_batches, startup_operations) = counts.startup();
        if comparison_mode().is_some() {
            eprintln!(
                "argui-comparison mode=quickjs startup_batches={startup_batches} startup_operations={startup_operations}"
            );
        }
        let _ = ready_sender.send(Ok(()));
        if let Err(error) = run_js_loop(&gallery, events, errors, stop, &counts) {
            eprintln!("{error}");
        }
        if let Err(error) = gallery.dispose() {
            eprintln!("{error}");
        }
    });
    ready.recv().map_err(|error| error.to_string())??;

    let assets = argui_gallery_assets::load()?;
    let result = launch(host, assets, batches, deliveries);
    let _ = stop_sender.send(());
    result?;
    Ok(())
}

/// Sends each decoded JS transaction to the UI thread and reports rejected commits.
fn relay_batches(
    wire: Receiver<Vec<WireOperation>>,
    batches: Sender<NativeHostBatch>,
    errors: Sender<String>,
) {
    let mut batch_sequence = 0_u64;
    for operations in wire {
        batch_sequence += 1;
        let operation_count = operations.len();
        let submitted = Instant::now();
        let (reply, completion) = mpsc::channel();
        if batches
            .send(NativeHostBatch {
                window: WindowKey::main(),
                operations,
                reply,
            })
            .is_err()
        {
            break;
        }
        match completion.recv() {
            Ok(Ok(commit)) => {
                if comparison_mode().is_some() {
                    if batch_sequence > 1 && operation_count >= 50 {
                        mark_commit_for_presentation();
                    }
                    eprintln!(
                        "argui-comparison host_batch_operations={operation_count} host_changed_nodes={} host_update={:?} queue_and_commit_ms={:.3}",
                        commit.changed_nodes,
                        commit.update,
                        submitted.elapsed().as_secs_f64() * 1000.0
                    );
                }
            }
            Ok(Err(error)) => {
                let _ = errors.send(error);
                break;
            }
            Err(_) => break,
        }
    }
}

/// Pumps native events, timer callbacks, and QuickJS microtasks until shutdown.
///
/// # Errors
/// Returns an error when JavaScript fails or the native host rejects a batch.
fn run_js_loop(
    gallery: &QuickJsGallery,
    events: Receiver<NativeHostDelivery>,
    errors: Receiver<String>,
    stop: Receiver<()>,
    counts: &JsCounts,
) -> Result<(), String> {
    let started = Instant::now();
    let mut work = Duration::ZERO;
    let mut ticks = 0_u64;
    let mut deliveries = 0_u64;
    loop {
        if stop.try_recv().is_ok() {
            break;
        }
        if let Ok(error) = errors.try_recv() {
            return Err(format!("native commit rejected: {error}"));
        }
        let entered = Instant::now();
        let delay = gallery.next_wake(started.elapsed().as_secs_f64() * 1000.0)?;
        work += entered.elapsed();
        if let Ok(delivery) = events.recv_timeout(delay) {
            let entered = Instant::now();
            gallery.deliver(&event_json(&delivery).to_string())?;
            if comparison_mode().is_some() && matches!(delivery.kind, UiEventKind::Click(_)) {
                eprintln!(
                    "argui-comparison click_js_callback_ms={:.3}",
                    entered.elapsed().as_secs_f64() * 1000.0
                );
            }
            work += entered.elapsed();
            deliveries += 1;
        }
        while let Ok(delivery) = events.try_recv() {
            let entered = Instant::now();
            gallery.deliver(&event_json(&delivery).to_string())?;
            if comparison_mode().is_some() && matches!(delivery.kind, UiEventKind::Click(_)) {
                eprintln!(
                    "argui-comparison click_js_callback_ms={:.3}",
                    entered.elapsed().as_secs_f64() * 1000.0
                );
            }
            work += entered.elapsed();
            deliveries += 1;
        }
        let entered = Instant::now();
        gallery.tick(started.elapsed().as_secs_f64() * 1000.0)?;
        work += entered.elapsed();
        ticks += 1;
        if comparison_mode().is_some() && ticks.is_multiple_of(5) {
            counts.report(deliveries, ticks, work, started.elapsed());
        }
    }
    if comparison_mode().is_some() {
        counts.report(deliveries, ticks, work, started.elapsed());
    }
    Ok(())
}

/// Encodes the input needed by the shared gallery controls.
fn event_json(delivery: &NativeHostDelivery) -> Value {
    let payload = match &delivery.kind {
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
        UiEventKind::SemanticAction { .. } => json!({"kind": "semantic_action"}),
        other => json!({"kind": format!("{:?}", other.event_type())}),
    };
    json!({
        "node": {"slot": delivery.callback.node.slot(), "generation": delivery.callback.node.generation()},
        "callback": delivery.callback.callback.0,
        "payload": payload,
    })
}

/// Gives controls stable key names for navigation and activation.
fn key_name(key: &Key) -> String {
    match key {
        Key::Character(value) => value.clone(),
        other => format!("{other:?}"),
    }
}

/// Selects the requested comparison presentation on desktop or in an Android feature build.
fn comparison_mode() -> Option<&'static str> {
    if cfg!(feature = "comparison-rust") {
        Some("rust")
    } else if cfg!(feature = "comparison-quickjs") {
        Some("quickjs")
    } else {
        match std::env::var("ARGUI_GALLERY_COMPARE").ok().as_deref() {
            Some("rust") => Some("rust"),
            Some("quickjs") => Some("quickjs"),
            _ => None,
        }
    }
}

/// Creates the exact Animation Lab snapshot from the same generated bundle as the live host.
/// Returns the frozen model and its decoded media.
///
/// # Errors
/// Returns an error when the bundle, schema, media, or snapshot fails validation.
fn animation_snapshot() -> Result<AnimationSnapshot, Box<dyn std::error::Error>> {
    let host = Host::with_builtins()?;
    let contract_json = include_str!("../../../../packages/host/src/contract.generated.json");
    let contract: Value = serde_json::from_str(contract_json)?;
    if contract["abiHash"].as_str() != Some(&host.abi_hash().to_string()) {
        return Err("generated JavaScript contract is stale; run bun run generate:jsx".into());
    }
    let source = include_str!("../../dist/gallery-core.mjs");
    let assets = argui_gallery_assets::load()?;
    Ok(AnimationSnapshot::build(source, contract_json, assets)?)
}

/// Builds the same window identity and viewport settings for both comparison presentations.
/// Returns a single-window application configuration.
///
/// # Errors
/// Returns an error if the application identifier is invalid.
fn snapshot_config() -> Result<ApplicationConfig, Box<dyn std::error::Error>> {
    Ok(ApplicationConfig::new(
        ApplicationIdentity::new(
            ApplicationId::new("dev.argui.solidgallery")?,
            "Argui Gallery Comparison",
            IconSet::new(),
        ),
        WindowConfig {
            title: "Argui Gallery / Rust snapshot".into(),
            ..WindowConfig::default()
        },
    ))
}

/// Runs the frozen Animation Lab tree as a direct desktop Rust model.
///
/// # Errors
/// Returns an error if snapshot materialization or the native window fails.
fn run_snapshot_desktop() -> Result<(), Box<dyn std::error::Error>> {
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let snapshot = animation_snapshot()?;
    eprintln!(
        "argui-comparison mode=rust startup_batches={} startup_operations={}",
        snapshot.wire_batches().len(),
        snapshot.wire_batches().iter().map(Vec::len).sum::<usize>()
    );
    let result = run_application(
        snapshot_config()?,
        RendererConfig {
            profiling: true,
            ..RendererConfig::default()
        },
        snapshot,
        move |event| observe_profile(&observed, &event, "rust"),
    );
    report_remaining(&profiles, "rust");
    Ok(result?)
}

/// Runs the frozen Animation Lab tree as a direct Android Rust model.
/// `android_app` supplies the native Activity and its Pixel viewport.
///
/// # Errors
/// Returns an error if snapshot materialization or the native Activity fails.
#[cfg(target_os = "android")]
fn run_snapshot_android(
    android_app: argui_android::AndroidApp,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let text_engine = argui_text::TextEngine::from_embedded_fonts(
        [NOTO_SANS],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    let snapshot = animation_snapshot()?;
    eprintln!(
        "argui-comparison mode=rust startup_batches={} startup_operations={}",
        snapshot.wire_batches().len(),
        snapshot.wire_batches().iter().map(Vec::len).sum::<usize>()
    );
    let result = argui_runtime::run_android_application_with_text_engine(
        android_app,
        snapshot_config()?,
        RendererConfig {
            profiling: true,
            ..RendererConfig::default()
        },
        text_engine,
        snapshot,
        move |event| observe_profile(&observed, &event, "rust"),
    );
    report_remaining(&profiles, "rust");
    Ok(result?)
}
